//! The formatting toolbar's edits, as plain text in, one replacement out.
//!
//! Line formats (headings, lists, tasks, quotes) toggle a marker at the
//! start of each selected line and leave every other line alone; inline
//! formats wrap or unwrap the selection; templates (links, images, tables, a
//! rule) go in at the cursor with the cursor inside them, where the typing
//! goes next. Offsets are UTF-8 byte offsets, as the editor's selection is.

use std::ops::Range;

use super::Edit;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// A heading of level 1 to 6 on the cursor's line.
    Heading(u8),
    Bulleted,
    Numbered,
    Task,
    Quote,
    /// Inline code, or a fenced block when the selection spans lines.
    Code,
    /// A fenced block around the selected lines (or an empty one at the
    /// cursor), taken away again when they are already fenced.
    CodeBlock,
    Bold,
    Italic,
    Strikethrough,
    Link,
    Image,
    Table,
    Rule,
}

/// The first header cell of [`TABLE`], selected after inserting it.
const TABLE: &str = "| Column | Column |\n| --- | --- |\n|  |  |";
const TABLE_FIRST_CELL: Range<usize> = 2..8;
const RULE: &str = "---";
/// What a new link or image points at until it's typed over.
const URL_PLACEHOLDER: &str = "https://";

/// The edit that applies `format` to `text` with `selection` selected.
pub fn format(format: Format, text: &str, selection: Range<usize>) -> Edit {
    let selection = clamp(text, selection);
    match format {
        Format::Heading(level) => heading(text, selection, level.clamp(1, 6)),
        Format::Bulleted => list(text, selection, ListKind::Bulleted),
        Format::Numbered => list(text, selection, ListKind::Numbered),
        Format::Task => list(text, selection, ListKind::Task),
        Format::Quote => quote(text, selection),
        Format::Code => code(text, selection),
        Format::CodeBlock => code_block(text, selection),
        Format::Bold => wrap(text, selection, "**"),
        Format::Italic => wrap(text, selection, "_"),
        Format::Strikethrough => wrap(text, selection, "~~"),
        Format::Link => link(text, selection, ""),
        Format::Image => link(text, selection, "!"),
        Format::Table => insert_block(text, selection, TABLE, TABLE_FIRST_CELL),
        Format::Rule => insert_block(text, selection, RULE, RULE.len()..RULE.len()),
    }
}

/// An ordered selection inside the text, on character boundaries.
fn clamp(text: &str, selection: Range<usize>) -> Range<usize> {
    let floor = |offset: usize| {
        let mut offset = offset.min(text.len());
        while !text.is_char_boundary(offset) {
            offset -= 1;
        }
        offset
    };
    let (a, b) = (floor(selection.start), floor(selection.end));
    a.min(b)..a.max(b)
}

/// The start of the line `at` is on.
pub fn line_start(text: &str, at: usize) -> usize {
    text[..at].rfind('\n').map_or(0, |ix| ix + 1)
}

/// The end of the line `at` is on, before its newline.
pub fn line_end(text: &str, at: usize) -> usize {
    text[at..].find('\n').map_or(text.len(), |ix| at + ix)
}

/// From the start of the selection's first line to the end of its last. A
/// selection that ends at the very start of a line leaves that line out, as
/// when whole lines are selected by dragging down.
pub fn selected_lines(text: &str, selection: &Range<usize>) -> Range<usize> {
    let start = line_start(text, selection.start);
    let last = if selection.end > selection.start && text[..selection.end].ends_with('\n') {
        selection.end - 1
    } else {
        selection.end
    };
    start..line_end(text, last)
}

/// A marker replaced at the start of one line: `old_len` bytes at `at`
/// become `new`.
struct LineChange {
    at: usize,
    old_len: usize,
    new: String,
}

/// Rewrites the lines in `block` with one optional change each, keeping the
/// selection on the same text: a position inside a replaced marker moves to
/// the end of the new one.
fn change_lines(
    text: &str,
    block: Range<usize>,
    changes: Vec<Option<LineChange>>,
    selection: Range<usize>,
) -> Edit {
    let mut out = String::with_capacity(block.len() + 16);
    // For each line: where it started, where it starts now, and its change.
    let mut starts = Vec::with_capacity(changes.len());
    let mut old_start = block.start;
    for (line, change) in text[block.clone()].split('\n').zip(&changes) {
        if old_start > block.start {
            out.push('\n');
        }
        starts.push((old_start, out.len()));
        match change {
            Some(change) => {
                out.push_str(&line[..change.at]);
                out.push_str(&change.new);
                out.push_str(&line[change.at + change.old_len..]);
            }
            None => out.push_str(line),
        }
        old_start += line.len() + 1;
    }
    let delta = out.len() as isize - block.len() as isize;
    let map = |offset: usize| -> usize {
        if offset < block.start {
            return offset;
        }
        if offset > block.end {
            return offset.saturating_add_signed(delta);
        }
        let ix = starts
            .iter()
            .rposition(|(start, _)| *start <= offset)
            .unwrap_or(0);
        let (old_line, new_line) = starts[ix];
        let rel = offset - old_line;
        let rel = match &changes[ix] {
            Some(change) if rel >= change.at + change.old_len => {
                rel - change.old_len + change.new.len()
            }
            Some(change) if rel >= change.at => change.at + change.new.len(),
            _ => rel,
        };
        block.start + new_line + rel
    };
    let selection = map(selection.start)..map(selection.end);
    Edit {
        range: block,
        text: out,
        selection,
    }
}

/// The heading level of a line, and the length of its marker with the
/// space after it.
pub fn heading_marker(line: &str) -> Option<(u8, usize)> {
    let hashes = line.bytes().take_while(|&b| b == b'#').count();
    match line.as_bytes().get(hashes) {
        _ if !(1..=6).contains(&hashes) => None,
        None => Some((hashes as u8, hashes)),
        Some(b' ') => Some((hashes as u8, hashes + 1)),
        Some(_) => None,
    }
}

/// Makes the cursor's line a heading of `level`, or plain again when it
/// already is one.
fn heading(text: &str, selection: Range<usize>, level: u8) -> Edit {
    let block = selected_lines(text, &(selection.start..selection.start));
    let line = &text[block.clone()];
    let current = heading_marker(line);
    let new = if current.map(|(l, _)| l) == Some(level) {
        String::new()
    } else {
        format!("{} ", "#".repeat(level.into()))
    };
    let change = LineChange {
        at: 0,
        old_len: current.map_or(0, |(_, len)| len),
        new,
    };
    change_lines(text, block, vec![Some(change)], selection)
}

/// The heading button pressed again and again on the cursor's line: no
/// heading becomes H1, H1 becomes H2, H2 becomes H3, and H3 (or deeper) is
/// plain again.
pub fn cycle_heading(text: &str, selection: Range<usize>) -> Edit {
    let selection = clamp(text, selection);
    let block = selected_lines(text, &(selection.start..selection.start));
    let current = heading_marker(&text[block]).map(|(level, _)| level);
    let level = match current {
        None => 1,
        Some(1) => 2,
        Some(2) => 3,
        // Taking it off is applying the level it has.
        Some(level) => level,
    };
    heading(text, selection, level)
}

/// `[[` and `]]` around the selection (or nothing), with the caret before
/// the closing brackets, where a title is typed or finished.
pub fn wiki_brackets(text: &str, selection: Range<usize>) -> Edit {
    let selection = clamp(text, selection);
    let selected = &text[selection.clone()];
    let caret = selection.start + 2 + selected.len();
    Edit {
        range: selection,
        text: format!("[[{selected}]]"),
        selection: caret..caret,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListKind {
    Bulleted,
    Numbered,
    Task,
}

pub fn indent_len(line: &str) -> usize {
    line.len() - line.trim_start_matches([' ', '\t']).len()
}

/// The list marker at the start of `rest` (a line after its indent): its
/// kind and its length with the space after it.
pub fn list_marker(rest: &str) -> Option<(ListKind, usize)> {
    let b = rest.as_bytes();
    if matches!(b.first(), Some(b'-' | b'*' | b'+')) && b.get(1) == Some(&b' ') {
        let task = b.len() >= 5
            && b[2] == b'['
            && matches!(b[3], b' ' | b'x' | b'X')
            && b[4] == b']'
            && b.get(5).is_none_or(|&c| c == b' ');
        return Some(if task {
            (ListKind::Task, b.len().min(6))
        } else {
            (ListKind::Bulleted, 2)
        });
    }
    let digits = b.iter().take_while(|c| c.is_ascii_digit()).count();
    if (1..=9).contains(&digits)
        && matches!(b.get(digits), Some(b'.' | b')'))
        && b.get(digits + 1) == Some(&b' ')
    {
        return Some((ListKind::Numbered, digits + 2));
    }
    None
}

/// Which of `lines` a line format acts on: a lone line always, and in a
/// longer selection every line with something on it.
fn targets(lines: &[&str]) -> Vec<bool> {
    if lines.len() == 1 {
        vec![true]
    } else {
        lines.iter().map(|line| !line.trim().is_empty()).collect()
    }
}

/// Turns the selected lines into a list of `kind`, numbered from 1, or back
/// into plain lines when they all already are one. Lines in another kind of
/// list switch to this one; lines already in it keep their marker (a done
/// task stays done); indentation stays.
fn list(text: &str, selection: Range<usize>, kind: ListKind) -> Edit {
    let block = selected_lines(text, &selection);
    let lines: Vec<&str> = text[block.clone()].split('\n').collect();
    let targets = targets(&lines);
    let markers: Vec<Option<(ListKind, usize)>> = lines
        .iter()
        .map(|line| list_marker(&line[indent_len(line)..]))
        .collect();
    let is_kind = |ix: usize| markers[ix].is_some_and(|(k, _)| k == kind);
    let all = (0..lines.len()).filter(|&ix| targets[ix]).all(is_kind);
    let mut number = 0;
    let changes = (0..lines.len())
        .map(|ix| {
            if !targets[ix] {
                return None;
            }
            number += 1;
            let new = match kind {
                _ if all => String::new(),
                _ if is_kind(ix) => return None,
                ListKind::Bulleted => "- ".to_owned(),
                ListKind::Numbered => format!("{number}. "),
                ListKind::Task => "- [ ] ".to_owned(),
            };
            Some(LineChange {
                at: indent_len(lines[ix]),
                old_len: markers[ix].map_or(0, |(_, len)| len),
                new,
            })
        })
        .collect();
    change_lines(text, block, changes, selection)
}

/// The length of a line's quote marker, `>` and the space after it.
fn quote_marker(line: &str) -> usize {
    match line.as_bytes() {
        [b'>', b' ', ..] => 2,
        [b'>', ..] => 1,
        _ => 0,
    }
}

/// Quotes the selected lines, or unquotes them when they all are quoted.
fn quote(text: &str, selection: Range<usize>) -> Edit {
    let block = selected_lines(text, &selection);
    let lines: Vec<&str> = text[block.clone()].split('\n').collect();
    let targets = targets(&lines);
    let all = (0..lines.len())
        .filter(|&ix| targets[ix])
        .all(|ix| quote_marker(lines[ix]) > 0);
    let changes = (0..lines.len())
        .map(|ix| {
            let marker = quote_marker(lines[ix]);
            match (targets[ix], all) {
                (false, _) => None,
                (true, true) => Some(LineChange {
                    at: 0,
                    old_len: marker,
                    new: String::new(),
                }),
                // Already quoted lines stay as they are.
                (true, false) if marker > 0 => None,
                (true, false) => Some(LineChange {
                    at: 0,
                    old_len: 0,
                    new: "> ".into(),
                }),
            }
        })
        .collect();
    change_lines(text, block, changes, selection)
}

/// Wraps the selection in `marker` on both sides, selecting what's inside,
/// or takes the markers off when they are already there, just outside or
/// just inside the selection. With nothing selected the cursor lands
/// between a new pair.
fn wrap(text: &str, selection: Range<usize>, marker: &str) -> Edit {
    let (start, end) = (selection.start, selection.end);
    let m = marker.len();
    if text[..start].ends_with(marker) && text[end..].starts_with(marker) {
        return Edit {
            range: start - m..end + m,
            text: text[selection.clone()].to_owned(),
            selection: start - m..end - m,
        };
    }
    let selected = &text[selection.clone()];
    if selected.len() >= 2 * m && selected.starts_with(marker) && selected.ends_with(marker) {
        return Edit {
            range: selection,
            text: selected[m..selected.len() - m].to_owned(),
            selection: start..end - 2 * m,
        };
    }
    Edit {
        range: selection,
        text: format!("{marker}{selected}{marker}"),
        selection: start + m..end + m,
    }
}

/// Whether a line opens or closes a fenced code block.
fn is_fence(line: &str) -> bool {
    line.trim_start().starts_with("```")
}

/// The fence lines right above and below `block`, when there are both.
fn fences_around(text: &str, block: &Range<usize>) -> Option<(Range<usize>, Range<usize>)> {
    if block.start == 0 || block.end >= text.len() {
        return None;
    }
    let before = line_start(text, block.start - 1)..block.start - 1;
    let after = block.end + 1..line_end(text, block.end + 1);
    (is_fence(&text[before.clone()]) && is_fence(&text[after.clone()])).then_some((before, after))
}

/// Code: a fenced block around selected lines, taken away again when
/// exactly the lines inside fences are selected, and inline code otherwise.
fn code(text: &str, selection: Range<usize>) -> Edit {
    // Exactly whole lines, the last one included even when it is empty.
    let whole = line_start(text, selection.start)..line_end(text, selection.end);
    if selection == whole
        && let Some((before, after)) = fences_around(text, &whole)
    {
        let inner = text[whole].to_owned();
        return Edit {
            range: before.start..after.end,
            selection: before.start..before.start + inner.len(),
            text: inner,
        };
    }
    if !text[selection.clone()].contains('\n') {
        return wrap(text, selection, "`");
    }
    let block = selected_lines(text, &selection);
    let inner = &text[block.clone()];
    Edit {
        range: block.clone(),
        text: format!("```\n{inner}\n```"),
        selection: block.start + 4..block.start + 4 + inner.len(),
    }
}

/// A fenced block around the lines the selection touches, or the fences
/// taken off when those lines are inside them or are themselves a fenced
/// block. On an empty line the block starts empty, with the cursor in it.
fn code_block(text: &str, selection: Range<usize>) -> Edit {
    let block = selected_lines(text, &selection);
    let unfence = |before: Range<usize>, after: Range<usize>| {
        let inner = text[before.end + 1..after.start - 1].to_owned();
        Edit {
            range: before.start..after.end,
            selection: before.start..before.start + inner.len(),
            text: inner,
        }
    };
    // A selection that ends at a line's start leaves that line out, unless
    // the line is the last one inside the fences (an empty line).
    let whole = line_start(text, selection.start)..line_end(text, selection.end);
    for lines in [&block, &whole] {
        if let Some((before, after)) = fences_around(text, lines) {
            return unfence(before, after);
        }
    }
    // The fences themselves are selected, with something between them.
    let lines: Vec<&str> = text[block.clone()].split('\n').collect();
    if lines.len() >= 3 && is_fence(lines[0]) && is_fence(lines[lines.len() - 1]) {
        let first_end = block.start + lines[0].len();
        let last_start = block.end - lines[lines.len() - 1].len();
        return unfence(block.start..first_end, last_start..block.end);
    }
    let inner = &text[block.clone()];
    Edit {
        range: block.clone(),
        text: format!("```\n{inner}\n```"),
        selection: block.start + 4..block.start + 4 + inner.len(),
    }
}

/// Whether the selection is a web address, which becomes a link's target
/// rather than its text.
fn looks_like_url(text: &str) -> bool {
    (text.starts_with("https://") || text.starts_with("http://"))
        && !text.contains(char::is_whitespace)
}

/// A link (or with `bang` "!", an image). Selected text becomes its label and
/// the placeholder address is selected; a selected address becomes its
/// target and the cursor waits in the empty label.
fn link(text: &str, selection: Range<usize>, bang: &str) -> Edit {
    let selected = &text[selection.clone()];
    let (label, url) = if looks_like_url(selected) {
        ("", selected)
    } else {
        (selected, URL_PLACEHOLDER)
    };
    let inserted = format!("{bang}[{label}]({url})");
    let label_at = selection.start + bang.len() + 1;
    let new_selection = if label.is_empty() {
        label_at..label_at
    } else {
        let url_at = label_at + label.len() + 2;
        url_at..url_at + url.len()
    };
    Edit {
        range: selection,
        text: inserted,
        selection: new_selection,
    }
}

/// Puts `block` on lines of its own at the cursor: in place of a blank
/// line, or after the cursor's line with a blank line between. `select` is
/// what to select inside it.
fn insert_block(text: &str, selection: Range<usize>, block: &str, select: Range<usize>) -> Edit {
    let line = selected_lines(text, &(selection.end..selection.end));
    let (range, prefix) = if text[line.clone()].trim().is_empty() {
        (line, "")
    } else {
        (line.end..line.end, "\n\n")
    };
    // Keep a blank line before any text that follows.
    let next =
        (range.end < text.len()).then(|| &text[range.end + 1..line_end(text, range.end + 1)]);
    let suffix = if next.is_some_and(|next| !next.trim().is_empty()) {
        "\n"
    } else {
        ""
    };
    let at = range.start + prefix.len();
    Edit {
        range,
        text: format!("{prefix}{block}{suffix}"),
        selection: at + select.start..at + select.end,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    const ALL: [Format; 19] = [
        Format::Heading(1),
        Format::Heading(2),
        Format::Heading(3),
        Format::Heading(4),
        Format::Heading(5),
        Format::Heading(6),
        Format::Bulleted,
        Format::Numbered,
        Format::Task,
        Format::Quote,
        Format::Code,
        Format::CodeBlock,
        Format::Bold,
        Format::Italic,
        Format::Strikethrough,
        Format::Link,
        Format::Image,
        Format::Table,
        Format::Rule,
    ];

    const LINE_FORMATS: [Format; 10] = [
        Format::Heading(1),
        Format::Heading(2),
        Format::Heading(3),
        Format::Heading(4),
        Format::Heading(5),
        Format::Heading(6),
        Format::Bulleted,
        Format::Numbered,
        Format::Task,
        Format::Quote,
    ];

    /// Markdown with markers of every kind, blank lines and multibyte text.
    fn markdown() -> impl Strategy<Value = String> {
        prop::collection::vec(
            "( {0,2})(#{0,3} |- |1\\. |- \\[[ x]\\] |> |```|)[a-zé*_`~\\[\\]() ]{0,10}",
            1..8,
        )
        .prop_map(|lines| lines.join("\n"))
    }

    /// Plain lines: no marker of any kind at their start.
    fn plain() -> impl Strategy<Value = String> {
        prop::collection::vec("( {0,2})([a-zé][a-zé ]{0,8})?", 1..8)
            .prop_map(|lines| lines.join("\n"))
    }

    /// A text and a selection in it, given as two fractions of its length.
    fn with_selection(
        text: impl Strategy<Value = String>,
    ) -> impl Strategy<Value = (String, Range<usize>)> {
        (text, 0f64..=1., 0f64..=1.).prop_map(|(text, a, b)| {
            let at = |f: f64| (text.len() as f64 * f) as usize;
            let selection = clamp(&text, at(a)..at(b));
            (text, selection)
        })
    }

    /// Plain lines of letters, accents, CJK and emoji (one to four bytes a
    /// character), some empty, so edges fall at line starts and ends.
    fn unicode() -> impl Strategy<Value = String> {
        prop::collection::vec("[a-zé日本😀 ]{0,8}", 1..6).prop_map(|lines| lines.join("\n"))
    }

    /// A text and a selection in it, anywhere on a character boundary or at
    /// a line's start or end.
    fn with_edge_selection(
        text: impl Strategy<Value = String>,
    ) -> impl Strategy<Value = (String, Range<usize>)> {
        (
            text,
            any::<prop::sample::Index>(),
            any::<prop::sample::Index>(),
        )
            .prop_map(|(text, a, b)| {
                let mut offsets: Vec<usize> = text
                    .char_indices()
                    .map(|(ix, _)| ix)
                    .chain([text.len()])
                    .collect();
                offsets.sort_unstable();
                let (a, b) = (
                    offsets[a.index(offsets.len())],
                    offsets[b.index(offsets.len())],
                );
                (text, a.min(b)..a.max(b))
            })
    }

    fn lines(text: &str) -> Vec<&str> {
        text.split('\n').collect()
    }

    proptest! {
        #![proptest_config(config())]

        /// Validity: every edit replaces a range of the old text on character
        /// boundaries and selects a range of the new one on character
        /// boundaries. Nothing beyond the lines next to the selection changes
        /// (a fence just above or below may go).
        #[test]
        fn prop_edits_are_valid_and_local((text, selection) in with_selection(markdown()), which in 0..ALL.len()) {
            let edit = format(ALL[which], &text, selection.clone());
            prop_assert!(edit.range.start <= edit.range.end && edit.range.end <= text.len());
            prop_assert!(text.is_char_boundary(edit.range.start) && text.is_char_boundary(edit.range.end));
            let new = edit.apply(&text);
            prop_assert!(edit.selection.start <= edit.selection.end && edit.selection.end <= new.len());
            prop_assert!(new.is_char_boundary(edit.selection.start) && new.is_char_boundary(edit.selection.end));
            let first = line_start(&text, line_start(&text, selection.start).saturating_sub(1));
            let last_end = line_end(&text, selection.end);
            let last = if last_end < text.len() { line_end(&text, last_end + 1) } else { last_end };
            prop_assert!(new.starts_with(&text[..first]), "{:?} {:?}", text, new);
            prop_assert!(new.ends_with(&text[last..]), "{:?} {:?}", text, new);
        }

        /// Locality for line formats: the text keeps its lines, the ones
        /// outside the selection are untouched, and each changed line differs
        /// only at its start (its text after the markers is the same).
        #[test]
        fn prop_line_formats_change_only_the_selected_lines((text, selection) in with_selection(markdown()), which in 0..LINE_FORMATS.len()) {
            let format_ = LINE_FORMATS[which];
            let edit = format(format_, &text, selection.clone());
            let new = edit.apply(&text);
            let (old_lines, new_lines) = (lines(&text), lines(&new));
            prop_assert_eq!(old_lines.len(), new_lines.len());
            let at_line = |offset: usize| text[..offset].matches('\n').count();
            let block = selected_lines(&text, &selection);
            let (first, last) = match format_ {
                Format::Heading(_) => (at_line(selection.start), at_line(selection.start)),
                _ => (at_line(block.start), at_line(block.end)),
            };
            for (ix, (old, new)) in old_lines.iter().zip(&new_lines).enumerate() {
                if ix < first || ix > last {
                    prop_assert_eq!(old, new, "line {}", ix);
                } else {
                    // What differs is a run of markup at the start; the
                    // rest of the line is the same text.
                    let common: usize = old.chars().rev().zip(new.chars().rev())
                        .take_while(|(a, b)| a == b).map(|(c, _)| c.len_utf8()).sum();
                    let markup = |s: &str| s.chars().all(|c| " \t#>-*+[]xX.)0123456789".contains(c));
                    prop_assert!(markup(&old[..old.len() - common]) && markup(&new[..new.len() - common]),
                        "line {}: {:?} -> {:?}", ix, old, new);
                }
            }
        }

        /// All or nothing: a list or quote format leaves every line it aims
        /// at (each non-blank selected line, or a lone line) with its
        /// marker; or, when they all had it before, takes exactly that
        /// marker off each one. Lines keep their place: line formats never
        /// add or remove lines.
        #[test]
        fn prop_block_formats_are_all_or_nothing((text, selection) in with_selection(markdown()), which in 0usize..4) {
            let (format_, kind) = [
                (Format::Bulleted, Some(ListKind::Bulleted)),
                (Format::Numbered, Some(ListKind::Numbered)),
                (Format::Task, Some(ListKind::Task)),
                (Format::Quote, None),
            ][which];
            // Where a line's marker of this format starts, and its length.
            let marker = |line: &str| -> Option<(usize, usize)> {
                match kind {
                    None => Some((0, quote_marker(line))).filter(|&(_, len)| len > 0),
                    Some(kind) => {
                        let at = indent_len(line);
                        list_marker(&line[at..])
                            .filter(|&(k, _)| k == kind)
                            .map(|(_, len)| (at, len))
                    }
                }
            };
            let block = selected_lines(&text, &selection);
            let old: Vec<&str> = text[block.clone()].split('\n').collect();
            let aimed = targets(&old);
            let first = text[..block.start].matches('\n').count();
            let changed = format(format_, &text, selection).apply(&text);
            let new = lines(&changed);
            let all_before = (0..old.len()).filter(|&ix| aimed[ix]).all(|ix| marker(old[ix]).is_some());
            for ix in (0..old.len()).filter(|&ix| aimed[ix]) {
                let after = new[first + ix];
                if all_before {
                    let (at, len) = marker(old[ix]).unwrap();
                    let unmarked = format!("{}{}", &old[ix][..at], &old[ix][at + len..]);
                    prop_assert_eq!(after, unmarked.as_str());
                } else {
                    // A bullet whose text starts with a box is a task, and a
                    // task is a bullet too.
                    let task_bullet = kind == Some(ListKind::Bulleted)
                        && list_marker(&after[indent_len(after)..]).is_some_and(|(k, _)| k == ListKind::Task);
                    prop_assert!(marker(after).is_some() || task_bullet, "{:?} -> {:?}", old[ix], after);
                }
            }
        }

        /// Toggling: on lines without markers, a line format applied twice
        /// (the second time to the selection the first left) gives back the
        /// text and the selection.
        #[test]
        fn prop_line_formats_toggle_back((text, selection) in with_selection(plain()), which in 0..LINE_FORMATS.len()) {
            let once = format(LINE_FORMATS[which], &text, selection.clone());
            let changed = once.apply(&text);
            let twice = format(LINE_FORMATS[which], &changed, once.selection.clone());
            prop_assert_eq!(twice.apply(&changed), text);
            prop_assert_eq!(twice.selection, selection);
        }

        /// Toggling: wrapping a selection in a marker and doing it again
        /// gives back the text and the selection, on one line or several
        /// (a fenced block).
        #[test]
        fn prop_wraps_toggle_back((text, selection) in with_selection(plain()), which in 0usize..4) {
            let format_ = [Format::Code, Format::Bold, Format::Italic, Format::Strikethrough][which];
            let once = format(format_, &text, selection.clone());
            let changed = once.apply(&text);
            let twice = format(format_, &changed, once.selection.clone());
            prop_assert_eq!(twice.apply(&changed), text.clone());
            // The selection stays on the same text, except that code across
            // lines takes in those whole lines, inside the fences.
            if format_ == Format::Code && text[selection.clone()].contains('\n') {
                prop_assert_eq!(&changed[once.selection.clone()], &text[selected_lines(&text, &selection)]);
            } else {
                prop_assert_eq!(&changed[once.selection.clone()], &text[selection.clone()]);
                prop_assert_eq!(twice.selection, selection);
            }
        }

        /// The shortcuts' round trip: bold, italic, strikethrough and inline
        /// code wrap any selection (empty, at a line's edge, over several
        /// lines, with accents, CJK or emoji) and the same format again
        /// unwraps it, giving back the text and the selection. The wrap
        /// keeps the same text selected, or sits the caret between the
        /// markers when nothing was.
        #[test]
        fn prop_inline_shortcuts_round_trip((text, selection) in with_edge_selection(unicode()), which in 0usize..4) {
            let format_ = [Format::Bold, Format::Italic, Format::Strikethrough, Format::Code][which];
            let multi_line = text[selection.clone()].contains('\n');
            // Code across lines becomes a fenced block, which has its own
            // property below.
            prop_assume!(!(format_ == Format::Code && multi_line));
            let once = format(format_, &text, selection.clone());
            let changed = once.apply(&text);
            prop_assert_eq!(&changed[once.selection.clone()], &text[selection.clone()]);
            prop_assert!(changed.len() > text.len());
            let twice = format(format_, &changed, once.selection.clone());
            prop_assert_eq!(twice.apply(&changed), text);
            prop_assert_eq!(twice.selection, selection);
        }

        /// A line format applied twice to any multi-line selection, with
        /// accents, CJK and emoji and blank lines among the lines, gives the
        /// text back.
        #[test]
        fn prop_line_shortcuts_twice_restore((text, selection) in with_edge_selection(unicode()), which in 0..LINE_FORMATS.len()) {
            let first = format(LINE_FORMATS[which], &text, selection.clone());
            let changed = first.apply(&text);
            let second = format(LINE_FORMATS[which], &changed, first.selection.clone());
            prop_assert_eq!(second.apply(&changed), text);
        }

        /// A code block applied twice to any selection gives the text back,
        /// and fences exactly the lines the selection touches.
        #[test]
        fn prop_code_block_twice_restores((text, selection) in with_edge_selection(unicode())) {
            let once = format(Format::CodeBlock, &text, selection.clone());
            let changed = once.apply(&text);
            prop_assert_eq!(&changed[once.selection.clone()], &text[selected_lines(&text, &selection)]);
            let twice = format(Format::CodeBlock, &changed, once.selection.clone());
            prop_assert_eq!(twice.apply(&changed), text);
        }

        /// Templates: the cursor or selection lands inside what was
        /// inserted, a link keeps the selected text as its label, and the
        /// template sits on lines of its own when it is a block.
        #[test]
        fn prop_templates_put_the_cursor_inside((text, selection) in with_selection(markdown()), which in 0usize..4) {
            let format_ = [Format::Link, Format::Image, Format::Table, Format::Rule][which];
            let edit = format(format_, &text, selection.clone());
            let inserted = edit.range.start..edit.range.start + edit.text.len();
            prop_assert!(inserted.start <= edit.selection.start && edit.selection.end <= inserted.end);
            let new = edit.apply(&text);
            match format_ {
                Format::Link | Format::Image => {
                    prop_assert_eq!(&edit.range, &selection);
                    let selected = &text[selection.clone()];
                    let (label, target) = (format!("[{selected}]"), format!("({selected})"));
                    prop_assert!(edit.text.contains(&label) || edit.text.contains(&target));
                }
                _ => {
                    let block = if format_ == Format::Table { TABLE } else { RULE };
                    let at = new.find(block).unwrap();
                    prop_assert!(at == 0 || new[..at].ends_with('\n'));
                    prop_assert!(new[at + block.len()..].is_empty() || new[at + block.len()..].starts_with('\n'));
                }
            }
        }
    }

    #[test]
    fn cycling_headings_goes_one_two_three_and_off() {
        let mut text = "Goals\nnext".to_owned();
        let mut selection = 2..2;
        let mut seen = Vec::new();
        for _ in 0..5 {
            let edit = cycle_heading(&text, selection.clone());
            text = edit.apply(&text);
            selection = edit.selection;
            seen.push(text.lines().next().unwrap().to_owned());
        }
        assert_eq!(
            seen,
            ["# Goals", "## Goals", "### Goals", "Goals", "# Goals"]
        );
        // Deeper headings are plain again at once.
        assert_eq!(cycle_heading("#### Deep", 5..5).apply("#### Deep"), "Deep");
    }

    #[test]
    fn brackets_wrap_the_selection_and_wait_before_the_closers() {
        let edit = wiki_brackets("see Plan now", 4..8);
        assert_eq!(edit.apply("see Plan now"), "see [[Plan]] now");
        assert_eq!(edit.selection, 10..10);
        let edit = wiki_brackets("a", 1..1);
        assert_eq!((edit.apply("a"), edit.selection), ("a[[]]".into(), 3..3));
    }

    fn apply(format_: Format, text: &str, selection: Range<usize>) -> (String, String) {
        let edit = format(format_, text, selection);
        let new = edit.apply(text);
        let selected = new[edit.selection.clone()].to_owned();
        (new, selected)
    }

    #[test]
    fn heading_examples() {
        assert_eq!(
            apply(Format::Heading(2), "Goals\nnext", 2..2).0,
            "## Goals\nnext"
        );
        assert_eq!(apply(Format::Heading(2), "## Goals", 4..4).0, "Goals");
        assert_eq!(apply(Format::Heading(1), "### Goals", 4..4).0, "# Goals");
        // Only the cursor's line, not the ones the selection runs into.
        assert_eq!(apply(Format::Heading(3), "a\nb\nc", 2..5).0, "a\n### b\nc");
        assert_eq!(heading_marker("#tag"), None);
        assert_eq!(heading_marker("##"), Some((2, 2)));
    }

    #[test]
    fn list_examples() {
        assert_eq!(
            apply(Format::Bulleted, "one\n\ntwo", 0..8).0,
            "- one\n\n- two"
        );
        assert_eq!(
            apply(Format::Numbered, "one\ntwo\nthree", 0..13).0,
            "1. one\n2. two\n3. three"
        );
        assert_eq!(
            apply(Format::Task, "- one\n  - two", 0..13).0,
            "- [ ] one\n  - [ ] two"
        );
        assert_eq!(apply(Format::Task, "- [x] done", 3..3).0, "done");
        assert_eq!(apply(Format::Bulleted, "- a\n- b", 0..7).0, "a\nb");
        // A selection that ends at a line's start leaves that line alone.
        assert_eq!(apply(Format::Bulleted, "a\nb\nc", 0..4).0, "- a\n- b\nc");
        // A blank line on its own starts a list.
        assert_eq!(
            apply(Format::Bulleted, "a\n\nb", 2..2),
            ("a\n- \nb".into(), String::new())
        );
        assert_eq!(apply(Format::Quote, "> a\nb", 0..5).0, "> a\n> b");
        assert_eq!(apply(Format::Quote, "> a\n>b", 0..6).0, "a\nb");
    }

    #[test]
    fn wrap_examples() {
        assert_eq!(
            apply(Format::Code, "run it", 0..3),
            ("`run` it".into(), "run".into())
        );
        assert_eq!(
            apply(Format::Code, "`run` it", 1..4),
            ("run it".into(), "run".into())
        );
        assert_eq!(
            apply(Format::Bold, "**run** it", 0..7),
            ("run it".into(), "run".into())
        );
        assert_eq!(
            apply(Format::Italic, "ab", 1..1),
            ("a__b".into(), String::new())
        );
        assert_eq!(
            apply(Format::Code, "a\nfn x\nfn y\nb", 2..11),
            ("a\n```\nfn x\nfn y\n```\nb".into(), "fn x\nfn y".into())
        );
        assert_eq!(
            apply(Format::Code, "a\n```\nfn x\nfn y\n```\nb", 6..15).0,
            "a\nfn x\nfn y\nb"
        );
    }

    #[test]
    fn shortcut_examples() {
        // ⌘B on "hello" selected, then with nothing selected.
        assert_eq!(
            apply(Format::Bold, "hello", 0..5),
            ("**hello**".into(), "hello".into())
        );
        let edit = format(Format::Bold, "", 0..0);
        assert_eq!((edit.apply(""), edit.selection), ("****".into(), 2..2));
        // ⇧⌘L on two lines.
        assert_eq!(
            apply(Format::Task, "one\ntwo", 0..7).0,
            "- [ ] one\n- [ ] two"
        );
        // A code block around the cursor's line, empty at the cursor, and gone again.
        assert_eq!(
            apply(Format::CodeBlock, "a\nfn x\nb", 3..3),
            ("a\n```\nfn x\n```\nb".into(), "fn x".into())
        );
        let edit = format(Format::CodeBlock, "", 0..0);
        assert_eq!(
            (edit.apply(""), edit.selection),
            ("```\n\n```".into(), 4..4)
        );
        assert_eq!(apply(Format::CodeBlock, "```\nfn x\n```", 0..12).0, "fn x");
        assert_eq!(apply(Format::Heading(6), "word", 0..0).0, "###### word");
    }

    #[test]
    fn template_examples() {
        assert_eq!(
            apply(Format::Link, "see docs", 4..8),
            ("see [docs](https://)".into(), "https://".into())
        );
        let (text, _) = apply(Format::Link, "https://kanso.app", 0..17);
        assert_eq!(text, "[](https://kanso.app)");
        assert_eq!(apply(Format::Image, "", 0..0).0, "![](https://)");
        assert_eq!(
            apply(Format::Table, "intro\nmore", 2..2),
            (format!("intro\n\n{TABLE}\n\nmore"), "Column".into())
        );
        assert_eq!(
            apply(Format::Table, "a\n\nb", 2..2).0,
            format!("a\n{TABLE}\n\nb")
        );
        assert_eq!(apply(Format::Rule, "a", 1..1).0, "a\n\n---");
    }
}
