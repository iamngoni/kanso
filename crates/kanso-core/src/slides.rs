//! A note as slides, for presenting it.
//!
//! A rule starts a new slide. `***` and `___` are rules wherever they are; a
//! line of dashes is one after a blank line or a heading, since `Title` over
//! `---` is a heading in Markdown, not a break. A note with no rule still
//! presents: each heading of level 1 or 2 starts a slide. Front matter is left
//! out, and neither kind of line counts inside a code block. The phones cut
//! their slides from the parsed blocks by the same rules.
//!
//! Each slide is drawn as a document of its own, so link and footnote
//! definitions go on every slide that uses them, wherever the note keeps them.
//! One that runs over several lines stays where it is.

use crate::{links, properties};

/// The slides of `body`, in order, with their blank edges trimmed. Empty
/// slides are left out, so a note with no text has none.
pub fn split(body: &str) -> Vec<String> {
    let text = properties::split(body).map_or(body, |front| &body[front.body_start..]);
    let marked = lines_outside_code(text);
    let (definitions, lines): (Vec<_>, Vec<_>) = marked
        .iter()
        .enumerate()
        .map(|(ix, line)| (ix, *line))
        .partition(|(ix, (line, in_code))| {
            !in_code && definition_label(line).is_some() && !continues(&marked[ix + 1..])
        });
    let lines: Vec<_> = lines.into_iter().map(|(_, line)| line).collect();
    let mut slides = by_rules_or_headings(&lines);
    for slide in &mut slides {
        let used = slide.to_lowercase();
        for (_, (line, _)) in &definitions {
            if definition_label(line).is_some_and(|label| used.contains(&format!("[{label}]"))) {
                slide.push_str("\n\n");
                slide.push_str(line.trim_end());
            }
        }
    }
    slides
}

/// The label of the link or footnote definition on `line`, in lower case as
/// Markdown matches it: `[Docs]: https://…` is `docs`, `[^1]: …` is `^1`. A
/// line that only looks like one, such as `[Note]: check this`, is text.
fn definition_label(line: &str) -> Option<String> {
    let rest = indented(line)?.strip_prefix('[')?;
    let (label, target) = rest.split_once("]:")?;
    let valid = !label.is_empty()
        && !label.contains(['[', ']'])
        && (label.starts_with('^') || is_link_target(target));
    valid.then(|| label.to_lowercase())
}

/// Whether `target` is a destination, with a quoted title or none.
fn is_link_target(target: &str) -> bool {
    let target = target.trim();
    let (destination, title) = target
        .split_once(char::is_whitespace)
        .unwrap_or((target, ""));
    let title = title.trim();
    !destination.is_empty() && (title.is_empty() || title.starts_with(['"', '\'', '(']))
}

/// Whether the next line with text in `after` is indented, so it belongs to a
/// definition just above it.
fn continues(after: &[(&str, bool)]) -> bool {
    after
        .iter()
        .find(|(line, _)| !line.trim().is_empty())
        .is_some_and(|(line, in_code)| !in_code && line.starts_with(['\t', ' ']))
}

fn by_rules_or_headings(lines: &[(&str, bool)]) -> Vec<String> {
    let after_break = |ix: usize| {
        ix == 0 || {
            let before = lines[ix - 1].0;
            before.trim().is_empty() || is_hash_heading(before)
        }
    };
    let breaks_slide = |ix: usize, line: &str| {
        is_rule(line) && (!line.trim_start().starts_with('-') || after_break(ix))
    };
    let has_rule = lines
        .iter()
        .enumerate()
        .any(|(ix, (line, in_code))| !in_code && breaks_slide(ix, line));
    if has_rule {
        cut(lines, breaks_slide)
    } else {
        cut(lines, |ix, line| {
            is_main_heading(line) || is_underlined(line, lines.get(ix + 1))
        })
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
        .map(|line| match fence {
            Some((mark, need)) => {
                if closes_fence(line, mark, need) {
                    fence = None;
                }
                (line, true)
            }
            None => {
                fence = opening_fence(line);
                (line, fence.is_some())
            }
        })
        .collect()
}

/// The character and length of the fence `line` opens, after any list markers.
/// A line of backticks with more backticks after it is inline code, and one
/// indented four spaces with no list marker is an indented code block.
fn opening_fence(line: &str) -> Option<(u8, usize)> {
    let trimmed = line.trim_start();
    let text = after_list_markers(trimmed);
    if indented(line).is_none() && text.len() == trimmed.len() {
        return None;
    }
    let (mark, len) = links::fence_marker(text)?;
    (mark != b'`' || !text[len..].contains('`')).then_some((mark, len))
}

/// Whether `line` closes a fence of `mark` that is `need` long: at least as
/// long, and with nothing after it.
fn closes_fence(line: &str, mark: u8, need: usize) -> bool {
    let text = line.trim_start();
    links::fence_marker(text)
        .is_some_and(|(found, len)| found == mark && len >= need && text[len..].trim().is_empty())
}

/// `text` without any `-`, `*`, `+`, `1.` or `1)` before it.
fn after_list_markers(mut text: &str) -> &str {
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
    let Some(rule) = indented(line.trim_end()) else {
        return false;
    };
    rule.len() >= 3
        && matches!(rule.as_bytes()[0], b'-' | b'*' | b'_')
        && rule.bytes().all(|b| b == rule.as_bytes()[0])
}

fn is_main_heading(line: &str) -> bool {
    let Some(text) = indented(line) else {
        return false;
    };
    ["# ", "## "].iter().any(|mark| text.starts_with(mark))
}

fn is_hash_heading(line: &str) -> bool {
    indented(line).is_some_and(|text| {
        let level = text.bytes().take_while(|b| *b == b'#').count();
        (1..=6).contains(&level) && text[level..].starts_with([' ', '\t', '\r', '\n'])
    })
}

/// Whether `line` is a heading of the kind written with a line of `=` or `-`
/// under it, which `next` is.
fn is_underlined(line: &str, next: Option<&(&str, bool)>) -> bool {
    let Some((next, false)) = next.copied() else {
        return false;
    };
    let under = next.trim();
    !line.trim().is_empty()
        && !under.is_empty()
        && (under.bytes().all(|b| b == b'=') || under.bytes().all(|b| b == b'-'))
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
    fn prose_shaped_like_a_definition_stays_on_its_slide() {
        let body = "[Note]: check the figures first\n\n---\n\nTwo";
        assert_eq!(split(body), ["[Note]: check the figures first", "Two"]);
    }

    #[test]
    fn a_titled_definition_moves() {
        let body = "See [a].\n\n---\n\nTwo\n\n[a]: https://x.y \"Title\"";
        assert_eq!(
            split(body),
            ["See [a].\n\n[a]: https://x.y \"Title\"", "Two"]
        );
    }

    #[test]
    fn a_definition_over_several_lines_stays_where_it_is() {
        let body = "One[^1].\n\n---\n\nTwo\n\n[^1]: First\n\n    More";
        assert_eq!(split(body), ["One[^1].", "Two\n\n[^1]: First\n\n    More"]);
    }

    #[test]
    fn a_list_marker_in_a_code_block_does_not_close_it() {
        let body = "```md\n- ```\n\n---\n\n```\n\n---\n\nTwo";
        assert_eq!(split(body), ["```md\n- ```\n\n---\n\n```", "Two"]);
    }

    #[test]
    fn a_fence_in_an_indented_code_block_opens_nothing() {
        let body = "    ```\n\n---\n\nTwo";
        assert_eq!(split(body), ["    ```", "Two"]);
    }

    #[test]
    fn stars_and_underscores_are_rules_wherever_they_are() {
        assert_eq!(
            split("One\n***\nTwo\n\n___\n\nThree"),
            ["One", "Two", "Three"]
        );
    }

    #[test]
    fn dashes_after_a_heading_are_a_rule() {
        assert_eq!(split("# Title\n---\nBody"), ["# Title", "Body"]);
    }

    #[test]
    fn underlined_headings_start_slides() {
        assert_eq!(
            split("One\n===\nA\n\nTwo\n---\nB"),
            ["One\n===\nA", "Two\n---\nB"]
        );
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
