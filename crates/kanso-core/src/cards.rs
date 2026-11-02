//! Flashcards written in a note as one line: `question :: answer`.
//!
//! A card is known by its [`card_key`]: the note and the words of its
//! question, so editing the answer keeps the card (and its review history)
//! and editing the question makes a new one. The note stays the only
//! source of truth for a card's words; this module only reads them back.
//!
//! The separator is ` :: ` with a space on both sides, so Dataview's
//! `key:: value`, `std::vec` and `fe80::1` are never cards. `\::` is a
//! literal `::`, which is how a line is kept from becoming a card and how
//! [`card_line`] writes a question that holds a `::`.

use std::ops::Range;

use crate::{links, properties, reminders::fnv1a};

/// The longest question or answer, in characters.
pub const MAX_SIDE: usize = 1_000;

/// The heading AI-suggested cards are written under.
pub const CARDS_HEADING: &str = "## Flashcards";

/// One card found in a note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardLine {
    /// The 0-based line of the note it is written on.
    pub line: u32,
    pub question: String,
    pub answer: String,
}

/// The cards in `body`, in order of where they are written.
///
/// Skipped: front matter, fenced and inline code, table rows, task lines
/// and HTML comments. A list marker or `> ` before the question is not
/// part of it. A line needs ` :: ` outside code with something on both
/// sides, neither longer than [`MAX_SIDE`].
pub fn card_lines(body: &str) -> Vec<CardLine> {
    let code = links::code_ranges(body);
    let comments = comment_ranges(body);
    let start = properties::split(body).map_or(0, |f| f.body_start);
    let mut out = Vec::new();
    let mut offset = 0usize;
    for (n, raw) in body.split_inclusive('\n').enumerate() {
        let line_start = offset;
        offset += raw.len();
        if line_start < start {
            continue;
        }
        let text = raw.trim_end_matches(['\n', '\r']);
        if in_any(&comments, line_start) || in_any(&code, line_start) {
            continue;
        }
        let Some((question, answer)) = split_line(text, line_start, &code) else {
            continue;
        };
        out.push(CardLine {
            line: n as u32,
            question,
            answer,
        });
    }
    out
}

fn in_any(ranges: &[Range<usize>], at: usize) -> bool {
    ranges.iter().any(|r| r.contains(&at))
}

/// Ranges of `<!-- … -->` comments (an unclosed one runs to the end).
fn comment_ranges(body: &str) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(open) = body[from..].find("<!--") {
        let open = from + open;
        let close = body[open + 4..]
            .find("-->")
            .map_or(body.len(), |c| open + 4 + c + 3);
        out.push(open..close);
        from = close;
        if from >= body.len() {
            break;
        }
    }
    out
}

/// The byte length of a leading blockquote and list marker.
fn marker_len(text: &str) -> usize {
    let mut cut = 0;
    let mut rest = text;
    loop {
        let t = rest.trim_start_matches([' ', '\t']);
        cut += rest.len() - t.len();
        rest = t;
        match rest.strip_prefix('>') {
            Some(r) => {
                cut += 1;
                rest = r;
            }
            None => break,
        }
    }
    for marker in ["- ", "* ", "+ "] {
        if rest.starts_with(marker) {
            return cut + marker.len();
        }
    }
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    if digits > 0 && rest[digits..].starts_with(". ") {
        return cut + digits + 2;
    }
    cut
}

fn split_line(text: &str, line_start: usize, code: &[Range<usize>]) -> Option<(String, String)> {
    if text.trim_start().starts_with('|') {
        return None;
    }
    let cut = marker_len(text);
    let rest = &text[cut..];
    if is_task(rest.trim_start()) {
        return None;
    }
    // The first ` :: ` outside code.
    let mut from = 0;
    let at = loop {
        let found = rest[from..].find(" :: ")? + from;
        let abs = line_start + cut + found;
        if !in_any(code, abs) && !in_any(code, abs + 1) && !in_any(code, abs + 3) {
            break found;
        }
        from = found + 1;
    };
    let question = unescape(rest[..at].trim());
    let answer = unescape(rest[at + 4..].trim());
    if question.is_empty()
        || answer.is_empty()
        || question.chars().count() > MAX_SIDE
        || answer.chars().count() > MAX_SIDE
    {
        return None;
    }
    Some((question, answer))
}

fn is_task(rest: &str) -> bool {
    let b = rest.as_bytes();
    b.len() >= 3 && b[0] == b'[' && b[2] == b']' && matches!(b[1], b' ' | b'x' | b'X')
}

/// `\::` reads as `::`.
fn unescape(text: &str) -> String {
    text.replace("\\::", "::")
}

/// `::` written as `\::`, so it cannot be read as a separator.
fn escape(text: &str) -> String {
    text.replace("::", "\\::")
}

/// The words a card's key is made of: lower case, whitespace collapsed.
pub fn normalize(question: &str) -> String {
    question
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// The key a card is known by on every device: the note and its question.
pub fn card_key(note_id: &str, question: &str) -> String {
    format!("{note_id}#{:016x}", fnv1a(&normalize(question)))
}

/// A card as a line of a note: `question :: answer`. Line breaks become
/// spaces and a `::` inside either side is escaped.
pub fn card_line(question: &str, answer: &str) -> String {
    let flat = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
    format!("{} :: {}", escape(&flat(question)), escape(&flat(answer)))
}

/// `body` with `cards` appended under a `## Flashcards` heading (made if
/// missing; reused if it is the last heading of the note).
pub fn append_cards(body: &str, cards: &[(String, String)]) -> String {
    if cards.is_empty() {
        return body.to_owned();
    }
    let lines: Vec<String> = cards.iter().map(|(q, a)| card_line(q, a)).collect();
    let mut out = body.trim_end_matches('\n').to_owned();
    let last_heading = body
        .lines()
        .filter(|l| l.starts_with('#'))
        .next_back()
        .map(str::trim_end);
    if last_heading == Some(CARDS_HEADING) {
        out.push('\n');
    } else {
        if !out.is_empty() {
            out.push_str("\n\n");
        }
        out.push_str(CARDS_HEADING);
        out.push_str("\n\n");
    }
    out.push_str(&lines.join("\n"));
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn qa(body: &str) -> Vec<(String, String)> {
        card_lines(body)
            .into_iter()
            .map(|c| (c.question, c.answer))
            .collect()
    }

    fn one(q: &str, a: &str) -> Vec<(String, String)> {
        vec![(q.to_owned(), a.to_owned())]
    }

    #[test]
    fn plain_list_and_quote_lines_are_cards() {
        assert_eq!(qa("What? :: That"), one("What?", "That"));
        assert_eq!(qa("- What? :: That"), one("What?", "That"));
        assert_eq!(qa("  * What? :: That"), one("What?", "That"));
        assert_eq!(qa("12. What? :: That"), one("What?", "That"));
        assert_eq!(qa("> What? :: That"), one("What?", "That"));
        let found = card_lines("a\n\nQ :: A\n");
        assert_eq!(found[0].line, 2);
    }

    #[test]
    fn lookalikes_are_not_cards() {
        for body in [
            "key:: value",
            "let v = std::vec::Vec::new();",
            "fe80::1 is a link-local address",
            "- [ ] task :: with answer",
            "- [x] task :: with answer",
            "| a :: b |",
            "empty ::  ",
            " :: no question",
            "Q \\:: A",
            "<!-- Q :: A -->",
            "`Q :: A`",
            "```\nQ :: A\n```",
            "---\ntitle: Q :: A\n---\n",
        ] {
            assert!(qa(body).is_empty(), "{body:?}");
        }
    }

    #[test]
    fn code_in_the_question_is_fine_but_not_in_the_separator() {
        assert_eq!(
            qa("What does `x` do? :: Nothing"),
            one("What does `x` do?", "Nothing")
        );
        assert_eq!(qa("A `b :: c` d :: e"), one("A `b :: c` d", "e"));
    }

    #[test]
    fn the_first_separator_wins_and_escapes_read_back() {
        assert_eq!(qa("a :: b :: c"), one("a", "b :: c"));
        assert_eq!(qa("a \\:: b :: c"), one("a :: b", "c"));
    }

    #[test]
    fn long_sides_are_not_cards() {
        let long = "x".repeat(MAX_SIDE + 1);
        assert!(qa(&format!("{long} :: a")).is_empty());
        assert!(qa(&format!("a :: {long}")).is_empty());
        assert_eq!(qa(&format!("{} :: a", "x".repeat(MAX_SIDE))).len(), 1);
    }

    #[test]
    fn keys_follow_the_question_only() {
        assert_eq!(
            card_key("note:a", "What?  Now"),
            card_key("note:a", "what? now")
        );
        assert_ne!(card_key("note:a", "What?"), card_key("note:b", "What?"));
        assert_ne!(card_key("note:a", "What?"), card_key("note:a", "Why?"));
    }

    #[test]
    fn appending_makes_or_reuses_the_heading() {
        let cards = vec![("Q".to_owned(), "A".to_owned())];
        let once = append_cards("# T\n\ntext\n", &cards);
        assert_eq!(once, "# T\n\ntext\n\n## Flashcards\n\nQ :: A\n");
        let twice = append_cards(&once, &[("R".into(), "S".into())]);
        assert_eq!(twice.matches("## Flashcards").count(), 1);
        assert_eq!(qa(&twice).len(), 2);
        let other = append_cards("## Flashcards\n\nx\n\n## After\n", &cards);
        assert_eq!(other.matches("## Flashcards").count(), 2);
    }

    // Plain prose: no code, no markers, no line breaks.
    fn side() -> impl Strategy<Value = String> {
        "[A-Za-z0-9?,.:\\\\ ]{1,40}".prop_filter("trimmed, not a marker", |s| {
            let t = s.trim();
            !t.is_empty()
                && !t.starts_with(['-', '*', '+', '>', '|', '[', '<', '#'])
                && !t.starts_with(|c: char| c.is_ascii_digit())
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// A card written by `card_line` parses back as the same words
        /// (with whitespace collapsed).
        #[test]
        fn prop_card_line_round_trips(q in side(), a in side()) {
            let line = card_line(&q, &a);
            let found = card_lines(&line);
            prop_assert_eq!(found.len(), 1, "{:?}", line);
            let flat = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
            prop_assert_eq!(&found[0].question, &flat(&q));
            prop_assert_eq!(&found[0].answer, &flat(&a));
        }

        /// Nothing inside a fence or an inline code span is a card.
        #[test]
        fn prop_no_card_inside_code(q in side(), a in side()) {
            let line = card_line(&q, &a);
            let fenced = format!("```\n{line}\n```\n");
            prop_assert!(card_lines(&fenced).is_empty());
            let tilde = format!("~~~text\n{line}\n~~~\n");
            prop_assert!(card_lines(&tilde).is_empty());
            let inline = format!("`{line}`");
            prop_assert!(card_lines(&inline).is_empty());
        }

        /// Editing the answer keeps the key.
        #[test]
        fn prop_card_key_ignores_answer(q in side(), a in side(), b in side()) {
            let ka = &card_lines(&card_line(&q, &a))[0].question;
            let kb = &card_lines(&card_line(&q, &b))[0].question;
            prop_assert_eq!(card_key("note:x", ka), card_key("note:x", kb));
        }

        /// Reading never panics and never yields an empty or oversize side.
        #[test]
        fn prop_card_lines_total(body in "\\PC{0,200}") {
            for c in card_lines(&body) {
                prop_assert!(!c.question.is_empty() && !c.answer.is_empty());
                prop_assert!(c.question.chars().count() <= MAX_SIDE);
            }
        }
    }
}
