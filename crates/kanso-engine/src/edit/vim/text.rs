//! The text underneath Vim's commands: lines, characters as the reader sees
//! them (grapheme clusters, so an emoji or an accented letter is one
//! step), the classes words are made of, and Vim's own way of stepping
//! through a buffer, where each line's end is a place of its own.
//!
//! Offsets are UTF-8 byte offsets into the whole text, as the editor's
//! selection is. Every function here takes any offset up to the text's
//! length and answers with one on a grapheme boundary; none of them panics.

use unicode_segmentation::GraphemeCursor;

/// The start of the line `pos` is on.
pub fn line_start(text: &str, pos: usize) -> usize {
    let pos = floor_char(text, pos);
    text[..pos].rfind('\n').map_or(0, |i| i + 1)
}

/// The end of the line `pos` is on: its newline, or the end of the text.
pub fn line_end(text: &str, pos: usize) -> usize {
    let pos = floor_char(text, pos);
    text[pos..].find('\n').map_or(text.len(), |i| pos + i)
}

/// `pos`, or the char boundary before it, within the text.
pub fn floor_char(text: &str, pos: usize) -> usize {
    let mut pos = pos.min(text.len());
    while !text.is_char_boundary(pos) {
        pos -= 1;
    }
    pos
}

/// Whether `pos` starts a line.
pub fn is_line_start(text: &str, pos: usize) -> bool {
    pos == 0 || text.as_bytes().get(pos - 1) == Some(&b'\n')
}

/// Whether `pos` is the start of an empty line (which the end of a text
/// that ends in a newline is too).
pub fn is_empty_line(text: &str, pos: usize) -> bool {
    is_line_start(text, pos) && (pos == text.len() || text.as_bytes()[pos] == b'\n')
}

/// Whether `pos` is at a line's end: on its newline, or the end of text.
pub fn at_line_end(text: &str, pos: usize) -> bool {
    pos >= text.len() || text.as_bytes()[pos] == b'\n'
}

/// The grapheme boundary at or before `pos`, counted within its line, so
/// the newline between two lines is never part of a cluster.
pub fn floor_grapheme(text: &str, pos: usize) -> usize {
    let pos = floor_char(text, pos);
    let start = line_start(text, pos);
    let end = line_end(text, pos);
    let line = &text[start..end];
    let local = pos - start;
    let mut cursor = GraphemeCursor::new(local, line.len(), true);
    if cursor.is_boundary(line, 0).unwrap_or(true) {
        return pos;
    }
    match cursor.prev_boundary(line, 0) {
        Ok(Some(b)) => start + b,
        _ => start,
    }
}

/// The end of the grapheme that starts at `pos`, within its line, or
/// `None` at the line's end.
pub fn next_in_line(text: &str, pos: usize) -> Option<usize> {
    let start = line_start(text, pos);
    let end = line_end(text, pos);
    if pos >= end {
        return None;
    }
    let line = &text[start..end];
    let mut cursor = GraphemeCursor::new(pos - start, line.len(), true);
    match cursor.next_boundary(line, 0) {
        Ok(Some(b)) => Some(start + b),
        _ => Some(end),
    }
}

/// The start of the grapheme before `pos` on its line, or `None` at the
/// line's start.
pub fn prev_in_line(text: &str, pos: usize) -> Option<usize> {
    let start = line_start(text, pos);
    if pos <= start {
        return None;
    }
    let end = line_end(text, pos);
    let line = &text[start..end];
    let mut cursor = GraphemeCursor::new(pos - start, line.len(), true);
    match cursor.prev_boundary(line, 0) {
        Ok(Some(b)) => Some(start + b),
        _ => Some(start),
    }
}

/// How long the grapheme at `pos` is; 0 at a line's end.
pub fn grapheme_len(text: &str, pos: usize) -> usize {
    next_in_line(text, pos).map_or(0, |next| next - pos)
}

/// The start of the line's last grapheme, or the line's start when it is
/// empty: where Normal mode's cursor rests at the end of a line.
pub fn last_in_line(text: &str, pos: usize) -> usize {
    let end = line_end(text, pos);
    prev_in_line(text, end).unwrap_or_else(|| line_start(text, pos))
}

/// Where Normal mode's cursor may be for `pos`: on a grapheme, never past
/// a line's last one, and on an empty line at its start.
pub fn clamp_normal(text: &str, pos: usize) -> usize {
    let pos = floor_grapheme(text, pos);
    if at_line_end(text, pos) {
        last_in_line(text, pos)
    } else {
        pos
    }
}

/// How many graphemes come before `pos` on its line: its column.
pub fn column(text: &str, pos: usize) -> usize {
    let start = line_start(text, pos);
    let mut col = 0;
    let mut at = start;
    while at < pos {
        match next_in_line(text, at) {
            Some(next) => {
                at = next;
                col += 1;
            }
            None => break,
        }
    }
    col
}

/// Column `col` of the line starting at `start`, as far as the line goes:
/// its last grapheme in Normal mode, or its end with `past_end`.
pub fn at_column(text: &str, start: usize, col: usize, past_end: bool) -> usize {
    let mut at = start;
    for _ in 0..col {
        match next_in_line(text, at) {
            Some(next) if past_end || !at_line_end(text, next) => at = next,
            _ => break,
        }
    }
    at
}

/// The start of the line after the one `pos` is on.
pub fn next_line(text: &str, pos: usize) -> Option<usize> {
    let end = line_end(text, pos);
    (end < text.len()).then_some(end + 1)
}

/// The start of the line before the one `pos` is on.
pub fn prev_line(text: &str, pos: usize) -> Option<usize> {
    let start = line_start(text, pos);
    (start > 0).then(|| line_start(text, start - 1))
}

/// The start of line `n` (0-based), or of the last line.
pub fn nth_line(text: &str, n: usize) -> usize {
    let mut start = 0;
    for _ in 0..n {
        match next_line(text, start) {
            Some(next) => start = next,
            None => break,
        }
    }
    start
}

/// The first character on `pos`'s line that isn't a blank; the line's
/// last grapheme when it is all blanks.
pub fn first_non_blank(text: &str, pos: usize) -> usize {
    let start = line_start(text, pos);
    let end = line_end(text, pos);
    let indent = text[start..end]
        .find(|c: char| c != ' ' && c != '\t')
        .map_or(end, |i| start + i);
    if indent == end {
        last_in_line(text, start)
    } else {
        indent
    }
}

/// The blanks a line starts with.
pub fn indent_of(text: &str, pos: usize) -> &str {
    let start = line_start(text, pos);
    let end = line_end(text, pos);
    let line = &text[start..end];
    let len = line.len() - line.trim_start_matches([' ', '\t']).len();
    &line[..len]
}

/// Whether nothing but blanks come before `pos` on its line.
pub fn in_indent(text: &str, pos: usize) -> bool {
    let start = line_start(text, pos);
    text[start..pos].chars().all(|c| c == ' ' || c == '\t')
}

/// What a character is to Vim's word motions: a blank, punctuation, or a
/// word character. A WORD only tells blanks from the rest.
pub fn class_of(c: char, big: bool) -> u8 {
    if c.is_whitespace() {
        0
    } else if big || c.is_alphanumeric() || c == '_' {
        2
    } else {
        1
    }
}

/// The class at `pos`; a line's end is a blank.
pub fn class_at(text: &str, pos: usize, big: bool) -> u8 {
    text.get(pos..)
        .and_then(|rest| rest.chars().next())
        .map_or(0, |c| class_of(c, big))
}

/// One step forward the way Vim takes it, where a non-empty line's end is
/// a place of its own: `(next, 0)` within a line, `(next, 2)` onto a line's
/// end, `(next, 1)` onto the next line, and `None` at the end of the text.
pub fn inc(text: &str, pos: usize) -> Option<(usize, u8)> {
    if pos >= text.len() {
        return None;
    }
    if text.as_bytes()[pos] == b'\n' {
        return Some((pos + 1, 1));
    }
    let next = next_in_line(text, pos).unwrap_or(pos + 1);
    Some((next, if at_line_end(text, next) { 2 } else { 0 }))
}

/// One step back: `(prev, 0)` within a line, `(prev, 1)` onto the end of
/// the line before, `None` at the start of the text.
pub fn dec(text: &str, pos: usize) -> Option<(usize, u8)> {
    if pos == 0 {
        return None;
    }
    if is_line_start(text, pos) {
        return Some((pos - 1, 1));
    }
    Some((prev_in_line(text, pos).unwrap_or(pos - 1), 0))
}

/// The one replacement that turns `old` into `new`: the range of `old`
/// that differs, and what goes there. `None` when they are equal.
pub fn diff(old: &str, new: &str) -> Option<(std::ops::Range<usize>, String)> {
    if old == new {
        return None;
    }
    let (a, b) = (old.as_bytes(), new.as_bytes());
    let mut prefix = a.iter().zip(b).take_while(|(x, y)| x == y).count();
    while !(old.is_char_boundary(prefix) && new.is_char_boundary(prefix)) {
        prefix -= 1;
    }
    let max_suffix = (a.len() - prefix).min(b.len() - prefix);
    let mut suffix = a
        .iter()
        .rev()
        .zip(b.iter().rev())
        .take(max_suffix)
        .take_while(|(x, y)| x == y)
        .count();
    while !(old.is_char_boundary(a.len() - suffix) && new.is_char_boundary(b.len() - suffix)) {
        suffix -= 1;
    }
    Some((
        prefix..a.len() - suffix,
        new[prefix..b.len() - suffix].to_owned(),
    ))
}
