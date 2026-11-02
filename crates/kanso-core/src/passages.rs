//! A note cut into passages: runs of lines inside one note, each knowing the
//! lines it spans and the heading above it. Ask sends passages with their
//! line numbers, so an answer can point at the exact lines it rests on.
//!
//! Pure: no I/O, no store. Lines are 0-based, as `note_links.line` and
//! `Backlink::line` are, and are counted in the whole body (front matter
//! included), so a line here is a line in the editor.

use crate::properties;

/// The most characters a passage grows to before it is split at a blank line.
pub const PASSAGE_MAX: usize = 1_200;

/// A run of lines inside one note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Passage {
    /// The first line, 0-based.
    pub start: u32,
    /// The last line, 0-based and inclusive.
    pub end: u32,
    /// The nearest heading at or above the passage, without its `#` marks.
    pub heading: Option<String>,
    /// The lines `start..=end` joined with `\n`.
    pub text: String,
}

impl Passage {
    /// The number of lines it spans.
    pub fn lines(&self) -> u32 {
        self.end - self.start + 1
    }
}

/// The passages of `body`, in order. A new passage starts at every heading;
/// a section longer than [`PASSAGE_MAX`] is split further at blank lines
/// (and, for one unbroken paragraph, at line ends). Front matter is skipped
/// and a fenced block is never split or read for headings. Every non-blank
/// line outside front matter is in exactly one passage; blank lines between
/// passages are in none.
pub fn passages(body: &str) -> Vec<Passage> {
    let lines: Vec<&str> = body
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .collect();
    let first = properties::split(body)
        .map(|front| body[..front.body_start].matches('\n').count())
        .unwrap_or(0);

    // For each line: is it in a fence, and is it a heading (outside one).
    let mut in_fence = vec![false; lines.len()];
    let mut heading_of: Vec<Option<String>> = vec![None; lines.len()];
    let mut fence: Option<(char, usize)> = None;
    for (ix, line) in lines.iter().enumerate().skip(first) {
        let trimmed = line.trim_start();
        match (fence, fence_marker(trimmed)) {
            (None, Some(marker)) => {
                fence = Some(marker);
                in_fence[ix] = true;
            }
            (Some((ch, len)), Some((c, l))) if c == ch && l >= len && closes(trimmed) => {
                fence = None;
                in_fence[ix] = true;
            }
            (Some(_), _) => in_fence[ix] = true,
            (None, None) => {
                heading_of[ix] = heading_text(line);
            }
        }
    }

    // Sections: split before every heading line.
    let mut sections: Vec<(usize, usize, Option<String>)> = Vec::new();
    let mut current: Option<String> = None;
    let mut start = first;
    for (ix, heading) in heading_of.iter().enumerate().take(lines.len()).skip(first) {
        if let Some(heading) = heading {
            if ix > start {
                sections.push((start, ix - 1, current.clone()));
            }
            start = ix;
            current = Some(heading.clone());
        }
    }
    if lines.len() > start {
        sections.push((start, lines.len() - 1, current.clone()));
    }

    let mut out = Vec::new();
    for (from, to, heading) in sections {
        split_section(&lines, &in_fence, (from, to), &heading, &mut out);
    }
    out
}

fn split_section(
    lines: &[&str],
    in_fence: &[bool],
    (from, to): (usize, usize),
    heading: &Option<String>,
    out: &mut Vec<Passage>,
) {
    // Paragraphs: runs of non-blank lines (a fence is one paragraph).
    let mut paragraphs: Vec<(usize, usize)> = Vec::new();
    let mut ix = from;
    while ix <= to {
        if lines[ix].trim().is_empty() && !in_fence[ix] {
            ix += 1;
            continue;
        }
        let begin = ix;
        while ix <= to && (!lines[ix].trim().is_empty() || in_fence[ix]) {
            ix += 1;
        }
        paragraphs.push((begin, ix - 1));
    }
    let push = |s: usize, e: usize, out: &mut Vec<Passage>| {
        out.push(Passage {
            start: s as u32,
            end: e as u32,
            heading: heading.clone(),
            text: lines[s..=e].join("\n"),
        });
    };
    // The passage being built: first line, last line, characters.
    let mut group: Option<(usize, usize, usize)> = None;
    for (begin, end) in paragraphs {
        let size: usize = lines[begin..=end]
            .iter()
            .map(|l| l.chars().count() + 1)
            .sum();
        if size > PASSAGE_MAX && !in_fence[begin] {
            // One unbroken paragraph that is too long: split at line ends.
            if let Some((s, e, _)) = group.take() {
                push(s, e, out);
            }
            let mut s = begin;
            let mut chars = 0;
            for (l, line) in lines.iter().enumerate().take(end + 1).skip(begin) {
                let n = line.chars().count() + 1;
                if chars > 0 && chars + n > PASSAGE_MAX {
                    push(s, l - 1, out);
                    s = l;
                    chars = 0;
                }
                chars += n;
            }
            group = Some((s, end, chars));
            continue;
        }
        match group {
            Some((s, _, chars)) if chars + size <= PASSAGE_MAX => {
                group = Some((s, end, chars + size));
            }
            _ => {
                if let Some((s, e, _)) = group.take() {
                    push(s, e, out);
                }
                group = Some((begin, end, size));
            }
        }
    }
    if let Some((s, e, _)) = group {
        push(s, e, out);
    }
}

/// `(character, length)` of a code fence marker that opens at the start of
/// `trimmed`.
fn fence_marker(trimmed: &str) -> Option<(char, usize)> {
    let ch = trimmed.chars().next().filter(|c| matches!(c, '`' | '~'))?;
    let len = trimmed.chars().take_while(|c| *c == ch).count();
    (len >= 3).then_some((ch, len))
}

/// Whether a fence line has nothing after its marks (so it can close).
fn closes(trimmed: &str) -> bool {
    trimmed.trim_start_matches(['`', '~']).trim().is_empty()
}

/// The text of an ATX heading line, `None` for any other line.
fn heading_text(line: &str) -> Option<String> {
    let stripped = line.trim_start_matches(' ');
    if line.len() - stripped.len() > 3 {
        return None;
    }
    let hashes = stripped.chars().take_while(|c| *c == '#').count();
    if !(1..=6).contains(&hashes) {
        return None;
    }
    let rest = &stripped[hashes..];
    if !(rest.is_empty() || rest.starts_with([' ', '\t'])) {
        return None;
    }
    let text = rest.trim().trim_end_matches('#').trim();
    Some(text.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn a_heading_starts_a_passage_and_names_the_ones_below_it() {
        let body =
            "intro\n\n# Pricing\nWe chose yearly.\n\nIt is 40.\n\n## Churn\nMonthly churned.\n";
        let found = passages(body);
        let shape: Vec<_> = found
            .iter()
            .map(|p| (p.start, p.end, p.heading.as_deref()))
            .collect();
        assert_eq!(
            shape,
            vec![(0, 0, None), (2, 5, Some("Pricing")), (7, 8, Some("Churn")),]
        );
        assert_eq!(found[1].text, "# Pricing\nWe chose yearly.\n\nIt is 40.");
    }

    #[test]
    fn front_matter_is_skipped_but_lines_still_count_from_the_top() {
        let body = "---\ntitle: x\n---\nHello\n";
        let found = passages(body);
        assert_eq!(found.len(), 1);
        assert_eq!((found[0].start, found[0].end), (3, 3));
    }

    #[test]
    fn a_fence_is_kept_whole_and_its_hashes_are_not_headings() {
        let body = "# A\n```\n# not a heading\n\ncode\n```\nafter\n";
        let found = passages(body);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].heading.as_deref(), Some("A"));
        assert_eq!(found[0].end, 6);
    }

    #[test]
    fn a_long_section_splits_at_blank_lines() {
        let para = "word ".repeat(150);
        let body = format!("# H\n{para}\n\n{para}\n\n{para}\n");
        let found = passages(&body);
        assert!(found.len() >= 3, "{}", found.len());
        assert!(found.iter().all(|p| p.heading.as_deref() == Some("H")));
    }

    #[test]
    fn an_empty_body_has_no_passages() {
        assert!(passages("").is_empty());
        assert!(passages("\n\n  \n").is_empty());
    }

    fn body_strategy() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                Just("".to_owned()),
                Just("# Heading".to_owned()),
                Just("## Sub heading".to_owned()),
                Just("```".to_owned()),
                Just("~~~".to_owned()),
                "[a-z #]{0,40}",
                "[a-z ]{200,500}",
            ],
            0..40,
        )
        .prop_map(|lines| lines.join("\n"))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Every non-blank line of a body without front matter is in exactly
        /// one passage, and passages are ordered, disjoint and match their lines.
        #[test]
        fn prop_passages_cover_every_nonblank_line_once(body in body_strategy()) {
            prop_assume!(properties::split(&body).is_none());
            let found = passages(&body);
            let lines: Vec<&str> = body.split('\n').collect();
            let mut covered = vec![0u32; lines.len()];
            let mut last_end: Option<u32> = None;
            for p in &found {
                prop_assert!(p.start <= p.end);
                prop_assert!((p.end as usize) < lines.len());
                if let Some(prev) = last_end {
                    prop_assert!(p.start > prev, "overlap or disorder");
                }
                last_end = Some(p.end);
                for ix in p.start..=p.end {
                    covered[ix as usize] += 1;
                }
                prop_assert_eq!(&p.text, &lines[p.start as usize..=p.end as usize].join("\n"));
            }
            for (ix, line) in lines.iter().enumerate() {
                if !line.trim().is_empty() {
                    prop_assert_eq!(covered[ix], 1, "line {} of {:?}", ix, body);
                }
            }
        }
    }
}
