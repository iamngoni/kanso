//! Spelling in the editor: misspelled words get a red wavy underline, and
//! right-clicking one offers the system's guesses, Learn Spelling and
//! Ignore Spelling (Settings › Editor › Writing turns it off).
//!
//! Only prose is checked. [`prose_words`] finds it from the same Markdown
//! parse the preview uses (markdown-rs, through `markdown_safe`): the text
//! of paragraphs, headings, quotes, lists, tables and link labels, but not
//! code spans or fenced code, link targets, bare web addresses, front
//! matter, HTML or math. Inside that text it leaves out `[[wiki links]]`,
//! `#tags`, addresses and file names, and words with digits, underscores
//! or capitals past their first letter (ALL CAPS, camelCase).
//!
//! The words are checked a moment after typing stops, off the main thread,
//! and only on the lines in view and a margin around them; scrolling past
//! that margin checks again. The checker is a [`Dictionary`]: the system's
//! on macOS (`NSSpellChecker`, which may be asked from any thread), and a
//! word list in tests. Windows and Linux have none yet, so the setting is
//! hidden there ([`AVAILABLE`]).

use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use gpui_kit::base::input::{TextDecoration, TextDecorationCollection};
use gpui_kit::component::input::EditorState;
use gpui_kit::*;
use markdown::ParseOptions;
use markdown::mdast::Node;

use crate::theme::Colors;

/// Whether this system has a spelling checker Kanso can use.
pub const AVAILABLE: bool = cfg!(target_os = "macos");

/// How long typing has to pause before the words are checked again.
pub const CHECK_DELAY: Duration = Duration::from_millis(350);

/// How many lines above and below the ones in view are checked too, so a
/// short scroll finds them underlined already.
const MARGIN_LINES: usize = 80;

/// The lines checked before the editor has been laid out.
const FIRST_LINES: usize = 120;

/// The guesses a context menu offers, at most.
pub const MAX_GUESSES: usize = 6;

/// Words a [`Speller`] remembers the verdict on before it starts over.
const KEPT_VERDICTS: usize = 50_000;

// MARK: Finding the words

/// The parse the preview makes, with front matter and math, so neither is
/// taken for prose.
fn parse_options() -> ParseOptions {
    let mut options = ParseOptions::gfm();
    options.constructs.frontmatter = true;
    options.constructs.math_text = true;
    options.constructs.math_flow = true;
    options
}

/// The words of `text` to check, as sorted, disjoint byte ranges on
/// character boundaries: the prose's words, without the parts of it that
/// aren't words (see the module's notes). Text markdown-rs can't parse has
/// none.
pub fn prose_words(text: &str) -> Vec<Range<usize>> {
    let Ok(root) = kanso_render::markdown_safe::to_mdast(text, &parse_options()) else {
        return Vec::new();
    };
    let mut prose = Vec::new();
    collect_prose(&root, &mut prose);
    prose.sort_by_key(|range| range.start);
    let wikis = wiki_spans(text);
    let mut words = Vec::new();
    for range in prose {
        for word in chunk_words(text, range) {
            let first = wikis.partition_point(|w| w.end <= word.start);
            if wikis.get(first).is_some_and(|w| w.start < word.end) {
                continue;
            }
            words.push(word);
        }
    }
    words
}

/// The byte ranges of the plain text under `node` that is prose.
fn collect_prose(node: &Node, out: &mut Vec<Range<usize>>) {
    match node {
        Node::Text(text) => {
            if let Some(at) = &text.position {
                out.push(at.start.offset..at.end.offset);
            }
        }
        Node::Code(_)
        | Node::InlineCode(_)
        | Node::Math(_)
        | Node::InlineMath(_)
        | Node::Html(_)
        | Node::Yaml(_)
        | Node::Toml(_)
        | Node::Definition(_)
        | Node::Image(_)
        | Node::ImageReference(_)
        | Node::FootnoteReference(_) => {}
        Node::Link(link) if is_bare_address(link) => {}
        _ => {
            for child in node.children().into_iter().flatten() {
                collect_prose(child, out);
            }
        }
    }
}

/// A link whose text is its own address: a web address or an email
/// address written out (`https://…`, `www.…`, `<…>`), which markdown-rs
/// gives a `http://` or `mailto:` in front.
fn is_bare_address(link: &markdown::mdast::Link) -> bool {
    match link.children.as_slice() {
        [Node::Text(text)] => {
            let text = text.value.as_str();
            let url = link.url.as_str();
            url == text
                || url.strip_prefix("http://") == Some(text)
                || url.strip_prefix("mailto:") == Some(text)
        }
        _ => false,
    }
}

/// `[[…]]` spans, each on one line, in order.
fn wiki_spans(text: &str) -> Vec<Range<usize>> {
    let mut spans = Vec::new();
    let mut line_start = 0;
    for line in text.split_inclusive('\n') {
        let mut from = 0;
        while let Some(open) = line[from..].find("[[") {
            let open = from + open;
            let Some(close) = line[open + 2..].find("]]") else {
                break;
            };
            let end = open + 2 + close + 2;
            spans.push(line_start + open..line_start + end);
            from = end;
        }
        line_start += line.len();
    }
    spans
}

/// The words to check in `range` of `text`, a run of prose. Each
/// whitespace-separated piece that is an address, a file name, a tag or
/// an email address is passed over whole.
fn chunk_words(text: &str, range: Range<usize>) -> Vec<Range<usize>> {
    let mut words = Vec::new();
    let slice = &text[range.clone()];
    let mut start = None;
    for (i, c) in slice.char_indices().chain([(slice.len(), ' ')]) {
        match (c.is_whitespace(), start) {
            (true, Some(s)) => {
                start = None;
                let chunk = &slice[s..i];
                if !skips_chunk(chunk) {
                    words.extend(
                        chunk_tokens(chunk)
                            .map(|w| range.start + s + w.start..range.start + s + w.end),
                    );
                }
            }
            (false, None) => start = Some(i),
            _ => {}
        }
    }
    words
}

/// Whether a piece of prose between spaces is no words at all: a web
/// address, a domain or file name (`kanso.app`, `notes.md`), a `#tag`, an
/// email address or handle.
fn skips_chunk(chunk: &str) -> bool {
    let bare = chunk.trim_start_matches(|c: char| !c.is_alphanumeric() && c != '#');
    if chunk.contains("://") || bare.starts_with("www.") || chunk.contains(['#', '@']) {
        return true;
    }
    // A dot with a letter right after it, as in a domain or a file name;
    // a sentence's full stop has a space after it, and so ends the piece.
    chunk
        .char_indices()
        .any(|(i, c)| c == '.' && chunk[i + 1..].starts_with(char::is_alphabetic))
}

/// The words in one piece of prose, as ranges in it: runs of letters with
/// apostrophes inside them (`don't`, `Kanso’s`), leaving out runs with
/// digits or underscores, single letters, capitals past the first letter,
/// and an HTML entity's or an escape's letters (`&amp;`, `\n`).
fn chunk_tokens(chunk: &str) -> impl Iterator<Item = Range<usize>> + '_ {
    let is_part = |c: char| c.is_alphanumeric() || matches!(c, '\'' | '’' | '_');
    let mut runs = Vec::new();
    let mut start = None;
    for (i, c) in chunk.char_indices().chain([(chunk.len(), ' ')]) {
        match (is_part(c), start) {
            (false, Some(s)) => {
                runs.push(s..i);
                start = None;
            }
            (true, None) => start = Some(i),
            _ => {}
        }
    }
    runs.into_iter().filter_map(move |run| {
        let quote = |c: char| matches!(c, '\'' | '’');
        let raw = &chunk[run.clone()];
        let trimmed = raw.trim_start_matches(quote);
        let start = run.start + (raw.len() - trimmed.len());
        let word = trimmed.trim_end_matches(quote);
        let range = start..start + word.len();
        let before = chunk[..run.start].chars().next_back();
        (is_word(word) && !matches!(before, Some('&' | '\\'))).then_some(range)
    })
}

/// Whether a run of letters is a word to check.
fn is_word(word: &str) -> bool {
    let mut chars = word.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_alphabetic()
        && word.chars().count() >= 2
        && !word.chars().any(|c| c.is_numeric() || c == '_')
        && !chars.any(char::is_uppercase)
}

/// The words of `words` that start within `window`.
pub fn in_window(words: &[Range<usize>], window: &Range<usize>) -> Vec<Range<usize>> {
    let first = words.partition_point(|w| w.start < window.start);
    words[first..]
        .iter()
        .take_while(|w| w.start < window.end)
        .cloned()
        .collect()
}

/// The byte range of `lines` (line numbers, from 0) in `text`.
pub fn line_span(text: &str, lines: &Range<usize>) -> Range<usize> {
    let mut start = None;
    let mut offset = 0;
    for (n, line) in text.split_inclusive('\n').enumerate() {
        if n == lines.start {
            start = Some(offset);
        }
        if n == lines.end {
            return start.unwrap_or(offset)..offset;
        }
        offset += line.len();
    }
    start.unwrap_or(text.len())..text.len()
}

// MARK: The checker

/// A spelling dictionary: the system's, or a word list in tests. It may be
/// asked from any thread.
pub trait Dictionary: Send + Sync {
    /// Whether `word` is spelled right.
    fn knows(&self, word: &str) -> bool;
    /// What `word` might have been meant to be, likeliest first.
    fn guesses(&self, word: &str) -> Vec<String>;
    /// Takes `word` as spelled right from now on, in every app.
    fn learn(&self, word: &str);
}

/// A [`Dictionary`] with the verdicts it gave kept, and the words ignored
/// since Kanso started.
pub struct Speller {
    dictionary: Arc<dyn Dictionary>,
    verdicts: Mutex<HashMap<String, bool>>,
    ignored: Mutex<HashSet<String>>,
}

impl Speller {
    /// Only the macOS system speller and tests make one.
    #[cfg(any(target_os = "macos", test))]
    pub fn new(dictionary: Arc<dyn Dictionary>) -> Speller {
        Speller {
            dictionary,
            verdicts: Mutex::new(HashMap::new()),
            ignored: Mutex::new(HashSet::new()),
        }
    }

    /// Whether `word` is spelled right or ignored.
    pub fn knows(&self, word: &str) -> bool {
        if self
            .ignored
            .lock()
            .is_ok_and(|ignored| ignored.contains(word))
        {
            return true;
        }
        if let Some(known) = self.verdicts.lock().ok().and_then(|v| v.get(word).copied()) {
            return known;
        }
        let known = self.dictionary.knows(word);
        if let Ok(mut verdicts) = self.verdicts.lock() {
            if verdicts.len() >= KEPT_VERDICTS {
                verdicts.clear();
            }
            verdicts.insert(word.to_owned(), known);
        }
        known
    }

    /// The words of `words` (ranges in `text`) that are misspelled.
    pub fn misspelled(&self, text: &str, words: &[Range<usize>]) -> Vec<Range<usize>> {
        words
            .iter()
            .filter(|w| !self.knows(&text[(*w).clone()]))
            .cloned()
            .collect()
    }

    /// The first [`MAX_GUESSES`] guesses for `word`.
    pub fn guesses(&self, word: &str) -> Vec<String> {
        let mut guesses = self.dictionary.guesses(word);
        guesses.truncate(MAX_GUESSES);
        guesses
    }

    /// Learn Spelling: the dictionary takes the word for good.
    pub fn learn(&self, word: &str) {
        self.dictionary.learn(word);
        if let Ok(mut verdicts) = self.verdicts.lock() {
            verdicts.insert(word.to_owned(), true);
        }
    }

    /// Ignore Spelling: the word passes until Kanso quits.
    pub fn ignore(&self, word: &str) {
        if let Ok(mut ignored) = self.ignored.lock() {
            ignored.insert(word.to_owned());
        }
    }
}

/// The speller tests put in place of the system's.
struct SpellerOverride(Arc<Speller>);

impl Global for SpellerOverride {}

/// The speller in use: one set with [`set_dictionary`], or the system's.
/// `None` where there is none, and in tests until one is set.
pub fn speller(cx: &App) -> Option<Arc<Speller>> {
    if let Some(set) = cx.try_global::<SpellerOverride>() {
        return Some(set.0.clone());
    }
    static SYSTEM: OnceLock<Option<Arc<Speller>>> = OnceLock::new();
    SYSTEM.get_or_init(system_speller).clone()
}

/// Checks with `dictionary` from now on, in place of the system's.
#[cfg(test)]
pub fn set_dictionary(dictionary: Arc<dyn Dictionary>, cx: &mut App) {
    cx.set_global(SpellerOverride(Arc::new(Speller::new(dictionary))));
}

fn system_speller() -> Option<Arc<Speller>> {
    #[cfg(all(target_os = "macos", not(test)))]
    {
        Some(Arc::new(Speller::new(Arc::new(
            crate::spellcheck_macos::SystemDictionary,
        ))))
    }
    #[cfg(not(all(target_os = "macos", not(test))))]
    {
        None
    }
}

/// A word list, for tests: every listed word is right, and the guesses for
/// a word are the listed words of the same length with the same first
/// letter.
#[cfg(test)]
pub struct WordList(Mutex<Vec<String>>);

#[cfg(test)]
impl WordList {
    pub fn new(words: &[&str]) -> WordList {
        WordList(Mutex::new(words.iter().map(|w| (*w).to_owned()).collect()))
    }
}

#[cfg(test)]
impl Dictionary for WordList {
    fn knows(&self, word: &str) -> bool {
        let lower = word.to_lowercase();
        self.0
            .lock()
            .unwrap()
            .iter()
            .any(|w| *w == word || *w == lower)
    }

    fn guesses(&self, word: &str) -> Vec<String> {
        self.0
            .lock()
            .unwrap()
            .iter()
            .filter(|w| w.len() == word.len() && w.chars().next() == word.chars().next())
            .cloned()
            .collect()
    }

    fn learn(&self, word: &str) {
        self.0.lock().unwrap().push(word.to_owned());
    }
}

// MARK: The editor

/// A guess from the context menu: `with` goes in place of `word` at
/// `range`, if the text there still says it.
#[derive(Clone, Debug, PartialEq, Action)]
#[action(namespace = kanso, no_json)]
pub struct CorrectSpelling {
    pub range: Range<usize>,
    pub word: String,
    pub with: String,
}

/// Learn Spelling, from the context menu.
#[derive(Clone, Debug, PartialEq, Action)]
#[action(namespace = kanso, no_json)]
pub struct LearnSpelling {
    pub word: String,
}

/// Ignore Spelling, from the context menu.
#[derive(Clone, Debug, PartialEq, Action)]
#[action(namespace = kanso, no_json)]
pub struct IgnoreSpelling {
    pub word: String,
}

/// The spelling items at the top of the context menu for `misspelling`:
/// a guess each (when the text can change), then Learn Spelling and
/// Ignore Spelling.
pub fn menu_items(
    misspelling: &Misspelling,
    guesses: Vec<String>,
    editable: bool,
) -> Vec<(String, Box<dyn Action>)> {
    let mut items: Vec<(String, Box<dyn Action>)> = Vec::new();
    if editable {
        for guess in guesses {
            items.push((
                guess.clone(),
                Box::new(CorrectSpelling {
                    range: misspelling.range.clone(),
                    word: misspelling.word.clone(),
                    with: guess,
                }),
            ));
        }
    }
    let word = misspelling.word.clone();
    items.push((
        "Learn Spelling".to_owned(),
        Box::new(LearnSpelling { word: word.clone() }),
    ));
    items.push((
        "Ignore Spelling".to_owned(),
        Box::new(IgnoreSpelling { word }),
    ));
    items
}

/// The underlines under the misspelled words in the editor's text, kept as
/// text decorations that follow edits, and checked again a moment after
/// typing stops or once scrolling leaves the lines checked.
#[derive(Default)]
pub struct SpellMarks {
    enabled: bool,
    collection: Option<TextDecorationCollection>,
    task: Option<Task<()>>,
    /// The lines last checked (from 0, end excluded).
    checked: Option<Range<usize>>,
    /// How many words are underlined now.
    count: usize,
}

/// A misspelled word in the text: where it is and what it says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Misspelling {
    pub range: Range<usize>,
    pub word: String,
}

impl SpellMarks {
    /// Turns the checking on or off, as Settings has it; off takes the
    /// underlines away.
    pub fn set_enabled(&mut self, enabled: bool, cx: &mut App) {
        self.enabled = enabled;
        if !enabled {
            self.clear(cx);
        }
    }

    /// Takes every underline away.
    pub fn clear(&mut self, cx: &mut App) {
        self.task = None;
        self.checked = None;
        self.count = 0;
        if let Some(collection) = &self.collection {
            collection.clear(cx);
        }
    }

    /// How many words are underlined now.
    #[cfg(test)]
    pub fn count(&self) -> usize {
        self.count
    }

    /// The misspelled word at byte `at` of `body`'s text, or touching it.
    pub fn at(&self, body: &Entity<EditorState>, at: usize, cx: &App) -> Option<Misspelling> {
        let collection = self.collection.as_ref()?;
        let ranges = collection.get_ranges(cx);
        let range = ranges.into_iter().find(|r| r.start <= at && at <= r.end)?;
        let word = body.read(cx).text().slice(range.clone()).to_string();
        Some(Misspelling { range, word })
    }

    /// Checks the lines in view (and around them) after `delay`, off the
    /// main thread. The underlines already there follow the edits
    /// meanwhile, and a newer call takes this one's place.
    pub fn refresh_soon<V: 'static>(
        &mut self,
        delay: Duration,
        body: Entity<EditorState>,
        owner: WeakEntity<V>,
        pick: fn(&mut V) -> &mut SpellMarks,
        cx: &mut App,
    ) {
        let Some(speller) = speller(cx).filter(|_| self.enabled) else {
            self.clear(cx);
            return;
        };
        self.task = Some(cx.spawn(async move |cx| {
            if !delay.is_zero() {
                cx.background_executor().timer(delay).await;
            }
            let (text, lines) = cx.update(|cx| {
                let state = body.read(cx);
                let lines = state.visible_row_range().map_or(0..FIRST_LINES, |rows| {
                    rows.start.saturating_sub(MARGIN_LINES)..rows.end + MARGIN_LINES
                });
                (state.value(), lines)
            });
            let checked_text = text.clone();
            let lines_checked = checked_lines(&text, lines.clone());
            let misspelled = cx
                .background_executor()
                .spawn(async move {
                    let words = prose_words(&checked_text);
                    let window = line_span(&checked_text, &lines);
                    speller.misspelled(&checked_text, &in_window(&words, &window))
                })
                .await;
            cx.update(|cx| {
                let Some(owner) = owner.upgrade() else {
                    return;
                };
                // Typed over meanwhile: the next check has been asked for.
                if body.read(cx).value() != text {
                    return;
                }
                owner.update(cx, |owner, cx| {
                    let marks = pick(owner);
                    marks.task = None;
                    marks.checked = Some(lines_checked);
                    marks.show(&body, misspelled, cx);
                });
            });
        }));
    }

    /// Checks again soon when the lines in view have left the ones checked.
    pub fn follow_scroll<V: 'static>(
        &mut self,
        body: Entity<EditorState>,
        owner: WeakEntity<V>,
        pick: fn(&mut V) -> &mut SpellMarks,
        cx: &mut App,
    ) {
        if !self.enabled || self.task.is_some() {
            return;
        }
        let Some(rows) = body.read(cx).visible_row_range() else {
            return;
        };
        let inside = self
            .checked
            .as_ref()
            .is_some_and(|c| c.start <= rows.start && rows.end <= c.end);
        if !inside {
            self.refresh_soon(Duration::from_millis(120), body, owner, pick, cx);
        }
    }

    /// Takes the underline off every `word` at once, as Learn and Ignore
    /// do, before the next check.
    pub fn forget(&mut self, body: &Entity<EditorState>, word: &str, cx: &mut App) {
        let text = body.read(cx).text().clone();
        self.keep(body, |range| text.slice(range.clone()) != word, cx);
    }

    /// Takes the underline off the word at `range`, which a guess replaces.
    pub fn forget_range(&mut self, body: &Entity<EditorState>, range: &Range<usize>, cx: &mut App) {
        self.keep(body, |r| r != range, cx);
    }

    fn keep(
        &mut self,
        body: &Entity<EditorState>,
        keep: impl Fn(&Range<usize>) -> bool,
        cx: &mut App,
    ) {
        let Some(collection) = &self.collection else {
            return;
        };
        let left = collection.get_ranges(cx).into_iter().filter(keep).collect();
        self.show(body, left, cx);
    }

    fn show(&mut self, body: &Entity<EditorState>, ranges: Vec<Range<usize>>, cx: &mut App) {
        let color = Colors::get(cx).danger.into();
        let decorations: Vec<TextDecoration> = ranges
            .into_iter()
            .map(|range| {
                TextDecoration::new(
                    range,
                    HighlightStyle {
                        underline: Some(UnderlineStyle {
                            thickness: px(1.),
                            color: Some(color),
                            wavy: true,
                        }),
                        ..Default::default()
                    },
                )
            })
            .collect();
        self.count = decorations.len();
        match &self.collection {
            Some(collection) => collection.set(decorations, cx),
            None if decorations.is_empty() => {}
            None => {
                self.collection = Some(body.update(cx, |state, cx| {
                    state.create_decorations_collection(decorations, cx)
                }));
            }
        }
    }
}

/// `lines`, cut to the lines `text` has, so a short note counts as
/// checked whatever is scrolled into view of it.
fn checked_lines(text: &str, lines: Range<usize>) -> Range<usize> {
    let count = text.split_inclusive('\n').count().max(1);
    if lines.end >= count {
        lines.start..usize::MAX
    } else {
        lines
    }
}

// The generated notes keep each piece's words as a list of ranges, often
// of one.
#[cfg(test)]
#[allow(clippy::single_range_in_vec_init)]
mod tests {
    use super::{Speller, WordList, in_window, is_word, line_span, prose_words};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;
    use std::ops::Range;
    use std::sync::Arc;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    /// The words found, as text.
    fn words(text: &str) -> Vec<&str> {
        prose_words(text).into_iter().map(|r| &text[r]).collect()
    }

    #[test]
    fn plain_prose_is_checked() {
        assert_eq!(
            words("Teh quick fox, don't stop.\n"),
            ["Teh", "quick", "fox", "don't", "stop"]
        );
        assert_eq!(
            words("# A headng\n\n> quoted wrods\n"),
            ["headng", "quoted", "wrods"]
        );
        assert_eq!(words("- [ ] buy mlik\n- item\n"), ["buy", "mlik", "item"]);
        assert_eq!(
            words("| head | cel |\n|---|---|\n| aa | bb |\n"),
            ["head", "cel", "aa", "bb"]
        );
        assert_eq!(words("*emph* and **strng**\n"), ["emph", "and", "strng"]);
        // Curly and straight quotes around a word aren't part of it.
        assert_eq!(
            words("'quoted' ‘curly’ Kanso’s\n"),
            ["quoted", "curly", "Kanso’s"]
        );
    }

    #[test]
    fn code_and_markup_are_skipped() {
        assert_eq!(words("use `fnord xyzzy` here\n"), ["use", "here"]);
        assert_eq!(words("```\nfnord xyzzy\n```\n\nafter\n"), ["after"]);
        assert_eq!(words("    indented codez\n\nafter\n"), ["after"]);
        assert_eq!(words("---\ntitel: fnord\n---\n\nbody\n"), ["body"]);
        assert_eq!(words("<div>\nfnord\n</div>\n\nafter\n"), ["after"]);
        assert_eq!(
            words("sum $fnord + xyzzy$ here\n\n$$\nqwfp\n$$\n"),
            ["sum", "here"]
        );
        assert_eq!(words("![alt texxt](pic.png)\n"), Vec::<&str>::new());
        assert_eq!(
            words("[[Wiki Notte]] and [[x|Alias Wrod]] end\n"),
            ["and", "end"]
        );
        assert_eq!(
            words("tagged #fnord and #work/clientz\n"),
            ["tagged", "and"]
        );
        assert_eq!(
            words("[labl](https://example.com/fnord) ok\n"),
            ["labl", "ok"]
        );
        // A label that is the end of its address is still a label.
        assert_eq!(words("[fnord](https://e.x/fnord)\n"), ["fnord"]);
        assert_eq!(words("[ref][fnord]\n\n[fnord]: https://e.x\n"), ["ref"]);
        assert_eq!(
            words("see https://example.com/fnord and www.xyzzy.org\n"),
            ["see", "and"]
        );
        assert_eq!(
            words("mail me@exmple.com or kanso.app or notes.md\n"),
            ["mail", "or", "or"]
        );
        assert_eq!(
            words("NASA abc123 snake_case camelCase iPhone a I\n"),
            Vec::<&str>::new()
        );
        assert_eq!(words("fish &amp; chips \\n\n"), ["fish", "chips"]);
    }

    #[test]
    fn windows_and_lines() {
        let text = "one\ntwo\nthree\n";
        assert_eq!(line_span(text, &(1..2)), 4..8);
        assert_eq!(line_span(text, &(0..9)), 0..text.len());
        assert_eq!(line_span(text, &(5..9)), text.len()..text.len());
        let all = prose_words(text);
        assert_eq!(in_window(&all, &(4..8)), vec![4..7]);
    }

    #[test]
    fn speller_learns_and_ignores() {
        let speller = Speller::new(Arc::new(WordList::new(&["the", "cat", "car"])));
        let text = "The cqt sat";
        let found = speller.misspelled(text, &prose_words(text));
        assert_eq!(
            found.iter().map(|r| &text[r.clone()]).collect::<Vec<_>>(),
            ["cqt", "sat"]
        );
        assert_eq!(speller.guesses("cqt"), ["cat", "car"]);
        speller.learn("cqt");
        speller.ignore("sat");
        assert!(speller.misspelled(text, &prose_words(text)).is_empty());
    }

    /// One piece of a generated note: its text, and for each of its words
    /// whether it is prose to check. Offsets are within the piece.
    #[derive(Clone, Debug)]
    struct Piece {
        text: String,
        prose: Vec<Range<usize>>,
        other: Vec<Range<usize>>,
    }

    fn word() -> impl Strategy<Value = String> {
        "[a-z]{2,8}"
    }

    /// Inline content: prose words, or things in a paragraph that aren't.
    fn inline() -> impl Strategy<Value = Piece> {
        prop_oneof![
            4 => word().prop_map(|w| Piece { prose: vec![0..w.len()], other: vec![], text: w }),
            1 => (word(), word()).prop_map(|(a, b)| {
                let text = format!("*{a}* **{b}**");
                Piece { prose: vec![1..1 + a.len(), a.len() + 5..a.len() + 5 + b.len()], other: vec![], text }
            }),
            1 => (word(), word()).prop_map(|(a, b)| {
                let text = format!("`{a} {b}`");
                Piece { prose: vec![], other: vec![0..text.len()], text }
            }),
            1 => (word(), word()).prop_map(|(a, b)| {
                let text = format!("[{a}](https://{b}.example/{a})");
                let target = a.len() + 2..text.len();
                Piece { prose: vec![1..1 + a.len()], other: vec![target], text }
            }),
            1 => (word(), word()).prop_map(|(a, b)| {
                let text = format!("[[{a} {b}]]");
                Piece { prose: vec![], other: vec![0..text.len()], text }
            }),
            1 => word().prop_map(|a| {
                let text = format!("#{a}");
                Piece { prose: vec![], other: vec![0..text.len()], text }
            }),
            1 => word().prop_map(|a| {
                let text = format!("https://{a}.example/{a}");
                Piece { prose: vec![], other: vec![0..text.len()], text }
            }),
            1 => (word(), word()).prop_map(|(a, b)| {
                let text = format!("${a} + {b}$");
                Piece { prose: vec![], other: vec![0..text.len()], text }
            }),
            1 => "[A-Z]{2,5}|[a-z]{1,4}[0-9][a-z]{0,3}".prop_map(|a| {
                Piece { prose: vec![], other: vec![0..a.len()], text: a }
            }),
        ]
    }

    /// Pieces joined with `sep` between them, after `prefix`.
    fn join(prefix: &str, pieces: Vec<Piece>, sep: &str, suffix: &str) -> Piece {
        let mut out = Piece {
            text: prefix.to_owned(),
            prose: vec![],
            other: vec![],
        };
        for (i, piece) in pieces.into_iter().enumerate() {
            if i > 0 {
                out.text.push_str(sep);
            }
            let at = out.text.len();
            out.prose
                .extend(piece.prose.into_iter().map(|r| r.start + at..r.end + at));
            out.other
                .extend(piece.other.into_iter().map(|r| r.start + at..r.end + at));
            out.text.push_str(&piece.text);
        }
        out.text.push_str(suffix);
        out
    }

    /// Blocks: paragraphs, headings, quotes and list items of inline
    /// content, and code blocks, HTML blocks and display math.
    fn block() -> impl Strategy<Value = Piece> {
        let line = || {
            proptest::collection::vec(inline(), 1..6)
                // A line can't start with a tag, which would be a heading.
                .prop_map(|mut p| {
                    if p[0].text.starts_with('#') {
                        p.insert(
                            0,
                            Piece {
                                text: "so".into(),
                                prose: vec![0..2],
                                other: vec![],
                            },
                        );
                    }
                    p
                })
        };
        prop_oneof![
            3 => line().prop_map(|p| join("", p, " ", "")),
            1 => line().prop_map(|p| join("## ", p, " ", "")),
            1 => line().prop_map(|p| join("> ", p, " ", "")),
            1 => line().prop_map(|p| join("- ", p, " ", "")),
            1 => code_block(),
        ]
    }

    /// A block with no prose in it.
    fn code_block() -> impl Strategy<Value = Piece> {
        (word(), word(), 0..3usize).prop_map(|(a, b, kind)| {
            let text = match kind {
                0 => format!("```\n{a} {b}\n```"),
                1 => format!("<div>\n{a} {b}\n</div>"),
                _ => format!("$$\n{a} {b}\n$$"),
            };
            Piece {
                prose: vec![],
                other: vec![0..text.len()],
                text,
            }
        })
    }

    /// A note: maybe front matter, then blocks apart.
    fn note() -> impl Strategy<Value = Piece> {
        (
            any::<bool>(),
            word(),
            proptest::collection::vec(block(), 1..8),
        )
            .prop_map(|(front, key, blocks)| {
                let mut note = join("", blocks, "\n\n", "\n");
                if front {
                    let head = format!("---\n{key}: {key}\n---\n\n");
                    let n = head.len();
                    note.text.insert_str(0, &head);
                    note.prose = note
                        .prose
                        .into_iter()
                        .map(|r| r.start + n..r.end + n)
                        .collect();
                    note.other = note
                        .other
                        .into_iter()
                        .map(|r| r.start + n..r.end + n)
                        .collect();
                    note.other.insert(0, 0..n);
                }
                note
            })
    }

    fn overlaps(a: &Range<usize>, b: &Range<usize>) -> bool {
        a.start < b.end && b.start < a.end
    }

    proptest! {
        #![proptest_config(config())]

        /// Oracle: the words found are exactly the prose words the note
        /// was built from, so none is in code, a link target, a wiki link,
        /// a tag, an address, math, HTML or front matter.
        #[test]
        fn prop_finds_exactly_the_prose(note in note()) {
            let found = prose_words(&note.text);
            prop_assert_eq!(&found, &note.prose, "{:?}", note.text);
            for range in &found {
                prop_assert!(note.other.iter().all(|o| !overlaps(range, o)));
            }
        }

        /// Metamorphic: a block of code put in between two blocks moves the
        /// words after it along and changes nothing else.
        #[test]
        fn prop_code_elsewhere_changes_nothing(
            note in note(),
            code in code_block(),
            at in any::<prop::sample::Index>(),
        ) {
            let breaks: Vec<usize> = note.text.match_indices("\n\n").map(|(i, _)| i + 2).collect();
            prop_assume!(!breaks.is_empty());
            let cut = breaks[at.index(breaks.len())];
            let inserted = format!("{}\n\n", code.text);
            let mut text = note.text.clone();
            text.insert_str(cut, &inserted);
            let shifted: Vec<_> = prose_words(&note.text)
                .into_iter()
                .map(|r| if r.start >= cut { r.start + inserted.len()..r.end + inserted.len() } else { r })
                .collect();
            prop_assert_eq!(prose_words(&text), shifted);
        }

        /// Any text at all: the words are sorted, apart, on character
        /// boundaries, made of letters (and apostrophes inside), at least
        /// two long, with no capital past the first, and none in the code
        /// `kanso_core` finds (which doesn't know front matter, so a fence
        /// in it is code there and not here).
        #[test]
        fn prop_words_are_words(text in "([a-zA-Z0-9 \n#*_`\\[\\]()<>!$:/.@'’&\\\\é-]|```\n|---\n){0,80}") {
            let found = prose_words(&text);
            let code = if text.starts_with("---\n") {
                Vec::new()
            } else {
                kanso_core::links::code_ranges(&text)
            };
            for pair in found.windows(2) {
                prop_assert!(pair[0].end <= pair[1].start);
            }
            for range in &found {
                prop_assert!(text.is_char_boundary(range.start) && text.is_char_boundary(range.end));
                let word = &text[range.clone()];
                prop_assert!(is_word(word), "{:?}", word);
                prop_assert!(word.chars().all(|c| c.is_alphabetic() || c == '\'' || c == '’'));
                prop_assert!(!word.ends_with(['\'', '’']));
                prop_assert!(code.iter().all(|c| !overlaps(range, c)), "{:?} in {:?}", word, text);
            }
        }

    }

    proptest! {
        #![proptest_config(config())]

        /// A window keeps the words that start in it, in order.
        #[test]
        fn prop_window_is_a_filter(note in note(), a in 0..400usize, b in 0..400usize) {
            let all = prose_words(&note.text);
            let window = a.min(b)..a.max(b);
            let expected: Vec<_> = all.iter().filter(|w| window.contains(&w.start)).cloned().collect();
            prop_assert_eq!(in_window(&all, &window), expected);
        }

        /// The span of lines is where those lines are.
        #[test]
        fn prop_line_span_matches_lines(text in "[ab\n]{0,30}", a in 0..12usize, b in 0..12usize) {
            let lines = a.min(b)..a.max(b);
            let span = line_span(&text, &lines);
            let expected: String = text
                .split_inclusive('\n')
                .skip(lines.start)
                .take(lines.len())
                .collect();
            prop_assert_eq!(&text[span], expected.as_str());
        }
    }
}
