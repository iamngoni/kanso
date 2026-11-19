//! The plain-text excerpt shown under a note's title in the note list.
//!
//! Most notes start with a heading that repeats their title, and Markdown
//! markers read as noise in a two-line preview, so the excerpt skips front
//! matter, code blocks, display math, table rules and a leading title
//! heading, and keeps only the words of everything else. Inline math keeps
//! its TeX without the dollar signs. Blocks (headings, list items, table
//! rows, paragraphs) are set apart with " · " so they don't run together
//! into one misleading sentence.
//!
//! A search result's excerpt starts near the first match instead, so the
//! list can show why the note matched.

use crate::query::find_ci;

/// The longest excerpt, in characters.
pub const EXCERPT_CHARS: usize = 200;
/// How much of a body the store reads for its excerpt, in characters: enough
/// to get past front matter, a title heading and a code block or two.
pub const HEAD_CHARS: usize = 2000;
/// How much of a body around a match beyond [`HEAD_CHARS`] the store reads,
/// in characters.
pub const WINDOW_CHARS: usize = 600;
/// How far before a match a window may start on a long line, in characters.
const WINDOW_LEAD: usize = 200;
/// A match no further than this into the text shows from the start.
const LEAD_CHARS: usize = 60;
/// How much text before a match an excerpt that starts near it keeps.
const CONTEXT_CHARS: usize = 24;
/// Between two blocks that don't end a sentence.
const SEPARATOR: &str = " · ";
/// Where a marker was taken out: a space between two words, nothing next to
/// punctuation or a space. Control characters in the text become spaces, so
/// this never comes from the text itself.
const REMOVED: char = '\u{1}';

/// The note's body as one line of plain text, at most [`EXCERPT_CHARS`]
/// characters, without a first heading that repeats `title`.
pub fn excerpt(title: &str, body: &str) -> String {
    truncate(&plain_text(title, body, EXCERPT_CHARS), EXCERPT_CHARS)
}

/// The excerpt for a search result: from shortly before the first match of
/// any of `terms` in the note's text, prefixed with "…" when that isn't its
/// start. `head` is the start of the body, and `window` the part around a
/// match beyond it (see [`match_window`]). Without a match in either, or
/// without terms, it is the usual [`excerpt`] of `head`.
pub fn excerpt_around(
    title: &str,
    head: &str,
    window: Option<&str>,
    terms: &[impl AsRef<str>],
) -> String {
    if terms.iter().all(|t| t.as_ref().is_empty()) {
        return excerpt(title, head);
    }
    let text = plain_text(title, head, HEAD_CHARS);
    if let Some(at) = first_match(&text, terms) {
        return excerpt_at(&text, at, false);
    }
    if let Some(window) = window {
        let around = plain_text(title, window, WINDOW_CHARS);
        if let Some(at) = first_match(&around, terms) {
            return excerpt_at(&around, at, true);
        }
    }
    truncate(&text, EXCERPT_CHARS)
}

/// The part of `body` around the first match of `term`, for an excerpt,
/// when the first [`HEAD_CHARS`] characters don't hold a match: from the
/// start of the match's line (or [`WINDOW_LEAD`] characters before the
/// match on a long line), [`WINDOW_CHARS`] long and never ending before the
/// match does. `None` when the head holds a match or the body none.
pub fn match_window<'a>(body: &'a str, term: &str) -> Option<&'a str> {
    if term.is_empty() {
        return None;
    }
    let head_end = char_offset(body, 0, HEAD_CHARS);
    if find_ci(&body[..head_end], term).is_some() {
        return None;
    }
    let at = find_ci(body, term)?;
    let line_start = body[..at].rfind('\n').map_or(0, |i| i + 1);
    let lead_start = char_offset_back(body, at, WINDOW_LEAD);
    let start = line_start.max(lead_start);
    let match_end = char_offset(body, at, term.chars().count());
    let end = char_offset(body, start, WINDOW_CHARS).max(match_end);
    Some(&body[start..end])
}

/// The byte offset `chars` characters after byte `from`, or the end.
fn char_offset(text: &str, from: usize, chars: usize) -> usize {
    text[from..]
        .char_indices()
        .nth(chars)
        .map_or(text.len(), |(i, _)| from + i)
}

/// The byte offset `chars` characters before byte `to`, or the start.
fn char_offset_back(text: &str, to: usize, chars: usize) -> usize {
    if chars == 0 {
        return to;
    }
    text[..to]
        .char_indices()
        .rev()
        .nth(chars - 1)
        .map_or(0, |(i, _)| i)
}

/// Where the earliest match of any of `terms` starts in `text`.
fn first_match(text: &str, terms: &[impl AsRef<str>]) -> Option<usize> {
    terms
        .iter()
        .map(AsRef::as_ref)
        .filter(|t| !t.is_empty())
        .filter_map(|t| find_ci(text, t))
        .min()
}

/// An excerpt of `text` that shows the match at byte `at`: the start when
/// the match is near it, otherwise a few words before the match with "…".
/// `mid` says `text` itself starts inside the note.
fn excerpt_at(text: &str, at: usize, mid: bool) -> String {
    if !mid && text[..at].chars().count() <= LEAD_CHARS {
        return truncate(text, EXCERPT_CHARS);
    }
    let floor = char_offset_back(text, at, CONTEXT_CHARS);
    // The start of a word, no more than the context before the match: the
    // match's own word, then earlier ones while they fit.
    let mut start = text[..at].rfind(' ').map_or(0, |i| i + 1).max(floor);
    loop {
        let before = text[..start].trim_end();
        if before.is_empty() {
            break;
        }
        let word = before.rfind(' ').map_or(0, |space| space + 1);
        if word < floor {
            break;
        }
        start = word;
    }
    // Never open on a block separator.
    let rest = text[start..].trim_start_matches(['·', ' ']);
    let prefix = if mid || start > 0 { "…" } else { "" };
    format!(
        "{prefix}{}",
        truncate(rest, EXCERPT_CHARS - prefix.chars().count())
    )
}

/// The note's text, blocks set apart, from the start until a line takes it
/// past `cap` characters.
fn plain_text(title: &str, body: &str, cap: usize) -> String {
    let mut out = String::new();
    let mut count = 0;
    let mut lines = body.lines().peekable();

    // YAML front matter.
    if lines.peek().is_some_and(|l| l.trim_end() == "---") {
        lines.next();
        for line in lines.by_ref() {
            if matches!(line.trim_end(), "---" | "...") {
                break;
            }
        }
    }

    let mut fence: Option<&str> = None;
    let mut in_math = false;
    let mut seen_text = false;
    // Whether the next line starts a block: after a blank line, a rule, a
    // code block, a heading or a table row.
    let mut block_ended = false;
    let mut in_quote = false;
    for line in lines {
        let trimmed = line.trim();
        if let Some(marker) = fence {
            if trimmed.starts_with(marker) {
                fence = None;
            }
            continue;
        }
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fence = Some(&trimmed[..3]);
            block_ended = true;
            continue;
        }
        // A display-math block, `$$` to `$$`, or `$$…$$` on one line.
        if in_math {
            if trimmed.ends_with("$$") {
                in_math = false;
            }
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("$$") {
            in_math = !rest.ends_with("$$");
            block_ended = true;
            continue;
        }
        if trimmed.is_empty() || is_rule(trimmed) {
            block_ended = true;
            continue;
        }
        let (block, plain) = if trimmed.starts_with('|') {
            (Block::TableRow, table_row(trimmed))
        } else {
            let (block, text) = strip_block_markers(trimmed);
            (block, strip_inline(text))
        };
        if plain.is_empty() {
            continue;
        }
        if !seen_text && block == Block::Heading && same_words(&plain, title) {
            seen_text = true;
            block_ended = true;
            continue;
        }
        seen_text = true;
        let quote = block == Block::Quote;
        let starts_block = block_ended
            || matches!(block, Block::Heading | Block::Item | Block::TableRow)
            || quote != in_quote;
        if !out.is_empty() {
            if starts_block && !out.ends_with(['.', '!', '?', ':', ';', '…']) {
                out.push_str(SEPARATOR);
                count += SEPARATOR.chars().count();
            } else {
                out.push(' ');
                count += 1;
            }
        }
        out.push_str(&plain);
        count += plain.chars().count();
        block_ended = matches!(block, Block::Heading | Block::TableRow);
        in_quote = quote;
        if count >= cap {
            break;
        }
    }
    out
}

/// What a line's leading markers make it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Block {
    /// Paragraph text, or a paragraph's next line.
    Text,
    Heading,
    /// A list or task item.
    Item,
    Quote,
    TableRow,
}

/// A table row's cells, each as plain text, set apart.
fn table_row(line: &str) -> String {
    let mut cells = Vec::new();
    let mut cell = String::new();
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            // An escaped pipe is part of the cell.
            '\\' if chars.peek() == Some(&'|') => {
                cell.push(c);
                cell.extend(chars.next());
            }
            '|' => cells.push(std::mem::take(&mut cell)),
            c => cell.push(c),
        }
    }
    cells.push(cell);
    cells
        .iter()
        .map(|cell| strip_inline(cell.trim()))
        .filter(|cell| !cell.is_empty())
        .collect::<Vec<_>>()
        .join(SEPARATOR)
}

/// A thematic break or a table's header rule, such as `---` or `|:--|--:|`.
fn is_rule(line: &str) -> bool {
    let marks = line
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    marks.len() >= 3
        && (marks.chars().all(|c| c == '-')
            || marks.chars().all(|c| c == '*')
            || marks.chars().all(|c| c == '_')
            || (marks.contains('-') && marks.chars().all(|c| matches!(c, '|' | '-' | ':'))))
}

/// Removes heading, quote, list and task markers, and says what they made
/// the line.
fn strip_block_markers(line: &str) -> (Block, &str) {
    let mut rest = line;
    let mut block = Block::Text;
    loop {
        let before = rest;
        if let Some(after) = rest.strip_prefix('>') {
            rest = after.trim_start();
            if block == Block::Text {
                block = Block::Quote;
            }
        }
        let hashes = rest.chars().take_while(|&c| c == '#').count();
        if (1..=6).contains(&hashes) && rest[hashes..].starts_with([' ', '\t']) {
            block = Block::Heading;
            rest = rest[hashes..].trim_start();
        }
        for marker in ["- ", "* ", "+ "] {
            if let Some(after) = rest.strip_prefix(marker) {
                rest = after.trim_start();
                block = Block::Item;
            }
        }
        let digits = rest.chars().take_while(char::is_ascii_digit).count();
        if digits > 0
            && digits <= 9
            && (rest[digits..].starts_with(". ") || rest[digits..].starts_with(") "))
        {
            rest = rest[digits + 2..].trim_start();
            block = Block::Item;
        }
        for task in ["[ ] ", "[x] ", "[X] "] {
            if let Some(after) = rest.strip_prefix(task) {
                rest = after.trim_start();
                block = Block::Item;
            }
        }
        if rest == before {
            break;
        }
    }
    // Closing hashes of an ATX heading.
    if block == Block::Heading {
        rest = rest.trim_end_matches('#').trim_end();
    }
    (block, rest)
}

/// Keeps the readable text of inline Markdown: link and image text, code
/// contents, words without emphasis markers, no HTML tags or table pipes.
/// A marker taken out leaves a space only between two words, so "by
/// `updatedAt`;" reads "by updatedAt;".
fn strip_inline(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            // An image keeps nothing, not even the space before it.
            '!' if chars.get(i + 1) == Some(&'[') => {
                if let Some(end) = link_end(&chars, i + 1) {
                    out.truncate(out.trim_end().len());
                    out.push(REMOVED);
                    i = end;
                    continue;
                }
                out.push(c);
            }
            '[' => {
                // [[wiki link]]: what it reads as in the preview, its
                // alias or its title (and heading).
                if chars.get(i + 1) == Some(&'[')
                    && let Some(close) = find(&chars, i + 2, &[']', ']'])
                {
                    let inner: String = chars[i + 2..close].iter().collect();
                    let shown = match crate::links::parse_target(&inner) {
                        Some((title, heading, alias)) => crate::links::WikiLink {
                            range: 0..0,
                            title,
                            heading,
                            alias,
                        }
                        .label(),
                        None => inner,
                    };
                    out.push(REMOVED);
                    out.push_str(&strip_inline(&shown));
                    out.push(REMOVED);
                    i = close + 2;
                    continue;
                }
                // A link keeps its text.
                if let Some(end) = link_end(&chars, i) {
                    let close = find(&chars, i + 1, &[']']).unwrap_or(i);
                    out.push_str(&strip_inline(
                        &chars[i + 1..close].iter().collect::<String>(),
                    ));
                    i = end;
                    continue;
                }
                out.push(c);
            }
            '<' => match chars[i + 1..].iter().position(|&c| c == '>') {
                Some(len) if looks_like_tag(&chars[i + 1..i + 1 + len]) => {
                    out.push(REMOVED);
                    i += len + 2;
                    continue;
                }
                _ => out.push(c),
            },
            // `$$…$$` within a line is display math: dropped.
            '$' if chars.get(i + 1) == Some(&'$') => {
                match find(&chars, i + 2, &['$', '$']) {
                    Some(close) => {
                        out.push(REMOVED);
                        i = close + 2;
                    }
                    None => {
                        out.push_str("$$");
                        i += 2;
                    }
                }
                continue;
            }
            // Inline math keeps its TeX, without the dollar signs.
            '$' => {
                if let Some(close) = inline_math_end(&chars, i) {
                    out.push(REMOVED);
                    let mut last = ' ';
                    for &t in &chars[i + 1..close] {
                        let t = match t {
                            '`' => REMOVED,
                            '|' => ' ',
                            t if t.is_control() => ' ',
                            t => t,
                        };
                        // Runs of `*` would read as emphasis.
                        if !(t == '*' && last == '*') {
                            out.push(t);
                        }
                        last = t;
                    }
                    out.push(REMOVED);
                    i = close + 1;
                    continue;
                }
                out.push(c);
            }
            '`' => out.push(REMOVED),
            '|' => out.push(' '),
            '*' | '_' | '~' => {
                // Emphasis markers sit at a word's edge; inside a word
                // (snake_case, 2*3) they are text.
                let prev = i.checked_sub(1).map(|p| chars[p]);
                let next = chars.get(i + 1).copied();
                let inside = prev.is_some_and(char::is_alphanumeric)
                    && next.is_some_and(char::is_alphanumeric);
                out.push(if inside { c } else { REMOVED });
            }
            '\\' if chars.get(i + 1).is_some_and(|n| n.is_ascii_punctuation()) => {
                // An escaped marker is still noise in a preview.
                let escaped = chars[i + 1];
                out.push(if "`|*_~".contains(escaped) {
                    REMOVED
                } else {
                    escaped
                });
                i += 2;
                continue;
            }
            c if c.is_control() => out.push(' '),
            c => out.push(c),
        }
        i += 1;
    }
    close_gaps(&out)
}

/// Resolves the places markers were taken out, then normalizes spaces. A
/// gap between two words becomes a space; next to a space or punctuation
/// it closes, unless closing it would make new Markdown, such as `](`.
fn close_gaps(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != REMOVED {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        let prev = i.checked_sub(1).map(|p| chars[p]);
        while chars.get(i) == Some(&REMOVED) {
            i += 1;
        }
        let next = chars.get(i).copied();
        if let (Some(prev), Some(next)) = (prev, next)
            && ((prev.is_alphanumeric() && next.is_alphanumeric()) || makes_markup(prev, next))
        {
            out.push(' ');
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Whether `prev` right before `next` would read as Markdown or HTML that
/// wasn't there: a link, an image, a wiki link, a tag or an escape.
fn makes_markup(prev: char, next: char) -> bool {
    matches!(
        (prev, next),
        (']', '(') | ('!', '[') | ('[', '[') | (']', ']')
    ) || prev == '<'
        || next == '>'
        || prev == '\\'
}

/// The index after `[text](target)` starting at `open`, if it is a link.
fn link_end(chars: &[char], open: usize) -> Option<usize> {
    let close = find(chars, open + 1, &[']'])?;
    if chars.get(close + 1) != Some(&'(') {
        return None;
    }
    let end = find(chars, close + 2, &[')'])?;
    Some(end + 1)
}

fn find(chars: &[char], from: usize, needle: &[char]) -> Option<usize> {
    (from..chars.len().saturating_sub(needle.len() - 1))
        .find(|&i| chars[i..i + needle.len()] == *needle)
}

/// Where the `$` closing inline math opened by the `$` at `open` is, by
/// Pandoc's rule: the opening `$` is not followed by a space, the closing
/// one is not preceded by a space or followed by a digit, and neither is
/// escaped. So "$5 and $10" is not math. The same rule as the preview's.
fn inline_math_end(chars: &[char], open: usize) -> Option<usize> {
    if chars.get(open) != Some(&'$') || (open > 0 && chars[open - 1] == '\\') {
        return None;
    }
    let first = *chars.get(open + 1)?;
    if first.is_whitespace() || first == '$' {
        return None;
    }
    let mut i = open + 1;
    while i < chars.len() {
        match chars[i] {
            // An escaped character, `\$` included, is part of the TeX.
            '\\' => i += 2,
            '$' => {
                let before = chars[i - 1];
                let after = chars.get(i + 1);
                if !before.is_whitespace() && !after.is_some_and(char::is_ascii_digit) {
                    return Some(i);
                }
                i += 1;
            }
            _ => i += 1,
        }
    }
    None
}

/// `<b>`, `</div>`, `<img src="…">`, `<br/>`: a name right after the bracket.
fn looks_like_tag(inner: &[char]) -> bool {
    let inner = inner.strip_prefix(&['/']).unwrap_or(inner);
    inner.first().is_some_and(char::is_ascii_alphabetic)
        && inner
            .iter()
            // What reads as a space or goes once stripped (a table pipe, an
            // emphasis marker) ends the name too, so stripping twice
            // finds the same tags.
            .take_while(|c| {
                !c.is_whitespace()
                    && !c.is_control()
                    && !matches!(**c, '/' | '|' | '*' | '_' | '~' | '`')
            })
            .all(|c| c.is_ascii_alphanumeric() || *c == '-')
}

/// Whether two texts have the same words, ignoring case and punctuation.
fn same_words(a: &str, b: &str) -> bool {
    let words = |s: &str| -> Vec<String> {
        s.split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .map(str::to_lowercase)
            .collect()
    };
    let (a, b) = (words(a), words(b));
    !a.is_empty() && a == b
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let cut: String = text.chars().take(max).collect();
    // End on a word boundary when one is reasonably close, and never on a
    // block separator.
    let cut = match cut.rfind(' ') {
        Some(at) if at > max / 2 => &cut[..at],
        _ => &cut[..],
    };
    cut.trim_end_matches([' ', '·']).to_owned()
}

#[cfg(test)]
mod tests {
    use super::{
        EXCERPT_CHARS, HEAD_CHARS, WINDOW_CHARS, excerpt, excerpt_around, inline_math_end,
        match_window, plain_text, strip_inline,
    };
    use crate::query::find_ci;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    #[test]
    fn examples() {
        assert_eq!(
            excerpt(
                "Reading list",
                "# Reading list\n\n1. *The Design of Everyday Things*\n2. **Shape Up**\n"
            ),
            "The Design of Everyday Things · Shape Up"
        );
        assert_eq!(
            excerpt(
                "Weekly sync",
                "# Weekly sync\n\n| Topic | Owner |\n|---|---|\n| Release | Ngoni |"
            ),
            "Topic · Owner · Release · Ngoni"
        );
        assert_eq!(
            excerpt(
                "x",
                "---\ntitle: x\n---\nSee [the docs](https://e.com/a_b) and ![shot](kanso://file:abc)."
            ),
            "See the docs and."
        );
        assert_eq!(
            excerpt(
                "Plan",
                "## Goals\n```rust\nfn main() {}\n```\n- [x] Ship `v1`\n> quoted <b>bold</b>"
            ),
            "Goals · Ship v1 · quoted bold"
        );
        // A heading that isn't the title stays; snake_case and 2*3 stay.
        assert_eq!(
            excerpt("Other", "# Heading\nuse snake_case and 2*3"),
            "Heading · use snake_case and 2*3"
        );
        assert_eq!(excerpt("T", ""), "");
        assert_eq!(excerpt("T", "# T"), "");
        assert_eq!(excerpt("a < b", "a < b > c"), "a < b > c");
    }

    /// Taking out a marker leaves no space before the punctuation after it.
    #[test]
    fn markers_leave_no_gaps() {
        assert_eq!(
            excerpt("x", "wins by `updatedAt`; the"),
            "wins by updatedAt; the"
        );
        assert_eq!(excerpt("x", "(worktree `../a`)"), "(worktree ../a)");
        assert_eq!(excerpt("x", "is **reliable**."), "is reliable.");
        assert_eq!(
            excerpt("x", "reopened: **Not Working In Project**, 100%"),
            "reopened: Not Working In Project, 100%"
        );
        assert_eq!(excerpt("x", "a `b` c and x`y`z"), "a b c and x y z");
        assert_eq!(excerpt("x", "see [[Other note]]."), "see Other note.");
        // A wiki-link reads as its alias, else its title and heading.
        assert_eq!(
            excerpt("x", "within its [[MCP support model#Goals|goals]]."),
            "within its goals."
        );
        assert_eq!(
            excerpt("x", "see [[MCP support model#Goals]]"),
            "see MCP support model › Goals"
        );
        assert_eq!(excerpt("x", "see [[Plan | ]] now"), "see Plan now");
    }

    /// Characters that only look like an excerpt's own marks stay as typed.
    #[test]
    fn typed_dots_and_brackets_stay() {
        assert_eq!(excerpt("", "·"), "·");
        assert_eq!(excerpt("", "]("), "](");
        assert_eq!(excerpt("", "One\n\nTwo ·"), "One · Two ·");
    }

    #[test]
    fn math_keeps_its_tex_and_drops_display_blocks() {
        assert_eq!(
            excerpt("x", "Area is $\\pi r^2$, roughly."),
            "Area is \\pi r^2, roughly."
        );
        assert_eq!(
            excerpt("x", "Costs $5 and $10 today"),
            "Costs $5 and $10 today"
        );
        assert_eq!(excerpt("x", "a $ b$ c"), "a $ b$ c");
        assert_eq!(excerpt("x", "a $b $ c"), "a $b $ c");
        assert_eq!(excerpt("x", "a \\$b$ c"), "a $b$ c");
        assert_eq!(
            excerpt("x", "Before\n\n$$\n\\int_0^1 x\\,dx\n$$\n\nAfter"),
            "Before · After"
        );
        assert_eq!(
            excerpt("x", "Before\n\n$$E = mc^2$$\n\nAfter"),
            "Before · After"
        );
        assert_eq!(excerpt("x", "So $$x$$ there"), "So there");
        // Closing the gap would make a link; it stays open.
        assert_eq!(strip_inline("[a`]`(b)"), "[a] (b)");
    }

    /// Headings, items, table rows and paragraphs are set apart; the lines
    /// of one paragraph are not.
    #[test]
    fn blocks_are_set_apart() {
        assert_eq!(
            excerpt(
                "Kanso roadmap",
                "# Kanso roadmap\n\nA calm, local-first notes app.\n\n## Next\n\n- [x] Wiki-links and backlinks\n- [x] Local API on port 19840\n- [ ] Command palette\n"
            ),
            "A calm, local-first notes app. Next · Wiki-links and backlinks · Local API on port 19840 · Command palette"
        );
        assert_eq!(
            excerpt(
                "Release checklist",
                "---\ntitle: Release checklist\n---\n\n# Release checklist\n\n- [x] Bump the version\n- [ ] Notarize the build\n"
            ),
            "Bump the version · Notarize the build"
        );
        assert_eq!(excerpt("", "one\ntwo"), "one two");
        assert_eq!(excerpt("", "Para one\n\nPara two"), "Para one · Para two");
        assert_eq!(excerpt("", "Para one.\n\nPara two"), "Para one. Para two");
        assert_eq!(
            excerpt("", "text\n> a quote\n> goes on"),
            "text · a quote goes on"
        );
        // A separator never ends a cut excerpt.
        let long = format!("{}\n\n{}", "word ".repeat(39).trim(), "next ".repeat(20));
        let cut = excerpt("", &long);
        assert!(!cut.ends_with('·') && !cut.ends_with(' '), "{cut}");
    }

    #[test]
    fn search_excerpts_start_near_the_match() {
        let body = "# Notes\n\nThe introduction runs on for a while before anything about the engine. The sync engine pushes changes.";
        let found = excerpt_around("Notes", body, None, &["SYNC"]);
        assert!(found.starts_with('…'), "{found}");
        assert!(found.contains("sync engine"), "{found}");
        assert!(found.chars().count() <= EXCERPT_CHARS);
        // Near the start, the excerpt is the usual one.
        let near = "Sync first, then the rest.";
        assert_eq!(excerpt_around("", near, None, &["sync"]), excerpt("", near));
        // No terms, or no match: the usual excerpt.
        let none: [&str; 0] = [];
        assert_eq!(
            excerpt_around("Notes", body, None, &none),
            excerpt("Notes", body)
        );
        assert_eq!(
            excerpt_around("Notes", body, None, &["absent"]),
            excerpt("Notes", body)
        );
        // A match only in the window beyond the head.
        let head = "Nothing here.";
        let window = "Deep down the sync design lives.";
        assert_eq!(
            excerpt_around("", head, Some(window), &["sync"]),
            "…Deep down the sync design lives."
        );
    }

    #[test]
    fn match_windows() {
        assert_eq!(match_window("a sync b", "sync"), None, "in the head");
        assert_eq!(match_window("abc", "sync"), None);
        assert_eq!(match_window("abc", ""), None);
        let body = format!("{}\nline with Sync here\nmore", "x ".repeat(HEAD_CHARS));
        assert_eq!(
            match_window(&body, "sync"),
            Some("line with Sync here\nmore")
        );
        // On a long line the window starts a little before the match.
        let long = format!("{}sync", "y".repeat(3000));
        let window = match_window(&long, "sync").unwrap();
        assert_eq!(window.len(), 204);
        assert!(window.ends_with("sync"));
    }

    fn gen_word() -> impl Strategy<Value = String> {
        "[A-Za-z0-9]{1,8}"
    }

    fn gen_markdown() -> impl Strategy<Value = String> {
        let line = prop_oneof![
            prop::collection::vec(gen_word(), 1..6).prop_map(|w| w.join(" ")),
            prop::collection::vec(gen_word(), 1..4).prop_map(|w| format!("# {}", w.join(" "))),
            gen_word().prop_map(|w| format!("- [ ] **{w}**")),
            (gen_word(), gen_word()).prop_map(|(a, b)| format!("[{a}](https://x.y/{b})")),
            gen_word().prop_map(|w| format!("> _{w}_ `code`")),
            Just("|---|:-:|".to_owned()),
            Just("```\nfn hidden() {}\n```".to_owned()),
            "\\PC{0,30}",
        ];
        prop::collection::vec(line, 0..12).prop_map(|lines| lines.join("\n"))
    }

    /// A word wrapped in an inline marker, then maybe punctuation.
    fn gen_marked() -> impl Strategy<Value = String> {
        let marker = prop::sample::select(vec!["`", "**", "*", "_", "__", "~~"]);
        let punct = prop::sample::select(vec!["", ",", ";", ".", ":", "!", "?"]);
        (marker, gen_word(), punct, any::<bool>()).prop_map(|(m, w, p, paren)| {
            if paren {
                format!("({m}{w}{m}){p}")
            } else {
                format!("{m}{w}{m}{p}")
            }
        })
    }

    /// Filler words and the term, somewhere among them, in any case.
    fn gen_search_body() -> impl Strategy<Value = (String, String)> {
        (
            prop::collection::vec(gen_word(), 0..80),
            "[a-z]{3,8}",
            any::<prop::sample::Index>(),
            any::<bool>(),
        )
            .prop_map(|(mut words, term, at, upper)| {
                let placed = if upper {
                    term.to_uppercase()
                } else {
                    term.clone()
                };
                let ix = at.index(words.len() + 1);
                words.insert(ix, placed);
                // Lines of up to eight words, in paragraphs.
                let body = words
                    .chunks(8)
                    .map(|line| line.join(" "))
                    .collect::<Vec<_>>()
                    .join("\n");
                (body, term)
            })
    }

    proptest! {
        #![proptest_config(config())]

        /// Refinement: an excerpt is one trimmed line, short, and free of
        /// the Markdown markers that read as noise.
        #[test]
        fn prop_excerpts_are_short_plain_lines(title in "\\PC{0,20}", body in gen_markdown()) {
            let text = excerpt(&title, &body);
            prop_assert!(text.chars().count() <= EXCERPT_CHARS);
            let noise = ['\n', '\r', '`', '|', '\u{1}'];
            prop_assert!(!text.contains(noise));
            prop_assert!(!text.contains("**"));
            // A link's syntax never shows; the same characters typed as
            // text, which are no link, stay as typed.
            prop_assert!(!text.contains("](") || body.contains("]("), "{:?}", text);
            prop_assert_eq!(text.trim(), text.as_str());
            prop_assert!(!text.contains("  "));
            // No block separator dangles at the end; a "·" the note itself
            // ends with is its own.
            prop_assert!(!text.ends_with(" ·") || body.trim_end().ends_with('·'), "{:?}", text);
            prop_assert!(!text.ends_with('·') || body.contains('·'), "{:?}", text);
        }

        /// Pandoc's rule: whatever counts as inline math opens on a
        /// non-space, closes on a non-space not followed by a digit, and
        /// isn't escaped; money amounts never do.
        #[test]
        fn prop_inline_math_follows_pandoc(text in "[a-z $\\\\0-9]{0,24}") {
            let chars: Vec<char> = text.chars().collect();
            for open in 0..chars.len() {
                if let Some(close) = inline_math_end(&chars, open) {
                    prop_assert!(close > open + 1);
                    prop_assert!(!chars[open + 1].is_whitespace());
                    prop_assert!(!chars[close - 1].is_whitespace());
                    prop_assert!(!chars.get(close + 1).is_some_and(char::is_ascii_digit));
                    prop_assert!(open == 0 || chars[open - 1] != '\\');
                }
            }
        }

        /// Oracle: amounts of money stay as written.
        #[test]
        fn prop_amounts_are_not_math(a in 1u32..1000, b in 1u32..1000, w in gen_word()) {
            let body = format!("${a} {w} ${b}");
            prop_assert_eq!(excerpt("", &body), body);
        }

        /// Oracle: inline math reads as its TeX; display math is dropped.
        #[test]
        fn prop_math_reads_as_tex(before in gen_word(), tex in "[a-z^_{}+=]{1,10}", after in gen_word()) {
            prop_assume!(!tex.starts_with(['_']) && !tex.ends_with(['_']));
            prop_assert_eq!(excerpt("", &format!("{before} ${tex}$ {after}")), format!("{before} {tex} {after}"));
            prop_assert_eq!(excerpt("", &format!("{before}\n\n$$\n{tex}\n$$\n\n{after}")), format!("{before} · {after}"));
        }

        /// Invariant: a wiki-link shows as words, never as `[[`, `]]`, or
        /// its `#heading` and `|alias` syntax; an alias stands in for the
        /// title.
        #[test]
        fn prop_wiki_links_read_as_their_label(
            before in prop::collection::vec(gen_word(), 0..4),
            title in prop::collection::vec(gen_word(), 1..3),
            heading in prop::option::of(gen_word()),
            alias in prop::option::of(gen_word()),
            after in prop::collection::vec(gen_word(), 0..4),
        ) {
            let title = title.join(" ");
            let mut inner = title.clone();
            if let Some(heading) = &heading {
                inner.push('#');
                inner.push_str(heading);
            }
            if let Some(alias) = &alias {
                inner.push('|');
                inner.push_str(alias);
            }
            let body = format!("{} [[{inner}]] {}", before.join(" "), after.join(" "));
            let text = excerpt("", &body);
            prop_assert!(!text.contains("[[") && !text.contains("]]"), "{}", text);
            prop_assert!(!text.contains('#') && !text.contains('|'), "{}", text);
            let shown = match (&alias, &heading) {
                (Some(alias), _) => alias.clone(),
                (None, Some(heading)) => format!("{title} › {heading}"),
                (None, None) => title.clone(),
            };
            prop_assert!(text.contains(&shown), "{} lacks {}", text, shown);
        }

        /// Metamorphic: a first heading repeating the title adds nothing.
        #[test]
        fn prop_a_title_heading_is_skipped(words in prop::collection::vec(gen_word(), 1..4), body in gen_markdown()) {
            let title = words.join(" ");
            let with_heading = format!("# {title}\n\n{body}");
            prop_assume!(!body.trim_start().starts_with('#') && !body.trim_start().starts_with("---"));
            prop_assert_eq!(excerpt(&title, &with_heading), excerpt(&title, &body));
        }

        /// Invariant: the excerpt invents nothing; every word in it is in
        /// the body.
        #[test]
        fn prop_excerpt_words_come_from_the_body(body in gen_markdown()) {
            let text = excerpt("", &body);
            for word in text.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()) {
                prop_assert!(body.contains(word), "{} not in body", word);
            }
            prop_assert!(!text.contains("hidden"), "code block content leaked");
        }

        /// Oracle: plain words come through as they are, joined by single
        /// spaces.
        #[test]
        fn prop_plain_text_is_kept(words in prop::collection::vec(gen_word(), 1..20)) {
            let body = words.join("  \n ");
            let expected = words.join(" ");
            prop_assume!(expected.chars().count() <= EXCERPT_CHARS);
            prop_assert_eq!(excerpt("", &body), expected);
        }

        /// Oracle: paragraphs come through set apart by exactly one
        /// separator each, and the lines inside a paragraph never are.
        #[test]
        fn prop_separators_fall_between_paragraphs_only(
            paragraphs in prop::collection::vec(prop::collection::vec(prop::collection::vec(gen_word(), 1..4), 1..3), 1..5),
        ) {
            let body = paragraphs
                .iter()
                .map(|lines| lines.iter().map(|l| l.join(" ")).collect::<Vec<_>>().join("\n"))
                .collect::<Vec<_>>()
                .join("\n\n");
            let expected = paragraphs
                .iter()
                .map(|lines| lines.concat().join(" "))
                .collect::<Vec<_>>()
                .join(" · ");
            prop_assume!(expected.chars().count() <= EXCERPT_CHARS);
            prop_assert_eq!(excerpt("", &body), expected);
        }

        /// Invariant: taking out inline markers never leaves a space before
        /// closing punctuation or after an opening parenthesis that the
        /// text didn't have.
        #[test]
        fn prop_markers_leave_no_space_before_punctuation(
            tokens in prop::collection::vec(prop_oneof![gen_word(), gen_marked()], 1..12),
        ) {
            let body = tokens.join(" ");
            let text = excerpt("", &body);
            for gap in [" ,", " ;", " .", " :", " !", " ?", " )", "( "] {
                prop_assert!(body.contains(gap) || !text.contains(gap), "{:?} in {:?} from {:?}", gap, text, body);
            }
        }

        /// Idempotence: stripping inline Markdown twice changes nothing more.
        /// Backslash escapes are decoded once by design (`\\{` becomes `\{`), so
        /// they are left out, and so is math, whose TeX is kept as written
        /// once its dollar signs are gone.
        #[test]
        fn prop_strip_inline_is_idempotent(text in "[^\\\\$]{0,60}") {
            let once = strip_inline(&text);
            prop_assert_eq!(strip_inline(&once), once);
        }

        /// Metamorphic: without terms a search excerpt is the usual one.
        #[test]
        fn prop_no_terms_no_change(title in "\\PC{0,10}", body in gen_markdown()) {
            let none: [&str; 0] = [];
            prop_assert_eq!(excerpt_around(&title, &body, None, &none), excerpt(&title, &body));
        }

        /// Invariant: whenever the note's text holds the term, so does the
        /// search excerpt, in any case; and it stays short.
        #[test]
        fn prop_search_excerpts_show_the_match((body, term) in gen_search_body(), title in "[a-z ]{0,10}") {
            let found = excerpt_around(&title, &body, None, &[term.as_str()]);
            prop_assert!(found.chars().count() <= EXCERPT_CHARS);
            if find_ci(&plain_text(&title, &body, HEAD_CHARS), &term).is_some() {
                prop_assert!(find_ci(&found, &term).is_some(), "{:?} lacks {:?}", found, term);
            }
        }

        /// Invariants: a window is a piece of the body that holds the term's
        /// first match, no longer than it needs to be; there is one exactly
        /// when the body holds the term and its first characters don't.
        #[test]
        fn prop_match_windows_hold_the_match(
            filler in prop::collection::vec("[a-z]{1,9}( |\n)", 0..700),
            term in "[a-z]{2,6}",
            at in any::<prop::sample::Index>(),
            include in any::<bool>(),
        ) {
            let mut parts = filler;
            if include {
                let ix = at.index(parts.len() + 1);
                parts.insert(ix, format!("{} ", term.to_uppercase()));
            }
            let body = parts.concat();
            let head: String = body.chars().take(HEAD_CHARS).collect();
            let window = match_window(&body, &term);
            let expected = find_ci(&body, &term).is_some() && find_ci(&head, &term).is_none();
            prop_assert_eq!(window.is_some(), expected);
            if let Some(window) = window {
                let offset = window.as_ptr() as usize - body.as_ptr() as usize;
                prop_assert_eq!(&body[offset..offset + window.len()], window);
                prop_assert_eq!(offset + find_ci(window, &term).unwrap(), find_ci(&body, &term).unwrap());
                prop_assert!(window.chars().count() <= WINDOW_CHARS + term.chars().count());
            }
        }
    }
}
