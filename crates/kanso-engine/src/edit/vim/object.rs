//! Text objects: `iw aw iW aW`, quotes `i" a" i' a' i` a``, brackets
//! `i( a( ib ab i[ a[ i{ a{ iB aB i< a<`, and paragraphs `ip ap`. Each one
//! is a range around the cursor, taken by an operator or selected in
//! Visual mode.

use std::ops::Range;

use super::motion::matching;
use super::text::*;

/// A text object's range, and whether it is whole lines.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub range: Range<usize>,
    pub linewise: bool,
}

/// The object `key` names (`w`, `"`, `(`, `p` and so on), inner or around,
/// at `at`; `None` when there isn't one there.
pub fn object(text: &str, at: usize, key: char, inner: bool, count: usize) -> Option<Span> {
    let at = floor_grapheme(text, at);
    let charwise = |range: Range<usize>| Span {
        range,
        linewise: false,
    };
    match key {
        'w' => word(text, at, false, inner).map(charwise),
        'W' => word(text, at, true, inner).map(charwise),
        '"' | '\'' | '`' => quote(text, at, key, inner).map(charwise),
        '(' | ')' | 'b' => brackets(text, at, '(', ')', inner, count).map(charwise),
        '[' | ']' => brackets(text, at, '[', ']', inner, count).map(charwise),
        '{' | '}' | 'B' => brackets(text, at, '{', '}', inner, count).map(charwise),
        '<' | '>' => brackets(text, at, '<', '>', inner, count).map(charwise),
        'p' => paragraph(text, at, inner),
        _ => None,
    }
}

/// The run of one class the cursor is on, within its line: a word, a run
/// of punctuation, or a run of blanks. `aw` adds the blanks after it, or
/// those before when there are none after.
fn word(text: &str, at: usize, big: bool, inner: bool) -> Option<Range<usize>> {
    let start_of_line = line_start(text, at);
    let end_of_line = line_end(text, at);
    if start_of_line == end_of_line {
        return None;
    }
    let at = clamp_normal(text, at);
    let class = class_at(text, at, big);
    let run_start = |mut pos: usize, class: u8| {
        while let Some(prev) = prev_in_line(text, pos) {
            if class_at(text, prev, big) != class {
                break;
            }
            pos = prev;
        }
        pos
    };
    let run_end = |mut pos: usize, class: u8| {
        while pos < end_of_line && class_at(text, pos, big) == class {
            pos = next_in_line(text, pos).unwrap_or(end_of_line);
        }
        pos
    };
    let start = run_start(at, class);
    let end = run_end(at, class);
    if inner {
        return Some(start..end);
    }
    if class == 0 {
        // On blanks: the blanks and the word after them.
        let after = class_at(text, end, big);
        let end = if end < end_of_line {
            run_end(end, after)
        } else {
            end
        };
        return Some(start..end);
    }
    let trailing = run_end(end, 0);
    if trailing > end {
        return Some(start..trailing);
    }
    // No blanks after: take the ones before, keeping any indent.
    let leading = run_start(start, 0);
    if leading < start && leading > start_of_line {
        return Some(leading..end);
    }
    Some(start..end)
}

/// The quoted text on the cursor's line that holds the cursor, or else the
/// next one after it. Quotes pair from the start of the line, and a quote
/// after a backslash doesn't count. `a"` takes the quotes and the blanks
/// after them (or before, when there are none after).
fn quote(text: &str, at: usize, q: char, inner: bool) -> Option<Range<usize>> {
    let start = line_start(text, at);
    let end = line_end(text, at);
    let line = &text[start..end];
    let mut quotes = Vec::new();
    let mut escaped = false;
    for (i, c) in line.char_indices() {
        if escaped {
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == q {
            quotes.push(start + i);
        }
    }
    let pair = quotes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&[open, close]| (open, close))
        .find(|&(open, close)| open <= at && at <= close)
        .or_else(|| {
            quotes
                .as_chunks::<2>()
                .0
                .iter()
                .map(|&[open, close]| (open, close))
                .find(|&(open, _)| open > at)
        })?;
    let (open, close) = pair;
    let qlen = q.len_utf8();
    if inner {
        return Some(open + qlen..close);
    }
    let after = close + qlen;
    let blanks_after =
        text[after..end].len() - text[after..end].trim_start_matches([' ', '\t']).len();
    if blanks_after > 0 {
        return Some(open..after + blanks_after);
    }
    let blanks_before =
        text[start..open].len() - text[start..open].trim_end_matches([' ', '\t']).len();
    let leading = open - blanks_before;
    if leading > start {
        Some(leading..after)
    } else {
        Some(open..after)
    }
}

/// The brackets around the cursor (the `count`th pair out), or the one
/// the cursor is on. Inside a block whose brackets sit on lines of their
/// own, the inner object is the lines between them.
fn brackets(
    text: &str,
    at: usize,
    open: char,
    close: char,
    inner: bool,
    count: usize,
) -> Option<Range<usize>> {
    let mut opener = enclosing_open(text, at, open, close)?;
    // A count looks further out, a pair at a time.
    for _ in 1..count.max(1) {
        opener = unmatched_before(text, opener, open, close)?;
    }
    let closer = matching(text, opener, open, close, true)?;
    if !inner {
        return Some(opener..closer + close.len_utf8());
    }
    let mut start = opener + open.len_utf8();
    let mut end = closer;
    // `{` at a line's end: the inner text starts on the next line.
    if text[start..].starts_with('\n') && start < end {
        start += 1;
    }
    // `}` alone on its line: the inner text ends before that line.
    let close_line = line_start(text, end);
    if close_line > start && text[close_line..end].chars().all(|c| c == ' ' || c == '\t') {
        end = close_line;
    }
    Some(start..end.max(start))
}

/// The unmatched `open` before (or at) `at`, or the one matching a
/// closer at `at`.
fn enclosing_open(text: &str, at: usize, open: char, close: char) -> Option<usize> {
    let at = floor_char(text, at);
    if text[at..].starts_with(open) {
        return Some(at);
    }
    if text[at..].starts_with(close) {
        return matching(text, at, open, close, false);
    }
    unmatched_before(text, at, open, close)
}

/// The nearest `open` before `at` that no `close` between closes.
fn unmatched_before(text: &str, at: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0usize;
    for (i, c) in text[..at].char_indices().rev() {
        if c == close {
            depth += 1;
        } else if c == open {
            if depth == 0 {
                return Some(i);
            }
            depth -= 1;
        }
    }
    None
}

/// The paragraph the cursor's line is in, as whole lines: the run of lines
/// with text, or the run of empty ones. `ap` adds the empty lines after it,
/// or those before when there are none after.
fn paragraph(text: &str, at: usize, inner: bool) -> Option<Span> {
    let line = line_start(text, at);
    let empty = is_blank_line(text, line);
    let same = |start: usize| is_blank_line(text, start) == empty;
    let mut first = line;
    while let Some(prev) = prev_line(text, first) {
        if !same(prev) {
            break;
        }
        first = prev;
    }
    let mut last = line;
    while let Some(next) = next_line(text, last) {
        if !same(next) {
            break;
        }
        last = next;
    }
    if !inner {
        let mut after = last;
        while let Some(next) = next_line(text, after) {
            if is_blank_line(text, next) == empty {
                break;
            }
            after = next;
        }
        if after != last {
            last = after;
        } else {
            while let Some(prev) = prev_line(text, first) {
                if is_blank_line(text, prev) == empty {
                    break;
                }
                first = prev;
            }
        }
    }
    Some(Span {
        range: first..line_end(text, last),
        linewise: true,
    })
}

/// Whether the line starting at `start` has nothing but blanks.
fn is_blank_line(text: &str, start: usize) -> bool {
    text[start..line_end(text, start)]
        .chars()
        .all(|c| c == ' ' || c == '\t')
}
