//! Receipts: checking, on this device, that the lines an answer points at
//! are lines the model was shown and say something about the sentence.
//!
//! The model writes `[2:L14]` or `[1:L3-5]` after a claim (excerpt number,
//! then 1-based lines). [`check`] reads every marker against the excerpts
//! that were sent, drops the ones that point at lines the model was never
//! shown or at lines with nothing in common with the sentence, and rewrites
//! the rest as numbered links (`[1](kanso-cite:1)`). A sentence left with no
//! surviving receipt is reported so it can be marked **Not found in your
//! notes**. The model cannot invent a citation that survives.
//!
//! Pure: no store, no clock, no model. Linear in the answer's length.
//! [`relocate`] finds a receipt's lines again when the note has changed.

use std::collections::{HashMap, HashSet};
use std::ops::Range;

use super::model::{
    CITE_PREFIX, Checked, NOT_FOUND_LABEL, Receipt, ShownExcerpt, UNVERIFIED_PREFIX,
};
use super::{content_words, stem};

/// The most lines one receipt may cover. A longer range is cut to the first
/// lines of it that were shown.
pub const MAX_RANGE: u32 = 15;

/// A marker as the model wrote it: excerpt number, first and last line
/// (1-based as written), and where it is in the answer.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Marker {
    excerpt: usize,
    first: u32,
    last: u32,
    at: Range<usize>,
}

/// A piece of the answer: plain text, or a marker.
#[derive(Clone, Debug)]
enum Piece {
    Text(Range<usize>),
    Marker(Marker),
}

/// What the checker made of a marker.
struct Cleared {
    excerpt: usize,
    note_id: String,
    title: String,
    updated: Option<String>,
    start: u32,
    end: u32,
    lines: Vec<(u32, String)>,
}

/// Checks `answer`'s receipts against `shown`, the excerpts (the first
/// prompt's and every tool result's) the model was sent.
pub fn check(answer: &str, shown: &[ShownExcerpt]) -> Checked {
    let trimmed = answer.trim();
    if is_not_in_notes(trimmed) {
        return Checked {
            not_in_notes: true,
            ..Checked::default()
        };
    }
    let code = kanso_core::links::code_ranges(answer);
    let pieces = split_pieces(answer, &code);
    let sentences = group_sentences(answer, &pieces, &code);

    let mut out = String::with_capacity(answer.len() + 32);
    let mut receipts: Vec<Receipt> = Vec::new();
    let mut unsupported = Vec::new();
    let mut claims = 0;
    let mut numbers: HashMap<(String, u32, u32), usize> = HashMap::new();

    for sentence in &sentences {
        let words = content_words(&sentence.plain(answer));
        let text_start = out.len();
        let mut survivors = 0;
        let mut had_marker = false;
        for piece in &sentence.pieces {
            match piece {
                Piece::Text(range) => out.push_str(&answer[range.clone()]),
                Piece::Marker(marker) => {
                    had_marker = true;
                    let cleared = clear(marker, shown)
                        .filter(|cleared| supports(&words, &sentence.plain(answer), cleared));
                    match cleared {
                        Some(cleared) => {
                            survivors += 1;
                            let key = (cleared.note_id.clone(), cleared.start, cleared.end);
                            let n = *numbers.entry(key).or_insert_with(|| {
                                receipts.push(Receipt {
                                    n: receipts.len() + 1,
                                    note_id: cleared.note_id.clone(),
                                    title: cleared.title.clone(),
                                    updated: cleared.updated.clone(),
                                    start: cleared.start,
                                    end: cleared.end,
                                    quote: cleared
                                        .lines
                                        .iter()
                                        .map(|(_, text)| text.as_str())
                                        .collect::<Vec<_>>()
                                        .join("\n"),
                                });
                                receipts.len()
                            });
                            out.push_str(&format!("[{n}]({CITE_PREFIX}{n})"));
                        }
                        None => {
                            // A dropped marker takes its space with it.
                            if out.ends_with(' ') {
                                out.pop();
                            }
                        }
                    }
                }
            }
        }
        if had_marker {
            claims += 1;
            if survivors == 0 {
                // The sentence's own words, without the space around them.
                let text = &out[text_start..];
                let lead = text.len() - text.trim_start().len();
                let trail = text.trim_end().len();
                if trail > lead {
                    unsupported.push(text_start + lead..text_start + trail);
                }
            }
        }
    }
    let markdown = neutralise_markers(&out);
    // Neutralising only inserts backslashes, which moves nothing before a
    // range's start by more than the insertions before it.
    let unsupported = if markdown.len() == out.len() {
        unsupported
    } else {
        remap(&out, &markdown, unsupported)
    };
    Checked {
        markdown,
        receipts,
        unsupported,
        not_in_notes: false,
        claims,
    }
}

/// Whether the whole answer is the sentinel, in any case and with the quotes,
/// backticks and full stop a model sometimes puts round it.
fn is_not_in_notes(trimmed: &str) -> bool {
    let bare = trimmed.trim_matches(|c: char| {
        c == '`' || c == '"' || c == '\'' || c == '.' || c == '*' || c.is_whitespace()
    });
    bare.eq_ignore_ascii_case(kanso_ai::NOT_IN_NOTES)
}

/// Escapes any `[n:Lx]` the removal of a marker left behind, so checking the
/// result again finds nothing new.
fn neutralise_markers(text: &str) -> String {
    let code = kanso_core::links::code_ranges(text);
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    while at < text.len() {
        let rest = &text[at..];
        if rest.starts_with('[') && !in_ranges(&code, at) && parse_marker(text, at).is_some() {
            out.push('\\');
        }
        let ch = rest.chars().next().expect("not at the end");
        out.push(ch);
        at += ch.len_utf8();
    }
    out
}

/// Maps ranges of `before` onto `after`, where `after` is `before` with
/// backslashes inserted.
fn remap(before: &str, after: &str, ranges: Vec<Range<usize>>) -> Vec<Range<usize>> {
    let mut map = vec![0usize; before.len() + 1];
    let (mut b, mut a) = (0, 0);
    let bytes_b = before.as_bytes();
    let bytes_a = after.as_bytes();
    while b < before.len() {
        if bytes_a[a] == b'\\' && bytes_b[b] != b'\\' {
            a += 1;
        }
        map[b] = a;
        b += 1;
        a += 1;
    }
    map[before.len()] = after.len();
    ranges
        .into_iter()
        .map(|r| map[r.start]..map[r.end])
        .collect()
}

fn in_ranges(ranges: &[Range<usize>], at: usize) -> bool {
    ranges.iter().any(|r| r.contains(&at))
}

/// Parses a marker that starts with `[` at byte `at`: `[2:L14]`,
/// `[1:L3-5]` (case and spaces forgiven). A marker after a backslash is the
/// person's text, not a marker.
fn parse_marker(text: &str, at: usize) -> Option<Marker> {
    let bytes = text.as_bytes();
    if bytes.get(at) != Some(&b'[') || (at > 0 && bytes[at - 1] == b'\\') {
        return None;
    }
    let mut i = at + 1;
    let skip = |i: &mut usize| {
        while bytes.get(*i).is_some_and(|b| *b == b' ' || *b == b'\t') {
            *i += 1;
        }
    };
    let number = |i: &mut usize| -> Option<u32> {
        let start = *i;
        while bytes.get(*i).is_some_and(u8::is_ascii_digit) {
            *i += 1;
        }
        if *i == start || *i - start > 6 {
            return None;
        }
        text[start..*i].parse().ok()
    };
    skip(&mut i);
    let excerpt = number(&mut i)?;
    skip(&mut i);
    if bytes.get(i) != Some(&b':') {
        return None;
    }
    i += 1;
    skip(&mut i);
    if !matches!(bytes.get(i), Some(b'L' | b'l')) {
        return None;
    }
    i += 1;
    let first = number(&mut i)?;
    skip(&mut i);
    let mut last = first;
    if matches!(bytes.get(i), Some(b'-' | 0xE2)) {
        // `-` or an en dash (E2 80 93).
        if bytes[i] == b'-' {
            i += 1;
        } else if text[i..].starts_with('\u{2013}') {
            i += '\u{2013}'.len_utf8();
        } else {
            return None;
        }
        skip(&mut i);
        if matches!(bytes.get(i), Some(b'L' | b'l')) {
            i += 1;
        }
        last = number(&mut i)?;
        skip(&mut i);
    }
    if bytes.get(i) != Some(&b']') {
        return None;
    }
    Some(Marker {
        excerpt: excerpt as usize,
        first,
        last,
        at: at..i + 1,
    })
}

/// The answer as text and markers, outside code.
fn split_pieces(answer: &str, code: &[Range<usize>]) -> Vec<Piece> {
    let mut pieces = Vec::new();
    let mut text_from = 0;
    let mut at = 0;
    while let Some(found) = answer[at..].find('[') {
        let start = at + found;
        if !in_ranges(code, start)
            && let Some(marker) = parse_marker(answer, start)
        {
            if start > text_from {
                pieces.push(Piece::Text(text_from..start));
            }
            at = marker.at.end;
            text_from = at;
            pieces.push(Piece::Marker(marker));
            continue;
        }
        at = start + 1;
    }
    if text_from < answer.len() {
        pieces.push(Piece::Text(text_from..answer.len()));
    }
    pieces
}

/// A sentence: text and the markers that follow or sit inside it.
struct Sentence {
    pieces: Vec<Piece>,
}

impl Sentence {
    /// The sentence's text without its markers.
    fn plain(&self, answer: &str) -> String {
        self.pieces
            .iter()
            .filter_map(|piece| match piece {
                Piece::Text(range) => Some(&answer[range.clone()]),
                Piece::Marker(_) => None,
            })
            .collect()
    }
}

/// Splits the pieces into sentences: at a full stop, question or exclamation
/// mark followed by a space or the end, and at a line end, outside code. The
/// markers right after the terminal mark belong to the sentence before it
/// (`Yearly. [1:L3]`).
fn group_sentences(answer: &str, pieces: &[Piece], code: &[Range<usize>]) -> Vec<Sentence> {
    let mut sentences: Vec<Sentence> = Vec::new();
    let mut current: Vec<Piece> = Vec::new();
    // After a terminal mark, only markers and spaces extend the sentence.
    let mut closing = false;
    for piece in pieces {
        match piece {
            Piece::Marker(_) => {
                current.push(piece.clone());
            }
            Piece::Text(range) => {
                let mut from = range.start;
                let text = &answer[range.clone()];
                let mut iter = text.char_indices().peekable();
                while let Some((offset, ch)) = iter.next() {
                    let at = range.start + offset;
                    let end = at + ch.len_utf8();
                    if closing && !ch.is_whitespace() {
                        // The sentence ended before this character.
                        if at > from {
                            current.push(Piece::Text(from..at));
                        }
                        sentences.push(Sentence {
                            pieces: std::mem::take(&mut current),
                        });
                        from = at;
                        closing = false;
                    }
                    if in_ranges(code, at) {
                        continue;
                    }
                    if ch == '\n' {
                        // A line end closes the sentence, line end included.
                        current.push(Piece::Text(from..end));
                        sentences.push(Sentence {
                            pieces: std::mem::take(&mut current),
                        });
                        from = end;
                        closing = false;
                        continue;
                    }
                    if matches!(ch, '.' | '?' | '!') {
                        let next = iter.peek().map(|(_, c)| *c);
                        if next.is_none_or(char::is_whitespace) {
                            closing = true;
                        }
                    }
                }
                if from < range.end {
                    current.push(Piece::Text(from..range.end));
                }
            }
        }
    }
    if !current.is_empty() {
        sentences.push(Sentence { pieces: current });
    }
    sentences
}

/// A marker cleared against what was shown: its lines are lines the model
/// saw, cut to [`MAX_RANGE`]. `None` when the excerpt or its lines were not
/// shown.
fn clear(marker: &Marker, shown: &[ShownExcerpt]) -> Option<Cleared> {
    if marker.first == 0 || marker.last < marker.first {
        return None;
    }
    let excerpt = shown.iter().find(|s| s.number == marker.excerpt)?;
    // The prompt counts from 1; notes count from 0.
    let (first, last) = (marker.first - 1, marker.last - 1);
    let wanted: Vec<&(u32, String)> = excerpt
        .lines
        .iter()
        .filter(|(n, _)| (first..=last).contains(n))
        .collect();
    let span = last - first + 1;
    let lines: Vec<(u32, String)> = if span <= MAX_RANGE {
        // Every line of a short range must have been shown: a gap or a line
        // past the end makes it invalid.
        if wanted.len() as u32 != span {
            return None;
        }
        wanted.into_iter().cloned().collect()
    } else {
        wanted
            .into_iter()
            .take(MAX_RANGE as usize)
            .cloned()
            .collect()
    };
    let (start, end) = (lines.first()?.0, lines.last()?.0);
    Some(Cleared {
        excerpt: marker.excerpt,
        note_id: excerpt.id.clone(),
        title: excerpt.title.clone(),
        updated: excerpt.updated.clone(),
        start,
        end,
        lines,
    })
}

/// The lexical support check: at least one content word of the sentence is
/// in the cited lines, or the cited lines hold a number that the sentence
/// holds too. Deliberately weak: it removes citations that point at
/// unrelated lines without punishing paraphrase. A sentence with nothing to
/// compare (no content word, no number) is given the benefit of the doubt.
fn supports(words: &[String], sentence: &str, cleared: &Cleared) -> bool {
    let cited: String = cleared
        .lines
        .iter()
        .map(|(_, text)| text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let numbers = numbers_in(sentence);
    if words.is_empty() && numbers.is_empty() {
        return true;
    }
    let _ = cleared.excerpt;
    let cited_words: HashSet<String> = cited
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| stem(w).to_owned())
        .collect();
    if words
        .iter()
        .any(|word| cited_words.contains(stem(&word.to_lowercase())))
    {
        return true;
    }
    let in_cited = numbers_in(&cited);
    numbers.iter().any(|n| in_cited.contains(n))
}

/// The numbers in `text` of two digits or more (a lone digit is in too many
/// lines to mean anything), with thousands commas dropped, and the days of
/// any dates.
fn numbers_in(text: &str) -> HashSet<String> {
    let mut numbers = HashSet::new();
    let mut digits = String::new();
    let flush = |digits: &mut String, numbers: &mut HashSet<String>| {
        if digits.len() >= 2 {
            numbers.insert(std::mem::take(digits));
        } else {
            digits.clear();
        }
    };
    let chars: Vec<char> = text.chars().collect();
    for (ix, ch) in chars.iter().enumerate() {
        if ch.is_ascii_digit() {
            digits.push(*ch);
        } else if *ch == ','
            && !digits.is_empty()
            && chars.get(ix + 1).is_some_and(char::is_ascii_digit)
        {
            // 1,200 is 1200.
        } else {
            flush(&mut digits, &mut numbers);
        }
    }
    flush(&mut digits, &mut numbers);
    numbers
}

/// Finds a receipt's lines again in `body`, which may have changed since the
/// answer was written: at the same lines first, then anywhere (exactly, then
/// ignoring differences in white space), the nearest to where they were.
/// `None` when they are gone. A quoted line that was cut when sent (ending in
/// `…`) matches the start of the line.
pub fn relocate(body: &str, receipt: &Receipt) -> Option<(u32, u32)> {
    let quote: Vec<&str> = receipt.quote.split('\n').collect();
    if receipt.quote.is_empty() || quote.is_empty() {
        return None;
    }
    let lines: Vec<&str> = body
        .split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .collect();
    let width = quote.len();
    if lines.len() < width {
        return None;
    }
    let matches_at = |at: usize, loose: bool| -> bool {
        quote.iter().enumerate().all(|(ix, q)| {
            let line = lines[at + ix];
            line_matches(line, q, loose)
        })
    };
    let original = receipt.start as usize;
    if original + width <= lines.len() && matches_at(original, false) {
        return Some((receipt.start, receipt.start + width as u32 - 1));
    }
    for loose in [false, true] {
        let best = (0..=lines.len() - width)
            .filter(|at| matches_at(*at, loose))
            .min_by_key(|at| at.abs_diff(original));
        if let Some(at) = best {
            return Some((at as u32, (at + width) as u32 - 1));
        }
    }
    None
}

fn line_matches(line: &str, quoted: &str, loose: bool) -> bool {
    let (quoted, cut) = match quoted.strip_suffix('\u{2026}') {
        Some(rest) => (rest, true),
        None => (quoted, false),
    };
    if loose {
        let (a, b) = (squash(line), squash(quoted));
        if cut { a.starts_with(&b) } else { a == b }
    } else if cut {
        line.starts_with(quoted)
    } else {
        line == quoted
    }
}

fn squash(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

impl Receipt {
    /// "line 3" or "lines 12–14", counted from 1 as people count.
    pub fn lines_label(&self) -> String {
        if self.end > self.start {
            format!("lines {}\u{2013}{}", self.start + 1, self.end + 1)
        } else {
            format!("line {}", self.start + 1)
        }
    }

    /// "Pricing · lines 12–14 · 14 Oct".
    pub fn source_line(&self) -> String {
        let title = if self.title.trim().is_empty() {
            "Untitled"
        } else {
            self.title.trim()
        };
        match self.updated.as_deref().and_then(short_date) {
            Some(date) => format!("{title} \u{b7} {} \u{b7} {date}", self.lines_label()),
            None => format!("{title} \u{b7} {}", self.lines_label()),
        }
    }
}

/// "2026-10-14" as "14 Oct".
pub fn short_date(iso: &str) -> Option<String> {
    let date = kanso_core::dates::Date::parse_iso(iso.trim().get(..10)?)?;
    Some(format!(
        "{} {}",
        date.day,
        &kanso_core::dates::MONTHS[date.month as usize - 1][..3]
    ))
}

impl Checked {
    /// The answer for display: every unsupported sentence is a link
    /// `[sentence](kanso-unverified:K)` that renderers draw as a muted
    /// dotted underline, followed by the label. A sentence that has markup
    /// of its own gets the label alone.
    pub fn display_markdown(&self) -> String {
        let mut out = String::with_capacity(self.markdown.len() + 64);
        let mut at = 0;
        for (k, range) in self.unsupported.iter().enumerate() {
            let sentence = &self.markdown[range.clone()];
            out.push_str(&self.markdown[at..range.start]);
            let plain = !sentence.contains(['[', ']', '`', '\n', '\\', '*', '_', '<', '>', '|']);
            if plain {
                out.push_str(&format!(
                    "[{}]({UNVERIFIED_PREFIX}{})",
                    sentence.trim(),
                    k + 1
                ));
            } else {
                out.push_str(sentence);
            }
            out.push_str(&format!(" *{NOT_FOUND_LABEL}*"));
            at = range.end;
        }
        out.push_str(&self.markdown[at..]);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn shown(number: usize, id: &str, title: &str, lines: &[(u32, &str)]) -> ShownExcerpt {
        ShownExcerpt {
            number,
            id: id.to_owned(),
            title: title.to_owned(),
            updated: Some("2026-10-14".to_owned()),
            lines: lines.iter().map(|(n, t)| (*n, (*t).to_owned())).collect(),
        }
    }

    fn pricing() -> Vec<ShownExcerpt> {
        vec![
            shown(
                1,
                "note:pricing",
                "Pricing",
                &[
                    (11, "We chose the yearly plan."),
                    (12, "40 dollars a year, billed once."),
                    (13, "Decided on 14 October."),
                    (30, "Sam wants a refund policy."),
                ],
            ),
            shown(
                2,
                "note:call",
                "Call with Sam",
                &[(2, "Sam argued the monthly plan churned too fast.")],
            ),
        ]
    }

    #[test]
    fn valid_receipts_become_numbered_links_in_order_of_first_use() {
        let answer = "We chose the yearly plan at 40 dollars [1:L12], after Sam argued the monthly plan churned too fast [2:L3]. The decision was made on 14 October [1:L14].";
        let checked = check(answer, &pricing());
        assert_eq!(
            checked.markdown,
            "We chose the yearly plan at 40 dollars [1](kanso-cite:1), after Sam argued the monthly plan churned too fast [2](kanso-cite:2). The decision was made on 14 October [3](kanso-cite:3)."
        );
        assert_eq!(checked.receipts.len(), 3);
        assert_eq!(checked.receipts[0].note_id, "note:pricing");
        assert_eq!(
            (checked.receipts[0].start, checked.receipts[0].end),
            (11, 11)
        );
        assert_eq!(checked.receipts[0].quote, "We chose the yearly plan.");
        assert_eq!(checked.receipts[1].note_id, "note:call");
        assert!(checked.unsupported.is_empty());
        assert_eq!(checked.claims, 2);
    }

    #[test]
    fn a_marker_after_the_full_stop_belongs_to_the_sentence_before() {
        let checked = check(
            "We chose the yearly plan. [1:L12] Sam wants a refund policy. [1:L31]",
            &pricing(),
        );
        assert_eq!(checked.receipts.len(), 2, "{checked:?}");
        assert!(checked.unsupported.is_empty(), "{checked:?}");
    }

    #[test]
    fn a_fabricated_receipt_is_removed_and_its_sentence_flagged() {
        // Excerpt 3 doesn't exist; line 99 of excerpt 1 was never shown;
        // line 20 is in a gap; L12-14 reaches the unshown line 14 (1-based)
        // wait: lines 11..13 are shown, so 12-14 (0-based 11..13) is fine.
        let answer = "The refund window is 30 days [3:L1]. Sam wants a refund policy [1:L99]. Customers may cancel at any time [1:L20]. We chose the yearly plan [1:L12-14].";
        let checked = check(answer, &pricing());
        let flagged: Vec<&str> = checked
            .unsupported
            .iter()
            .map(|r| &checked.markdown[r.clone()])
            .collect();
        assert_eq!(
            flagged,
            vec![
                "The refund window is 30 days.",
                "Sam wants a refund policy.",
                "Customers may cancel at any time."
            ]
        );
        assert_eq!(checked.receipts.len(), 1);
        assert!(checked.markdown.contains("yearly plan [1](kanso-cite:1)"));
        assert!(!checked.markdown.contains("L99"));
        assert!(checked.mostly_unchecked());
        let shown_md = checked.display_markdown();
        assert!(shown_md.contains("(kanso-unverified:1)"), "{shown_md}");
        assert!(shown_md.contains("*Not found in your notes*"));
    }

    #[test]
    fn a_receipt_on_unrelated_lines_is_dropped() {
        // The sentence is about refunds; the cited line is about the plan.
        let checked = check(
            "Customers get a full refund within thirty days [1:L12].",
            &pricing(),
        );
        assert!(checked.receipts.is_empty());
        assert_eq!(checked.unsupported.len(), 1);
    }

    #[test]
    fn a_shared_number_supports_a_paraphrase() {
        let checked = check("It costs 40 [1:L13].", &pricing());
        assert_eq!(checked.receipts.len(), 1, "{checked:?}");
    }

    #[test]
    fn a_sentence_without_markers_is_not_flagged() {
        let checked = check(
            "Your notes don't say who approved it. We chose the yearly plan [1:L12].",
            &pricing(),
        );
        assert!(checked.unsupported.is_empty());
        assert_eq!(checked.claims, 1);
    }

    #[test]
    fn the_sentinel_means_the_notes_do_not_answer() {
        for text in [
            "NOT_IN_NOTES",
            " not_in_notes \n",
            "`NOT_IN_NOTES`.",
            "**NOT_IN_NOTES**",
        ] {
            let checked = check(text, &pricing());
            assert!(checked.not_in_notes, "{text:?}");
            assert!(checked.markdown.is_empty());
        }
        assert!(!check("The NOT_IN_NOTES marker is a word.", &pricing()).not_in_notes);
    }

    #[test]
    fn a_long_range_is_cut_to_the_shown_lines() {
        let many: Vec<(u32, String)> = (0..40).map(|n| (n, format!("line {n} plan"))).collect();
        let excerpt = ShownExcerpt {
            number: 1,
            id: "note:x".into(),
            title: "X".into(),
            updated: None,
            lines: many,
        };
        let checked = check("The plan has many lines [1:L1-40].", &[excerpt]);
        assert_eq!(checked.receipts.len(), 1);
        assert_eq!(
            checked.receipts[0].end - checked.receipts[0].start + 1,
            MAX_RANGE
        );
    }

    #[test]
    fn markers_in_code_are_left_alone() {
        let checked = check("Use `[1:L12]` like so.\n\n```\n[1:L12]\n```\n", &pricing());
        assert!(checked.receipts.is_empty());
        assert!(checked.markdown.contains("`[1:L12]`"));
    }

    #[test]
    fn the_source_lines_read_as_the_sources_list_does() {
        let checked = check(
            "We chose the yearly plan [1:L12-14] and Sam argued [2:L3].",
            &pricing(),
        );
        let lines: Vec<String> = checked.receipts.iter().map(Receipt::source_line).collect();
        assert_eq!(lines[0], "Pricing \u{b7} lines 12\u{2013}14 \u{b7} 14 Oct");
        assert_eq!(lines[1], "Call with Sam \u{b7} line 3 \u{b7} 14 Oct");
    }

    #[test]
    fn relocating_finds_lines_that_moved_and_says_when_they_are_gone() {
        let receipt = Receipt {
            n: 1,
            note_id: "note:a".into(),
            title: "A".into(),
            updated: None,
            start: 1,
            end: 2,
            quote: "We chose yearly.\n40 dollars.".into(),
        };
        let same = "x\nWe chose yearly.\n40 dollars.\ny";
        assert_eq!(relocate(same, &receipt), Some((1, 2)));
        let moved = "new\nnew\nnew\nx\nWe chose yearly.\n40 dollars.\ny";
        assert_eq!(relocate(moved, &receipt), Some((4, 5)));
        let spaced = "x\n  We   chose yearly. \n40  dollars.";
        assert_eq!(relocate(spaced, &receipt), Some((1, 2)));
        assert_eq!(relocate("nothing alike", &receipt), None);
        let cut = Receipt {
            quote: "A very long line that was cut\u{2026}".into(),
            start: 0,
            end: 0,
            ..receipt
        };
        assert_eq!(
            relocate(
                "Intro\nA very long line that was cut and then goes on and on.",
                &cut
            ),
            Some((1, 1))
        );
    }

    fn shown_for_props() -> Vec<ShownExcerpt> {
        vec![
            shown(
                1,
                "note:a",
                "A",
                &[(0, "alpha beta"), (1, "gamma 42 delta"), (5, "epsilon")],
            ),
            shown(
                2,
                "note:b",
                "B",
                &[(3, "zeta eta theta"), (4, "iota 77 kappa")],
            ),
        ]
    }

    fn answer_strategy() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                "[a-z ]{0,30}",
                Just(". ".to_owned()),
                Just("\n".to_owned()),
                Just("alpha ".to_owned()),
                Just("42 ".to_owned()),
                "\\[[0-4]:L[0-9]{1,2}(-[0-9]{1,2})?\\]",
                Just("[".to_owned()),
                Just("`".to_owned()),
                Just("\\".to_owned()),
                Just("```\n".to_owned()),
            ],
            0..16,
        )
        .prop_map(|parts| parts.concat())
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// No receipt survives that points at an excerpt or a line that was
        /// not shown, and every survivor's quote is what was sent.
        #[test]
        fn prop_a_receipt_never_survives_for_an_unshown_line(answer in answer_strategy()) {
            let shown = shown_for_props();
            let checked = check(&answer, &shown);
            for receipt in &checked.receipts {
                let excerpt = shown.iter().find(|s| s.id == receipt.note_id);
                prop_assert!(excerpt.is_some());
                let excerpt = excerpt.unwrap();
                let sent: Vec<&str> = excerpt
                    .lines
                    .iter()
                    .filter(|(n, _)| (receipt.start..=receipt.end).contains(n))
                    .map(|(_, t)| t.as_str())
                    .collect();
                prop_assert!(!sent.is_empty());
                prop_assert!(receipt.quote.lines().count() <= MAX_RANGE as usize);
                prop_assert!(receipt.quote.lines().all(|l| sent.contains(&l)));
                prop_assert!(excerpt.lines.iter().any(|(n, _)| *n == receipt.start));
                prop_assert!(excerpt.lines.iter().any(|(n, _)| *n == receipt.end));
            }
            // Every link in the output names a receipt that exists.
            for n in 1..=checked.receipts.len() {
                let link = format!("]({CITE_PREFIX}{n})");
                prop_assert!(checked.markdown.contains(&link));
            }
            let beyond = format!("]({}{})", CITE_PREFIX, checked.receipts.len() + 1);
            prop_assert!(!checked.markdown.contains(&beyond));
            for range in &checked.unsupported {
                prop_assert!(checked.markdown.get(range.clone()).is_some());
            }
        }

        /// Checking what checking produced changes nothing: no marker is
        /// left to find, and the text is the same.
        #[test]
        fn prop_check_is_idempotent_on_its_output(answer in answer_strategy()) {
            let shown = shown_for_props();
            let once = check(&answer, &shown);
            let twice = check(&once.markdown, &shown);
            prop_assert_eq!(&twice.markdown, &once.markdown);
            prop_assert!(twice.receipts.is_empty());
            prop_assert!(twice.unsupported.is_empty());
        }

        /// A receipt's quote is found again after lines are inserted above
        /// it, at its new place.
        #[test]
        fn prop_relocate_finds_moved_lines(
            above in prop::collection::vec("[a-z]{0,12}", 0..8),
            inserted in prop::collection::vec("[a-z]{0,12}", 0..8),
            quoted in prop::collection::vec("[A-Z][a-z]{3,12}", 1..4),
        ) {
            let original: Vec<String> = above.iter().cloned().chain(quoted.iter().cloned()).collect();
            let start = above.len() as u32;
            let receipt = Receipt {
                n: 1, note_id: "n".into(), title: "T".into(), updated: None,
                start, end: start + quoted.len() as u32 - 1,
                quote: quoted.join("\n"),
            };
            let moved: Vec<String> = inserted.iter().cloned().chain(original.iter().cloned()).collect();
            let found = relocate(&moved.join("\n"), &receipt).expect("the quote is still there");
            let width = quoted.len();
            let at = found.0 as usize;
            prop_assert_eq!(found.1 as usize, at + width - 1);
            prop_assert_eq!(&moved[at..at + width], &quoted[..]);
        }
    }
}
