//! Highlighting one block of a note: parse it, collect what the queries
//! capture, and flatten the captures into [`Span`]s.
//!
//! This is the desktop's highlighter (the component library's
//! `SyntaxHighlighter`) without its window types and its clocks. Where it
//! differs, on purpose:
//!
//! - The result depends on the block's text alone, so it can be cached by
//!   content ([`super::Highlighter`]) and the same text always gives the
//!   same spans: no timeouts and no cap on the number of injected parses
//!   per note (the byte and range limits per injection remain, which are
//!   deterministic).
//! - A fence names its language with [`super::grammar`], which knows more
//!   spellings than the library's registry.
//! - Injected code is highlighted with its language's highlight query alone,
//!   as the library does, without injections of its own.

use std::collections::HashMap;
use std::ops::Range;

use tree_sitter::{Parser, QueryCursor, StreamingIterator, Tree};

use super::registry::{self, Grammar};
use super::{Span, Token, grammar};

/// Injected ranges beyond these are not highlighted (the library's limits).
const MAX_INJECTION_RANGES: usize = 4096;
const MAX_INJECTION_BYTES: usize = 512 * 1024;
/// A fence's language name is a word; a longer capture names nothing.
const MAX_INJECTION_LANGUAGE_BYTES: usize = 64;

/// What a block that doesn't start the note is parsed after, so the parser
/// reads it as it would in the note: front matter and the rule-text-rule
/// reading of it exist only at the very start, and the paragraph keeps a
/// block that begins with `---` from being either.
const PREAMBLE: &str = "x\n\n";

/// One parser per grammar, reused.
#[derive(Default)]
pub(super) struct Parsers(HashMap<&'static str, Parser>);

impl Parsers {
    /// `text` parsed with `grammar`.
    fn parse(&mut self, grammar: &'static Grammar, text: &str) -> Option<Tree> {
        let parser = match self.0.entry(grammar.name) {
            std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let mut parser = Parser::new();
                parser.set_language(&grammar.language).ok()?;
                entry.insert(parser)
            }
        };
        parser.parse(text, None)
    }
}

/// A range of the text and the token it is drawn as, `None` when no theme
/// key colors the capture (it still counts for the rule below).
struct Item {
    range: Range<usize>,
    token: Option<Token>,
}

/// An injected language's parse of a piece of the block, which is parsed as
/// a text of its own: tree-sitter's recovery from syntax errors depends on
/// where in a text the code sits, so a layer parsed in place would color
/// broken code differently with other text before it, and a block's colors
/// would not be a function of the block.
struct Layer<'a> {
    grammar: &'static Grammar,
    tree: Tree,
    /// Where the piece starts in the block's text.
    start: usize,
    text: &'a str,
}

/// The text a block is parsed as: after the preamble when it doesn't begin
/// the note, and with a last line ending.
fn source(block: &str, at_start: bool) -> String {
    let preamble = if at_start { "" } else { PREAMBLE };
    // A note's last line gets its line ending, which the grammar wants after
    // a heading marker, and which makes the end of the note read the same
    // whatever comes before it.
    let ending = if block.ends_with('\n') { "" } else { "\n" };
    format!("{preamble}{block}{ending}")
}

/// Whether the grammar parses `block` without a syntax error. After an
/// error the parser's recovery can reach into the text that follows (a
/// broken table has swallowed the rest of a note), so the blocks after it
/// are not independent of it. Only the block grammar is run: far less than
/// highlighting.
pub(super) fn is_clean(parsers: &mut Parsers, block: &str, at_start: bool) -> bool {
    registry::get("markdown")
        .and_then(|markdown| parsers.parse(markdown, &source(block, at_start)))
        .is_some_and(|tree| !tree.root_node().has_error())
}

/// The spans of `block`, relative to its start. `at_start` says whether the
/// block begins the note.
pub(super) fn block_spans(parsers: &mut Parsers, block: &str, at_start: bool) -> Vec<Span> {
    spans_within(parsers, block, at_start, None)
}

/// The spans of `block` that lie in `window` (relative to its start), or all
/// of them without one: the block is parsed whole, but only what touches the
/// window is captured and, for injected code and inline text, parsed. Spans
/// outside the window may be missing or cut short; inside it they are those
/// of [`block_spans`]. For a block that is most of a note (the parse that
/// follows a syntax error), of which a window is wanted.
pub(super) fn spans_within(
    parsers: &mut Parsers,
    block: &str,
    at_start: bool,
    window: Option<Range<usize>>,
) -> Vec<Span> {
    let Some(markdown) = registry::get("markdown") else {
        return Vec::new();
    };
    let preamble = if at_start { "" } else { PREAMBLE };
    let source = source(block, at_start);
    let Some(tree) = parsers.parse(markdown, &source) else {
        return Vec::new();
    };

    // The window in the parsed text's offsets.
    let window = window.map(|window| window.start + preamble.len()..window.end + preamble.len());
    let mut items = host_items(markdown, &tree, &source, window.as_ref());
    items.extend(injected_items(
        parsers,
        markdown,
        &tree,
        &source,
        preamble.len(),
        window.as_ref(),
    ));

    // Back to the block's own offsets, on its character boundaries.
    let shift = preamble.len();
    let items: Vec<(Range<usize>, Token)> = items
        .into_iter()
        .filter(|item| item.range.start >= shift)
        .filter_map(|item| {
            let token = item.token?;
            let mut start = item.range.start - shift;
            let mut end = (item.range.end - shift).min(block.len());
            while !block.is_char_boundary(start) {
                start -= 1;
            }
            while !block.is_char_boundary(end) {
                end += 1;
            }
            (start < end).then_some((start..end, token))
        })
        .collect();
    flatten(&items)
}

/// The block grammar's own captures, in the order the queries yield them.
/// A capture over the same range as the one before it takes the one before's
/// name: the library's rule, which its grammars' queries rely on (a capture
/// meant to be overridden says so with the same range).
fn host_items(
    markdown: &'static Grammar,
    tree: &Tree,
    source: &str,
    window: Option<&Range<usize>>,
) -> Vec<Item> {
    let Some(highlights) = markdown.highlights() else {
        return Vec::new();
    };
    let mut items: Vec<Item> = Vec::new();
    let mut cursor = QueryCursor::new();
    if let Some(window) = window {
        cursor.set_byte_range(window.clone());
    }
    let mut matches = cursor.matches(&highlights.query, tree.root_node(), source.as_bytes());
    while let Some(found) = matches.next() {
        for capture in found.captures {
            let range = capture.node.byte_range();
            let token = match items.last() {
                Some(last) if last.range == range => last.token,
                _ => highlights.tokens[capture.index as usize],
            };
            items.push(Item { range, token });
        }
    }
    items
}

/// The captures of every injected layer, after the host's: injected code is
/// more specific than the Markdown around it, so its colors win.
fn injected_items(
    parsers: &mut Parsers,
    markdown: &'static Grammar,
    tree: &Tree,
    source: &str,
    shift: usize,
    window: Option<&Range<usize>>,
) -> Vec<Item> {
    let mut layers = layers(parsers, markdown, tree, source, window);
    layers.retain(|layer| layer.start >= shift);
    layers.sort_by_key(|layer| layer.start);

    let mut items = Vec::new();
    for layer in &layers {
        let Some(highlights) = layer.grammar.highlights() else {
            continue;
        };
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(
            &highlights.query,
            layer.tree.root_node(),
            layer.text.as_bytes(),
        );
        // Within a layer, a capture that starts inside the one before it is
        // dropped, unless its pattern allows overlap.
        let mut last_end = 0usize;
        while let Some(found) = matches.next() {
            let overlap = highlights
                .query
                .property_settings(found.pattern_index)
                .iter()
                .any(|property| property.key.as_ref() == "highlight.allow-overlap");
            for capture in found.captures {
                let range =
                    capture.node.start_byte() + layer.start..capture.node.end_byte() + layer.start;
                if !overlap && range.start < last_end {
                    continue;
                }
                if !overlap {
                    last_end = range.end;
                }
                items.push(Item {
                    token: highlights.tokens[capture.index as usize],
                    range,
                });
            }
        }
    }
    items
}

/// The layers the host's injection query asks for: a language per fence, the
/// inline text of every paragraph, HTML blocks and front matter. With a
/// `window`, only the layers that touch it (each whole, so that what is
/// captured in it is what it is without a window).
fn layers<'a>(
    parsers: &mut Parsers,
    markdown: &'static Grammar,
    tree: &Tree,
    source: &'a str,
    window: Option<&Range<usize>>,
) -> Vec<Layer<'a>> {
    let Some(injections) = markdown.injections() else {
        return Vec::new();
    };
    let mut layers = Vec::new();
    let mut cursor = QueryCursor::new();
    if let Some(window) = window {
        cursor.set_byte_range(window.clone());
    }
    let mut matches = cursor.matches(&injections.query, tree.root_node(), source.as_bytes());
    while let Some(found) = matches.next() {
        // The language is set by the pattern or named by the text.
        let mut named = None;
        for property in injections.query.property_settings(found.pattern_index) {
            if property.key.as_ref() == "injection.language" {
                named = property.value.as_deref().map(registry::get);
            }
        }
        let language = match named {
            Some(language) => language,
            None => found
                .captures
                .iter()
                .find(|capture| Some(capture.index) == injections.language)
                .and_then(|capture| {
                    let range = capture.node.byte_range();
                    (range.len() <= MAX_INJECTION_LANGUAGE_BYTES)
                        .then(|| source.get(range))
                        .flatten()
                })
                .and_then(|info| grammar(info.trim()))
                .and_then(registry::get),
        };
        let Some(language) = language else {
            continue;
        };
        let mut ranges: Vec<tree_sitter::Range> = found
            .captures
            .iter()
            .filter(|capture| Some(capture.index) == injections.content)
            .map(|capture| capture.node.range())
            .collect();
        // Plain prose has nothing for the inline grammar to find.
        if language.name == "markdown_inline" {
            ranges
                .retain(|range| has_trigger(&source.as_bytes()[range.start_byte..range.end_byte]));
        }
        if ranges.is_empty() {
            continue;
        }
        ranges.sort_unstable_by_key(|range| (range.start_byte, range.end_byte));
        let bytes: usize = ranges.iter().map(|r| r.end_byte - r.start_byte).sum();
        if ranges.len() > MAX_INJECTION_RANGES || bytes > MAX_INJECTION_BYTES {
            continue;
        }
        // Every pattern captures one piece of text.
        let [range] = ranges.as_slice() else {
            continue;
        };
        if window
            .is_some_and(|window| range.end_byte <= window.start || range.start_byte >= window.end)
        {
            continue;
        }
        let Some(text) = source.get(range.start_byte..range.end_byte) else {
            continue;
        };
        if let Some(tree) = parsers.parse(language, text) {
            layers.push(Layer {
                grammar: language,
                tree,
                start: range.start_byte,
                text,
            });
        }
    }
    layers
}

/// Whether inline text holds a byte that could start an inline construct
/// (emphasis, code spans, links, images, autolinks). A superset of what the
/// inline query captures.
fn has_trigger(bytes: &[u8]) -> bool {
    bytes.iter().any(|byte| {
        matches!(
            byte,
            b'*' | b'_' | b'`' | b'[' | b']' | b'(' | b')' | b'<' | b'>' | b'!' | b'~' | b'$'
        )
    })
}

/// Turns captures into spans: where captures overlap, the text is split into
/// the pieces they share, each piece carrying the tokens of every capture
/// over it, the earlier-starting (and for equal starts, earlier-captured)
/// first, so a later token wins. The library's `unique_styles`, on tokens
/// instead of styles; neighbors with equal tokens are merged.
pub(super) fn flatten(items: &[(Range<usize>, Token)]) -> Vec<Span> {
    if items.is_empty() {
        return Vec::new();
    }
    // (position, is start, item): ends sort before starts at one position,
    // and the sort is stable, so equal starts stay in capture order.
    let mut events: Vec<(usize, bool, usize)> = Vec::with_capacity(items.len() * 2);
    for (index, (range, _)) in items.iter().enumerate() {
        events.push((range.start, true, index));
        events.push((range.end, false, index));
    }
    events.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

    let mut spans: Vec<Span> = Vec::new();
    let mut active: Vec<usize> = Vec::new();
    let mut last = events[0].0;
    for (position, start, index) in events {
        if position > last && !active.is_empty() {
            let tokens: Vec<Token> = active.iter().map(|i| items[*i].1).collect();
            match spans.last_mut() {
                Some(previous) if previous.range.end == last && previous.tokens == tokens => {
                    previous.range.end = position;
                }
                _ => spans.push(Span {
                    range: last..position,
                    tokens,
                }),
            }
        }
        if start {
            active.push(index);
        } else {
            active.retain(|i| *i != index);
        }
        last = position;
    }
    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(range: Range<usize>, tokens: &[Token]) -> Span {
        Span {
            range,
            tokens: tokens.to_vec(),
        }
    }

    #[test]
    fn flattening_splits_overlaps_and_merges_neighbors() {
        use Token::*;
        // AA / BBB / CCCCC / DD, as the library's doc comment draws it.
        let items = [
            (0..2, Keyword),
            (2..5, String),
            (3..8, Comment),
            (5..7, Number),
        ];
        assert_eq!(
            flatten(&items),
            [
                s(0..2, &[Keyword]),
                s(2..3, &[String]),
                s(3..5, &[String, Comment]),
                s(5..7, &[Comment, Number]),
                s(7..8, &[Comment]),
            ]
        );
        // Equal neighbors merge; gaps stay gaps.
        assert_eq!(
            flatten(&[(0..2, Keyword), (2..4, Keyword), (6..7, Keyword)]),
            [s(0..4, &[Keyword]), s(6..7, &[Keyword])]
        );
        assert_eq!(flatten(&[]), []);
    }
}
