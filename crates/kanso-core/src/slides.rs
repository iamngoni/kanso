//! A note as slides, for presenting it.
//!
//! A line of three or more dashes after a blank line starts a new slide. The
//! blank line matters: `Title` over `---` is a heading in Markdown, not a
//! break. A note with no such line still presents: each `#` or `##` heading
//! starts a slide. Front matter is left out, and neither kind of line counts
//! inside a code block.
//!
//! Each slide is drawn as a document of its own, so link and footnote
//! definitions go on every slide that uses them, wherever the note keeps them.

use crate::{links, properties};

/// The slides of `body`, in order, with their blank edges trimmed. Empty
/// slides are left out, so a note with no text has none.
pub fn split(body: &str) -> Vec<String> {
    let text = properties::split(body).map_or(body, |front| &body[front.body_start..]);
    let (definitions, lines): (Vec<_>, Vec<_>) = lines_outside_code(text)
        .into_iter()
        .partition(|(line, in_code)| !in_code && definition_label(line).is_some());
    let mut slides = by_rules_or_headings(&lines);
    for slide in &mut slides {
        let used = slide.to_lowercase();
        for (line, _) in &definitions {
            if definition_label(line).is_some_and(|label| used.contains(&format!("[{label}]"))) {
                slide.push_str("\n\n");
                slide.push_str(line.trim_end());
            }
        }
    }
    slides
}

/// The label of the link or footnote definition on `line`, in lower case as
/// Markdown matches it: `[Docs]: https://…` is `docs`, `[^1]: …` is `^1`.
fn definition_label(line: &str) -> Option<String> {
    let rest = indented(line)?.strip_prefix('[')?;
    let (label, _) = rest.split_once("]:")?;
    (!label.is_empty() && !label.contains(['[', ']'])).then(|| label.to_lowercase())
}

fn by_rules_or_headings(lines: &[(&str, bool)]) -> Vec<String> {
    let breaks_slide =
        |ix: usize, line: &str| is_rule(line) && (ix == 0 || lines[ix - 1].0.trim().is_empty());
    let has_rule = lines
        .iter()
        .enumerate()
        .any(|(ix, (line, in_code))| !in_code && breaks_slide(ix, line));
    if has_rule {
        cut(lines, breaks_slide)
    } else {
        cut(lines, |_, line| is_main_heading(line))
    }
}

/// The slides `lines` make when each line `breaks` accepts starts a new one.
/// A rule that starts a slide isn't part of it; a heading is.
fn cut(lines: &[(&str, bool)], breaks: impl Fn(usize, &str) -> bool) -> Vec<String> {
    let mut slides = Vec::new();
    let mut current = String::new();
    for (ix, (line, in_code)) in lines.iter().enumerate() {
        if !in_code && breaks(ix, line) {
            push_slide(&mut slides, &current);
            current.clear();
            if is_rule(line) {
                continue;
            }
        }
        current.push_str(line);
    }
    push_slide(&mut slides, &current);
    slides
}

fn push_slide(slides: &mut Vec<String>, text: &str) {
    let text = text.trim_matches(['\n', '\r']).trim_end();
    if !text.trim().is_empty() {
        slides.push(text.to_owned());
    }
}

/// Each line of `text` with its line ending, and whether it belongs to a
/// fenced code block.
fn lines_outside_code(text: &str) -> Vec<(&str, bool)> {
    let mut fence: Option<(u8, usize)> = None;
    text.split_inclusive('\n')
        .map(|line| {
            let marker = fence_marker(line);
            match (fence, marker) {
                (None, Some((mark, len, _))) => {
                    fence = Some((mark, len));
                    (line, true)
                }
                (Some((open, need)), Some((mark, len, bare)))
                    if mark == open && len >= need && bare =>
                {
                    fence = None;
                    (line, true)
                }
                (Some(_), _) => (line, true),
                (None, None) => (line, false),
            }
        })
        .collect()
}

/// The fence `line` opens or closes, after any list markers: its character,
/// its length, and whether nothing follows it (only such a line closes a
/// block). A line of backticks with more backticks after it is inline code.
fn fence_marker(line: &str) -> Option<(u8, usize, bool)> {
    let trimmed = after_list_markers(line);
    let (mark, len) = links::fence_marker(trimmed)?;
    let rest = &trimmed[len..];
    (mark != b'`' || !rest.contains('`')).then(|| (mark, len, rest.trim().is_empty()))
}

/// `line` without its indent and any `-`, `*`, `+`, `1.` or `1)` before it.
fn after_list_markers(line: &str) -> &str {
    let mut text = line.trim_start();
    loop {
        let digits = text.bytes().take_while(u8::is_ascii_digit).count();
        let marker = match text.as_bytes().get(digits) {
            Some(b'.' | b')') if digits > 0 => digits + 1,
            Some(b'-' | b'*' | b'+') if digits == 0 => 1,
            _ => return text,
        };
        match text[marker..].strip_prefix(' ') {
            Some(rest) => text = rest.trim_start(),
            None => return text,
        }
    }
}

/// `line` without the up to three spaces Markdown allows before a block.
fn indented(line: &str) -> Option<&str> {
    let trimmed = line.trim_start_matches(' ');
    (line.len() - trimmed.len() <= 3).then_some(trimmed)
}

fn is_rule(line: &str) -> bool {
    let line = line.trim_end();
    indented(line).is_some_and(|rule| rule.len() >= 3 && rule.bytes().all(|b| b == b'-'))
}

fn is_main_heading(line: &str) -> bool {
    let Some(text) = indented(line) else {
        return false;
    };
    ["# ", "## "].iter().any(|mark| text.starts_with(mark))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn rules_after_a_blank_line_split_slides() {
        assert_eq!(
            split("One\n\n---\n\nTwo\n\n-----\nThree\n"),
            ["One", "Two", "Three"]
        );
    }

    #[test]
    fn a_rule_under_text_is_a_heading() {
        assert_eq!(split("Title\n---\nBody"), ["Title\n---\nBody"]);
    }

    #[test]
    fn front_matter_is_left_out() {
        assert_eq!(split("---\ntag: a\n---\nOne\n\n---\n\nTwo"), ["One", "Two"]);
    }

    #[test]
    fn rules_in_code_stay_in_the_slide() {
        let body = "```\n\n---\n\n```\n\n---\n\nTwo";
        assert_eq!(split(body), ["```\n\n---\n\n```", "Two"]);
    }

    #[test]
    fn a_longer_fence_needs_a_long_enough_close() {
        let body = "````\n```\n\n---\n\n````\n\n---\n\nTwo";
        assert_eq!(split(body), ["````\n```\n\n---\n\n````", "Two"]);
    }

    #[test]
    fn without_rules_main_headings_split_slides() {
        assert_eq!(
            split("Intro\n# One\nA\n## Two\nB\n### Kept\nC"),
            ["Intro", "# One\nA", "## Two\nB\n### Kept\nC"]
        );
    }

    #[test]
    fn rules_win_over_headings() {
        assert_eq!(
            split("# One\n# Two\n\n---\n\n# Three"),
            ["# One\n# Two", "# Three"]
        );
    }

    #[test]
    fn empty_slides_are_left_out() {
        assert_eq!(split("\n\n---\n\n---\n\n"), Vec::<String>::new());
        assert_eq!(split(""), Vec::<String>::new());
    }

    #[test]
    fn backticks_with_more_backticks_after_them_are_inline_code() {
        let body = "```inline``` is handy\n\n---\n\nTwo";
        assert_eq!(split(body), ["```inline``` is handy", "Two"]);
    }

    #[test]
    fn a_fence_opened_in_a_list_item_holds_its_code() {
        let body = "- ```sh\n  # comment\n  ```\n\n# Next";
        assert_eq!(split(body), ["- ```sh\n  # comment\n  ```", "# Next"]);
    }

    #[test]
    fn definitions_go_on_the_slides_that_use_them() {
        let body = "See [Docs].\n\n---\n\nNo links, one note[^1].\n\n---\n\nNeither.\n\n[docs]: https://example.com\n[^1]: Footnote.\n";
        assert_eq!(
            split(body),
            [
                "See [Docs].\n\n[docs]: https://example.com",
                "No links, one note[^1].\n\n[^1]: Footnote.",
                "Neither."
            ]
        );
    }

    #[test]
    fn a_definition_in_code_stays_where_it_is() {
        let body = "```\n[a]: b\n```\n\n---\n\nSee [a].";
        assert_eq!(split(body), ["```\n[a]: b\n```", "See [a]."]);
    }

    #[test]
    fn a_task_is_not_a_definition() {
        assert_eq!(split("- [x]: done"), ["- [x]: done"]);
    }

    #[test]
    fn crlf_line_endings_split_too() {
        assert_eq!(split("One\r\n\r\n---\r\n\r\nTwo\r\n"), ["One", "Two"]);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Whatever the text, no slide is blank, nothing panics, and every
        /// word of the slides comes from the note.
        #[test]
        fn prop_slides_hold_only_the_notes_text(
            body in "(([a-z#` ~-]{0,6})?\r?\n){0,20}",
        ) {
            for slide in split(&body) {
                prop_assert!(!slide.trim().is_empty());
                for line in slide.lines() {
                    prop_assert!(body.contains(line));
                }
            }
        }

        /// Slides joined by rules come back as those slides.
        #[test]
        fn prop_rules_round_trip(
            slides in prop::collection::vec("[a-z][a-z ]{0,8}[a-z]", 2..6),
        ) {
            let body = slides.join("\n\n---\n\n");
            prop_assert_eq!(split(&body), slides);
        }
    }
}
