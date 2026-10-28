//! Colors for Markdown source and for the code in fenced blocks, without a
//! window: the text and a byte range in, [`Span`]s out, each naming the
//! [`Token`]s a theme colors it with.
//!
//! The desktop editor highlights with its component library's
//! tree-sitter highlighter, which is built on the library's window types.
//! This is a highlighter of the engine's own on the same tree-sitter runtime
//! and grammar crates, with the same queries (patched the same way, see
//! [`queries`]), for the mobile app. Its output is checked against the
//! desktop's by a test in `kanso-app` (`markdown_source`), which lists
//! where the two differ on purpose: the spellings a fence may use for a
//! language, and the limits the desktop puts on time and on the number of
//! injected parses, which would make a result depend on the clock and on the
//! rest of the note.
//!
//! [`spans`] computes from scratch. A [`Highlighter`] returns the same
//! spans, quicker: it cuts the note into blocks (`blocks`) that highlight
//! on their own, and remembers each block's spans by the block's text, so
//! typing in one paragraph parses that paragraph only.
//!
//! Offsets are UTF-8 byte offsets; an editor that counts in UTF-16 converts
//! with [`crate::text`].

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::Range;
use std::sync::Arc;

use crate::theme::file::CAPTURES;

mod block;
mod blocks;
mod grammars;
pub mod queries;
mod registry;

pub use grammars::{GRAMMARS, grammar};

/// What a span is drawn as: a key of a theme's `syntax` section (every key
/// but `text`, which is what no span covers).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Token {
    Heading,
    BlockMarker,
    Marker,
    ListMarker,
    Task,
    Hr,
    Quote,
    Link,
    Url,
    Code,
    CodeBlock,
    Emphasis,
    Strong,
    FrontMatter,
    Html,
    Escape,
    Comment,
    CommentDoc,
    Keyword,
    String,
    StringSpecial,
    Function,
    Type,
    TypeConstructor,
    Number,
    Boolean,
    Constant,
    Variable,
    VariableSpecial,
    Property,
    Attribute,
    Tag,
    Label,
    Operator,
    Punctuation,
    Bracket,
    Embedded,
}

impl Token {
    /// Every token, in the order of the theme format's `syntax` keys.
    pub const ALL: [Token; 37] = [
        Token::Heading,
        Token::BlockMarker,
        Token::Marker,
        Token::ListMarker,
        Token::Task,
        Token::Hr,
        Token::Quote,
        Token::Link,
        Token::Url,
        Token::Code,
        Token::CodeBlock,
        Token::Emphasis,
        Token::Strong,
        Token::FrontMatter,
        Token::Html,
        Token::Escape,
        Token::Comment,
        Token::CommentDoc,
        Token::Keyword,
        Token::String,
        Token::StringSpecial,
        Token::Function,
        Token::Type,
        Token::TypeConstructor,
        Token::Number,
        Token::Boolean,
        Token::Constant,
        Token::Variable,
        Token::VariableSpecial,
        Token::Property,
        Token::Attribute,
        Token::Tag,
        Token::Label,
        Token::Operator,
        Token::Punctuation,
        Token::Bracket,
        Token::Embedded,
    ];

    /// The theme `syntax` key that colors this token.
    pub fn key(self) -> &'static str {
        match self {
            Token::Heading => "heading",
            Token::BlockMarker => "block_marker",
            Token::Marker => "marker",
            Token::ListMarker => "list_marker",
            Token::Task => "task",
            Token::Hr => "hr",
            Token::Quote => "quote",
            Token::Link => "link",
            Token::Url => "url",
            Token::Code => "code",
            Token::CodeBlock => "code_block",
            Token::Emphasis => "emphasis",
            Token::Strong => "strong",
            Token::FrontMatter => "front_matter",
            Token::Html => "html",
            Token::Escape => "escape",
            Token::Comment => "comment",
            Token::CommentDoc => "comment_doc",
            Token::Keyword => "keyword",
            Token::String => "string",
            Token::StringSpecial => "string_special",
            Token::Function => "function",
            Token::Type => "type",
            Token::TypeConstructor => "type_constructor",
            Token::Number => "number",
            Token::Boolean => "boolean",
            Token::Constant => "constant",
            Token::Variable => "variable",
            Token::VariableSpecial => "variable_special",
            Token::Property => "property",
            Token::Attribute => "attribute",
            Token::Tag => "tag",
            Token::Label => "label",
            Token::Operator => "operator",
            Token::Punctuation => "punctuation",
            Token::Bracket => "bracket",
            Token::Embedded => "embedded",
        }
    }

    /// The token for a theme `syntax` key.
    pub fn from_key(key: &str) -> Option<Token> {
        Token::ALL.into_iter().find(|token| token.key() == key)
    }

    /// The token a highlight capture is drawn as: the one
    /// [`CAPTURES`] names for it, and for a capture it doesn't name, the one
    /// for the part before its first dot (`keyword.modifier` is a keyword),
    /// as the component library falls back. `None` for a capture no theme key
    /// colors.
    pub fn from_capture(name: &str) -> Option<Token> {
        let exact = |name: &str| {
            CAPTURES
                .iter()
                .find(|(capture, _)| *capture == name)
                .and_then(|(_, key)| Token::from_key(key))
        };
        exact(name).or_else(|| name.split_once('.').and_then(|(prefix, _)| exact(prefix)))
    }
}

/// Text drawn with `tokens`, outermost first: a later token's color, slant
/// or weight wins over an earlier one's (see `theme::style`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub range: Range<usize>,
    pub tokens: Vec<Token>,
}

/// The spans of `text` within `range`, computed from scratch: sorted, not
/// overlapping, never empty, with at least one token each, and on character
/// boundaries, in the note's byte offsets. The text between spans is plain.
/// `range` is clamped to `text` and widened to character boundaries.
pub fn spans(text: &str, range: Range<usize>) -> Vec<Span> {
    thread_local! {
        static PARSERS: std::cell::RefCell<Uncached> = std::cell::RefCell::default();
    }
    PARSERS.with(|parsers| assemble(text, range, &mut *parsers.borrow_mut(), true))
}

/// [`spans`] with the parse that follows a syntax error carried to the end
/// of the note, as it was before it was cut short: what [`spans`] must give
/// in `range`.
#[cfg(test)]
pub(crate) fn spans_uncapped(text: &str, range: Range<usize>) -> Vec<Span> {
    assemble(text, range, &mut Uncached::default(), false)
}

/// Where the blocks of a note get their spans from.
trait Blocks {
    /// Whether the block parses without a syntax error.
    fn clean(&mut self, block: &str, at_start: bool) -> bool;
    /// The block's spans, relative to its start.
    fn spans(&mut self, block: &str, at_start: bool) -> Arc<[Span]>;
    /// The block's spans that lie in `window` (relative to its start; the
    /// rest may be missing), for a block too big to be worth keeping whole:
    /// not cached.
    fn spans_within(&mut self, block: &str, at_start: bool, window: Range<usize>) -> Vec<Span>;
}

/// Every block parsed afresh.
#[derive(Default)]
struct Uncached(block::Parsers);

impl Blocks for Uncached {
    fn clean(&mut self, block: &str, at_start: bool) -> bool {
        block::is_clean(&mut self.0, block, at_start)
    }

    fn spans(&mut self, block: &str, at_start: bool) -> Arc<[Span]> {
        Arc::from(block::block_spans(&mut self.0, block, at_start))
    }

    fn spans_within(&mut self, block: &str, at_start: bool, window: Range<usize>) -> Vec<Span> {
        block::spans_within(&mut self.0, block, at_start, Some(window))
    }
}

/// `range` clamped to `text` and widened to character boundaries.
fn normalize(text: &str, range: Range<usize>) -> Range<usize> {
    let mut end = range.end.min(text.len());
    let mut start = range.start.min(end);
    while !text.is_char_boundary(start) {
        start -= 1;
    }
    while !text.is_char_boundary(end) {
        end += 1;
    }
    start..end
}

/// The spans of the blocks that overlap `range`, as `blocks` gives them,
/// clipped to `range` and joined: the one place the spans of a range are
/// made, so every way of getting them agrees.
///
/// Blocks are independent of each other, so each is highlighted alone, until
/// one has a syntax error: the grammar's recovery from it can reach into what
/// follows, so that block and what follows it are highlighted as one parse.
/// For a range, that parse is made less of than it was:
///
/// - it stops at the end of the block that holds the end of `range` and of
///   the block after that (the text beyond is not read at all), and
/// - when `range` is less than half of that parse, only what touches it is
///   captured and parsed again (inline text, fenced code), though the block
///   grammar parses all of it; a bigger part is highlighted whole and kept
///   in the cache, as the whole note's would be.
///
/// So a note with an error in it costs a range the text from the error to
/// the range's end, parsed once for the grammar and no more, and not the
/// rest of the note. The spans inside `range` are those of the parse to the
/// end of the note (tested, `spans_uncapped`). Nothing is kept of the small
/// parts (what follows an error changes with every key typed after it).
///
/// The blocks before the range are checked for an error, which is far less
/// work than highlighting them.
fn assemble(text: &str, range: Range<usize>, blocks: &mut impl Blocks, capped: bool) -> Vec<Span> {
    let range = normalize(text, range);
    let mut out: Vec<Span> = Vec::new();
    if range.is_empty() {
        return out;
    }
    let add = |out: &mut Vec<Span>, base: usize, spans: &[Span]| {
        for span in spans {
            let start = (span.range.start + base).max(range.start);
            let end = (span.range.end + base).min(range.end);
            if start >= end {
                continue;
            }
            match out.last_mut() {
                Some(last) if last.range.end == start && last.tokens == span.tokens => {
                    last.range.end = end;
                }
                _ => out.push(Span {
                    range: start..end,
                    tokens: span.tokens.clone(),
                }),
            }
        }
    };
    let cuts = self::blocks::cut(text);
    for (index, cut) in cuts.iter().enumerate() {
        if cut.start >= range.end {
            break;
        }
        let at_start = cut.start == 0;
        if !blocks.clean(&text[cut.clone()], at_start) {
            let limit = if capped {
                reach(&cuts, index, range.end, text.len())
            } else {
                text.len()
            };
            let tail = &text[cut.start..limit];
            let wanted = range.start.max(cut.start) - cut.start..range.end - cut.start;
            // A small part of a long parse is read for the range alone; one
            // that is most of it is kept whole, to be read again.
            let spans = if capped && wanted.len() * 2 <= tail.len() {
                blocks.spans_within(tail, at_start, wanted)
            } else {
                blocks.spans(tail, at_start).to_vec()
            };
            add(&mut out, cut.start, &spans);
            break;
        }
        if cut.end > range.start {
            let spans = blocks.spans(&text[cut.clone()], at_start);
            add(&mut out, cut.start, &spans);
        }
    }
    out
}

/// Where the parse that starts at the broken block `cuts[from]` stops: the
/// end of the block that holds `end` and of the one after it.
fn reach(cuts: &[Range<usize>], from: usize, end: usize, len: usize) -> usize {
    let holding = cuts[from..]
        .iter()
        .position(|cut| cut.end >= end)
        .map_or(cuts.len() - 1, |found| from + found);
    cuts.get(holding + 1).map_or(len, |next| next.end)
}

/// The whole note parsed as one block, with no cutting: what the block
/// cutter must agree with.
#[cfg(test)]
pub(crate) fn whole_document_spans(text: &str) -> Vec<Span> {
    let mut parsers = block::Parsers::default();
    block::block_spans(&mut parsers, text, true)
}

/// What the cache may hold, in bytes of text and spans.
const CACHE_BYTES: usize = 2 * 1024 * 1024;

/// What a block's spans are cached under: its text, and whether it starts
/// the note (which is all the rest of the note changes about them).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    hash: u64,
    len: usize,
    at_start: bool,
}

struct Entry {
    /// Checked on every hit, so a collision of hashes never returns the
    /// spans of other text.
    text: Box<str>,
    clean: Option<bool>,
    spans: Option<Arc<[Span]>>,
    used: u64,
}

/// [`spans`] with a memory: each block of the note is highlighted once, by
/// its text, until it falls out of a cache of about 2 MB. `spans` returns
/// exactly what the free function does for any state of the cache, because a
/// block's spans are a function of its text and whether it starts the note,
/// which is all the key holds, and the text is compared on a hit.
pub struct Highlighter {
    parsers: block::Parsers,
    cache: HashMap<Key, Entry>,
    capacity: usize,
    bytes: usize,
    clock: u64,
    parsed: usize,
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::with_capacity(CACHE_BYTES)
    }
}

impl Highlighter {
    pub fn new() -> Self {
        Self::default()
    }

    /// A highlighter that keeps about `bytes` of blocks (text and spans)
    /// between calls.
    pub fn with_capacity(bytes: usize) -> Self {
        Highlighter {
            parsers: block::Parsers::default(),
            cache: HashMap::new(),
            capacity: bytes,
            bytes: 0,
            clock: 0,
            parsed: 0,
        }
    }

    /// The spans of `text` within `range`; `highlight::spans(text, range)`
    /// exactly.
    pub fn spans(&mut self, text: &str, range: Range<usize>) -> Vec<Span> {
        let out = assemble(text, range, self, true);
        self.evict();
        out
    }

    /// How many blocks have been highlighted (not found in the cache) since
    /// this was made.
    pub fn blocks_parsed(&self) -> usize {
        self.parsed
    }

    /// The entry for a block's text, made if there is none.
    fn entry(&mut self, text: &str, at_start: bool) -> &mut Entry {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let key = Key {
            hash: hasher.finish(),
            len: text.len(),
            at_start,
        };
        self.clock += 1;
        let clock = self.clock;
        let held = self
            .cache
            .get(&key)
            .is_some_and(|entry| &*entry.text == text);
        if !held {
            let entry = Entry {
                text: text.into(),
                clean: None,
                spans: None,
                used: clock,
            };
            self.bytes += entry.size();
            if let Some(old) = self.cache.insert(key, entry) {
                self.bytes -= old.size();
            }
        }
        let entry = self.cache.get_mut(&key).expect("just inserted");
        entry.used = clock;
        entry
    }

    /// Drops the least recently used blocks until the cache fits.
    fn evict(&mut self) {
        while self.bytes > self.capacity {
            let Some(oldest) = self
                .cache
                .iter()
                .min_by_key(|(_, entry)| entry.used)
                .map(|(key, _)| *key)
            else {
                return;
            };
            if let Some(entry) = self.cache.remove(&oldest) {
                self.bytes -= entry.size();
            }
        }
    }
}

impl Blocks for Highlighter {
    fn clean(&mut self, block: &str, at_start: bool) -> bool {
        let entry = self.entry(block, at_start);
        if let Some(clean) = entry.clean {
            return clean;
        }
        let clean = block::is_clean(&mut self.parsers, block, at_start);
        let entry = self.entry(block, at_start);
        entry.clean = Some(clean);
        clean
    }

    fn spans(&mut self, block: &str, at_start: bool) -> Arc<[Span]> {
        if let Some(spans) = self.entry(block, at_start).spans.clone() {
            return spans;
        }
        self.parsed += 1;
        let spans: Arc<[Span]> = Arc::from(block::block_spans(&mut self.parsers, block, at_start));
        let entry = self.entry(block, at_start);
        entry.spans = Some(spans.clone());
        let size = entry.size();
        // The entry grew from the text alone to text and spans.
        self.bytes = self.bytes.saturating_add(size.saturating_sub(block.len()));
        spans
    }

    fn spans_within(&mut self, block: &str, at_start: bool, window: Range<usize>) -> Vec<Span> {
        self.parsed += 1;
        block::spans_within(&mut self.parsers, block, at_start, Some(window))
    }
}

impl Entry {
    fn size(&self) -> usize {
        self.text.len() + self.spans.as_ref().map_or(0, |spans| spans.len() * 64)
    }
}

#[cfg(test)]
mod tests;
