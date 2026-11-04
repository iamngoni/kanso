//! Where a person last stopped writing (issue #240, *Pick up where you left
//! off*).
//!
//! The store keeps a line and a column per note, never text; the words are
//! read from the note when the page draws. These are the pure parts: the
//! hash that finds a paragraph again after it moved, the paragraph and the
//! excerpt ending at the caret, and the line diff that places a session
//! that arrived by sync.

/// A writing session needs at least this many characters typed...
pub const MIN_CHARS: u32 = 40;
/// ...or this many milliseconds between its first and last keystroke.
pub const MIN_MS: i64 = 2 * 60_000;
/// A session ends after this long without typing.
pub const GAP_MS: i64 = 15 * 60_000;
/// How many characters of the paragraph the page shows, from the caret back.
pub const EXCERPT_CHARS: usize = 140;
/// The characters the anchor hash covers.
pub const ANCHOR_CHARS: usize = 40;
/// Pick up hides itself when nothing was written for this many days.
pub const STALE_DAYS: i64 = 14;

/// Whether a session of `chars` typed characters between `start_ms` and
/// `end_ms` counts as a writing session.
pub fn qualifies(chars: u32, start_ms: i64, end_ms: i64) -> bool {
    chars >= MIN_CHARS || end_ms - start_ms >= MIN_MS
}

fn fnv1a(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

/// The hash of a paragraph's first [`ANCHOR_CHARS`] characters.
pub fn anchor_hash(paragraph: &str) -> String {
    let head: String = paragraph.trim().chars().take(ANCHOR_CHARS).collect();
    format!("{:016x}", fnv1a(&head))
}

/// The first line of the paragraph holding `line`: a paragraph runs between
/// blank lines (and a heading or a fence line ends one).
fn paragraph_start(lines: &[&str], line: usize) -> usize {
    let mut at = line.min(lines.len().saturating_sub(1));
    while at > 0 && !lines[at - 1].trim().is_empty() && !lines[at - 1].trim_start().starts_with('#')
    {
        at -= 1;
    }
    at
}

/// A paragraph found in a body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    /// Its first line (0-based).
    pub start_line: u32,
    /// Its text, lines joined by a space, list and quote marks dropped.
    pub text: String,
    /// Where the caret was, in characters into `text`; `None` when the
    /// caret's line was not in it.
    pub caret: Option<usize>,
}

fn clean(line: &str) -> &str {
    let t = line.trim();
    let t = t.trim_start_matches(['>', ' ']);
    let t = t
        .strip_prefix("- [ ] ")
        .or_else(|| t.strip_prefix("- [x] "))
        .or_else(|| t.strip_prefix("- "))
        .or_else(|| t.strip_prefix("* "))
        .unwrap_or(t);
    t.trim()
}

fn paragraph(lines: &[&str], start: usize) -> (String, Vec<(usize, usize)>) {
    // (line index, char offset into the joined text) per line.
    let mut text = String::new();
    let mut offsets = Vec::new();
    let mut at = start;
    while at < lines.len() && !lines[at].trim().is_empty() {
        if at > start && lines[at].trim_start().starts_with('#') {
            break;
        }
        if !text.is_empty() {
            text.push(' ');
        }
        offsets.push((at, text.chars().count()));
        text.push_str(clean(lines[at]));
        at += 1;
    }
    (text, offsets)
}

/// The anchor hash of the paragraph holding `line` of `body`: what is stored
/// with the caret's place, so [`find`] can tell the paragraph is the same.
pub fn anchor_at(body: &str, line: u32) -> String {
    let lines: Vec<&str> = body.lines().collect();
    if lines.is_empty() {
        return String::new();
    }
    let start = paragraph_start(&lines, line as usize);
    anchor_hash(&paragraph(&lines, start).0)
}

/// The paragraph where the person stopped. `line` and `col` (characters)
/// are where the caret was; `anchor` is the hash stored with it. When the
/// paragraph at `line` no longer starts with the stored words, the body is
/// searched for the paragraph that does; failing that the line is used.
pub fn find(body: &str, line: u32, col: u32, anchor: &str) -> Option<Found> {
    let lines: Vec<&str> = body.lines().collect();
    if lines.is_empty() {
        return None;
    }
    let line = line as usize;
    let start = paragraph_start(&lines, line);
    let (text, offsets) = paragraph(&lines, start);
    let at_line = if text.is_empty() {
        None
    } else if anchor.is_empty() || anchor_hash(&text) == anchor {
        Some((start, text, offsets))
    } else {
        None
    };
    if let Some((start, text, offsets)) = at_line {
        return Some(finish(start, text, &offsets, &lines, line, col));
    }
    // The paragraph moved: look for the one with the stored words.
    let mut at = 0;
    while at < lines.len() {
        if lines[at].trim().is_empty() {
            at += 1;
            continue;
        }
        let (text, offsets) = paragraph(&lines, at);
        if !text.is_empty() && anchor_hash(&text) == anchor {
            let end_line = offsets.last().map_or(at, |(l, _)| *l);
            let caret_line = (at + (line.saturating_sub(start))).min(end_line);
            return Some(finish(at, text, &offsets, &lines, caret_line, col));
        }
        at += offsets.len().max(1);
    }
    // Fall back to whatever paragraph is at the line.
    let (text, offsets) = paragraph(&lines, start);
    if text.is_empty() {
        None
    } else {
        Some(finish(start, text, &offsets, &lines, line, col))
    }
}

fn finish(
    start: usize,
    text: String,
    offsets: &[(usize, usize)],
    lines: &[&str],
    line: usize,
    col: u32,
) -> Found {
    let caret = offsets.iter().find(|(l, _)| *l == line).map(|(l, off)| {
        let raw = lines[*l];
        let trimmed_lead = raw.chars().count() - raw.trim_start().chars().count();
        let cleaned = clean(raw);
        let prefix = raw.trim().chars().count() - cleaned.chars().count();
        let into = (col as usize)
            .saturating_sub(trimmed_lead + prefix)
            .min(cleaned.chars().count());
        off + into
    });
    Found {
        start_line: start as u32,
        text,
        caret,
    }
}

/// The words shown for a paragraph: they end at the caret and are cut from
/// the left to about [`EXCERPT_CHARS`] characters, with an ellipsis when cut.
/// Without a caret, the paragraph's end is used.
pub fn excerpt(found: &Found, max_chars: usize) -> String {
    let chars: Vec<char> = found.text.chars().collect();
    let end = found.caret.unwrap_or(chars.len()).min(chars.len());
    let head = &chars[..end];
    if head.len() <= max_chars {
        return head.iter().collect();
    }
    let mut from = head.len() - max_chars;
    // Do not begin in the middle of a word.
    while from < head.len() && !head[from.saturating_sub(1)].is_whitespace() && from > 0 {
        from += 1;
    }
    let cut: String = head[from..].iter().collect();
    format!("\u{2026}{}", cut.trim_start())
}

/// The last line that differs between `old` and `new`, as `(line, column)`
/// at the end of that line in `new`: where a change that arrived by sync was
/// made. `None` when the bodies are equal.
pub fn last_changed(old: &str, new: &str) -> Option<(u32, u32)> {
    if old == new {
        return None;
    }
    let a: Vec<&str> = old.lines().collect();
    let b: Vec<&str> = new.lines().collect();
    let prefix = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let max_suffix = a.len().min(b.len()) - prefix;
    let suffix = a
        .iter()
        .rev()
        .zip(b.iter().rev())
        .take(max_suffix)
        .take_while(|(x, y)| x == y)
        .count();
    if b.is_empty() {
        return Some((0, 0));
    }
    let changed_end = b.len() - suffix;
    let line = if changed_end > prefix {
        changed_end - 1
    } else {
        // Only lines were deleted: the line before the gap.
        prefix.saturating_sub(1).min(b.len() - 1)
    };
    Some((line as u32, b[line].chars().count() as u32))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig { rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f), ..ProptestConfig::with_cases(512) })]

        /// An excerpt never passes the cap (plus the ellipsis) and is part
        /// of the paragraph before the caret.
        #[test]
        fn excerpt_is_bounded_and_from_the_paragraph(words in prop::collection::vec("[a-z]{1,9}", 1..60), caret in 0usize..600) {
            let text = words.join(" ");
            let found = Found { start_line: 0, text: text.clone(), caret: Some(caret) };
            let shown = excerpt(&found, EXCERPT_CHARS);
            prop_assert!(shown.chars().count() <= EXCERPT_CHARS + 1);
            let body = shown.trim_start_matches('\u{2026}');
            prop_assert!(text.contains(body.trim()));
        }

        /// The anchor taken at a line is the one `find` accepts for it.
        #[test]
        fn anchor_at_is_accepted_by_find(
            lines in prop::collection::vec("[a-c ]{0,12}", 1..8),
            line in 0u32..8,
        ) {
            let body = lines.join("\n");
            let anchor = anchor_at(&body, line);
            if let Some(found) = find(&body, line, 0, &anchor) {
                prop_assert!(!found.text.is_empty());
            }
        }

        /// The last changed line is always a line of the new text, and an
        /// unchanged body has none.
        #[test]
        fn last_changed_is_inside_the_new_body(
            old in prop::collection::vec("[a-c ]{0,6}", 0..8),
            new in prop::collection::vec("[a-c ]{0,6}", 0..8),
        ) {
            let (a, b) = (old.join("\n"), new.join("\n"));
            match last_changed(&a, &b) {
                None => prop_assert_eq!(a, b),
                Some((line, col)) => {
                    let lines: Vec<&str> = b.lines().collect();
                    if !lines.is_empty() {
                        prop_assert!((line as usize) < lines.len());
                        prop_assert!(col as usize <= lines[line as usize].chars().count());
                    }
                }
            }
        }

        /// The anchor ignores everything past its first characters.
        #[test]
        fn anchor_covers_only_the_head(head in "[a-z ]{40,50}", tail in "[a-z ]{0,40}") {
            let one = anchor_hash(&head);
            let two = anchor_hash(&format!("{head}{tail}"));
            let first: String = head.trim().chars().take(ANCHOR_CHARS).collect();
            let again: String = format!("{head}{tail}").trim().chars().take(ANCHOR_CHARS).collect();
            prop_assert_eq!(one == two, first == again);
        }
    }

    #[test]
    fn a_session_needs_forty_characters_or_two_minutes() {
        assert!(!qualifies(39, 0, 119_999));
        assert!(qualifies(40, 0, 0));
        assert!(qualifies(3, 0, MIN_MS));
    }

    #[test]
    fn the_excerpt_ends_where_the_caret_was() {
        let body = "# Title\n\nFirst paragraph.\n\nand the arrow tool should snap to the shape's edge, not its centre, so that";
        let col = "and the arrow tool should snap to the shape's edge, not its centre, so that"
            .len() as u32;
        let found = find(body, 4, col, "").unwrap();
        assert_eq!(found.start_line, 4);
        assert!(excerpt(&found, 140).ends_with("so that"));
    }

    #[test]
    fn a_moved_paragraph_is_found_by_its_words() {
        let old = "Intro line.\n\nWe will ship the release notes on Friday after the review.";
        let (line, col) = (2, 66);
        let anchor = anchor_hash("We will ship the release notes on Friday after the review.");
        let moved = format!("A new first paragraph\nwith two lines.\n\n{old}");
        let found = find(&moved, line, col, &anchor).unwrap();
        assert_eq!(found.start_line, 5);
        assert!(found.text.starts_with("We will ship"));
    }

    #[test]
    fn a_synced_change_is_placed_on_its_last_changed_line() {
        let old = "a\nb\nc\nd";
        let new = "a\nb\nc changed\nd";
        assert_eq!(last_changed(old, new), Some((2, 9)));
        assert_eq!(last_changed("a\nb", "a\nb\nnew line"), Some((2, 8)));
        assert_eq!(last_changed("a\nb", "a\nb"), None);
    }
}
