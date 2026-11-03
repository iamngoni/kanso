//! Links between notes, as written in a body: ID links
//! ([`NOTE_LINK_PREFIX`]`<id>`), and hand-typed or imported wiki-links
//! `[[Title]]`, `[[Title#Heading]]` and `[[Title|alias]]`.
//!
//! An ID link names its note by ID, so it survives renames; a
//! wiki-link names it by title, matched without regard to case or runs of
//! whitespace ([`normalize_title`]). Wiki-links inside code (fenced blocks
//! and inline code spans) are text, not links.

use std::ops::Range;

use crate::tasks::linked_note_ids;

/// The URL scheme of links inside note bodies. It is the scheme the app
/// that first wrote these notes uses, so imported notes, API clients and
/// Kanso's own links all agree; it stays exactly as written.
pub const LINK_SCHEME: &str = "inkdrop://";
/// An ID link to a note: this, then the note ID without its `note:` prefix.
pub const NOTE_LINK_PREFIX: &str = "inkdrop://note/";
/// An attachment link: this, then the file ID without its `file:` prefix.
pub const FILE_LINK_PREFIX: &str = "inkdrop://file:";

/// A wiki-link in a body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WikiLink {
    /// The bytes of `[[…]]`, brackets included.
    pub range: Range<usize>,
    /// The note's title as written, trimmed.
    pub title: String,
    /// The heading after `#`, trimmed, when there is one.
    pub heading: Option<String>,
    /// The text after `|`, trimmed, when there is one.
    pub alias: Option<String>,
}

impl WikiLink {
    /// What the link reads as: its alias, else its title (and heading).
    pub fn label(&self) -> String {
        match (&self.alias, &self.heading) {
            (Some(alias), _) => alias.clone(),
            (None, Some(heading)) => format!("{} › {heading}", self.title),
            (None, None) => self.title.clone(),
        }
    }
}

/// Where a link in a body points.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LinkTarget {
    /// A note by ID (`note:…`), from an ID link.
    Note(String),
    /// A note by normalized title, from a wiki-link.
    Title(String),
}

/// A link in a body and the line (0-based) it is on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoteLink {
    pub target: LinkTarget,
    /// The heading a wiki-link points at, as written.
    pub heading: Option<String>,
    pub line: u32,
    /// The lines (0-based, inclusive) an ID link points at with `#L12` or
    /// `#L12-14` after the note ID.
    pub lines: Option<(u32, u32)>,
}

/// The lines a link's `#L12` / `#L12-14` anchor names, as 0-based inclusive
/// `(first, last)`, from the text right after a note ID. `#L<digits>` and
/// `#L<digits>-<digits>` are reserved for lines, so a heading literally
/// named `L12` is only reachable by its wiki-link.
pub fn line_anchor(after_id: &str) -> Option<(u32, u32)> {
    let rest = after_id.strip_prefix("#L")?;
    let digits = |text: &str| -> (Option<u32>, usize) {
        let n = text.chars().take_while(char::is_ascii_digit).count();
        (text[..n].parse().ok(), n)
    };
    let (first, n) = digits(rest);
    let first = first.filter(|first| *first >= 1)?;
    let tail = &rest[n..];
    let last = match tail.strip_prefix('-') {
        Some(after) => {
            let (last, _) = digits(after);
            last.filter(|last| *last >= first).unwrap_or(first)
        }
        None => first,
    };
    Some((first - 1, last - 1))
}

/// The anchor text for lines `first..=last` (0-based), as `L13` or `L13-15`.
pub fn line_anchor_text(first: u32, last: u32) -> String {
    if last > first {
        format!("L{}-{}", first + 1, last + 1)
    } else {
        format!("L{}", first + 1)
    }
}

/// A title as wiki-links match it: trimmed, inner whitespace collapsed to
/// one space, lowercased.
pub fn normalize_title(title: &str) -> String {
    title
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Splits what is between `[[` and `]]` into title, heading and alias.
/// `None` when there is no title, or the text can't be a link (a bracket or
/// a line break inside).
pub fn parse_target(inner: &str) -> Option<(String, Option<String>, Option<String>)> {
    if inner.contains(['[', ']', '\n', '\r']) {
        return None;
    }
    let (target, alias) = match inner.split_once('|') {
        Some((target, alias)) => (target, Some(alias.trim().to_owned())),
        None => (inner, None),
    };
    let (title, heading) = match target.split_once('#') {
        Some((title, heading)) => (title, Some(heading.trim().to_owned())),
        None => (target, None),
    };
    let title = title.trim();
    if title.is_empty() {
        return None;
    }
    Some((
        title.to_owned(),
        heading.filter(|h| !h.is_empty()),
        alias.filter(|a| !a.is_empty()),
    ))
}

/// The byte ranges of a body that are code: fenced blocks (fences
/// included) and inline code spans. Sorted and disjoint.
pub fn code_ranges(body: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut fence: Option<(u8, usize, usize)> = None;
    let mut offset = 0;
    for line in body.split_inclusive('\n') {
        let trimmed = line.trim_start();
        let marker = fence_marker(trimmed);
        match (fence, marker) {
            (None, Some((ch, len))) => fence = Some((ch, len, offset)),
            (Some((ch, len, start)), Some((c, l))) if c == ch && l >= len => {
                ranges.push(start..offset + line.len());
                fence = None;
            }
            (Some(_), _) => {}
            (None, None) => {
                for span in inline_code_spans(line) {
                    ranges.push(offset + span.start..offset + span.end);
                }
            }
        }
        offset += line.len();
    }
    if let Some((_, _, start)) = fence {
        ranges.push(start..body.len());
    }
    ranges
}

/// `(fence char, run length)` when the line opens or closes a code fence.
pub(crate) fn fence_marker(trimmed: &str) -> Option<(u8, usize)> {
    let first = *trimmed.as_bytes().first()?;
    if first != b'`' && first != b'~' {
        return None;
    }
    let run = trimmed.bytes().take_while(|&b| b == first).count();
    (run >= 3).then_some((first, run))
}

/// Inline code spans in one line: a run of backticks up to the next run of
/// the same length.
fn inline_code_spans(line: &str) -> Vec<Range<usize>> {
    let bytes = line.as_bytes();
    let mut spans = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'`' {
            i += 1;
            continue;
        }
        let run = bytes[i..].iter().take_while(|&&b| b == b'`').count();
        let mut j = i + run;
        let mut closed = None;
        while j < bytes.len() {
            if bytes[j] == b'`' {
                let other = bytes[j..].iter().take_while(|&&b| b == b'`').count();
                if other == run {
                    closed = Some(j + other);
                    break;
                }
                j += other;
            } else {
                j += 1;
            }
        }
        match closed {
            Some(end) => {
                spans.push(i..end);
                i = end;
            }
            None => i += run,
        }
    }
    spans
}

fn in_ranges(ranges: &[Range<usize>], at: usize) -> bool {
    let ix = ranges.partition_point(|r| r.end <= at);
    ranges.get(ix).is_some_and(|r| r.start <= at)
}

/// The wiki-links in a body, in order, outside code. An embed (`![[…]]`)
/// isn't a link.
pub fn wiki_links(body: &str) -> Vec<WikiLink> {
    if !body.contains("[[") {
        return Vec::new();
    }
    let code = code_ranges(body);
    let mut links = Vec::new();
    let mut from = 0;
    while let Some(found) = body[from..].find("[[") {
        let start = from + found;
        // A run of three or more brackets: start at the last two.
        if body.as_bytes().get(start + 2) == Some(&b'[') {
            from = start + 1;
            continue;
        }
        if in_ranges(&code, start) || (start > 0 && body.as_bytes()[start - 1] == b'!') {
            from = start + 2;
            continue;
        }
        let rest = &body[start + 2..];
        let Some(close) = rest.find("]]") else {
            break;
        };
        let inner = &rest[..close];
        match parse_target(inner) {
            Some((title, heading, alias)) => {
                let end = start + 2 + close + 2;
                links.push(WikiLink {
                    range: start..end,
                    title,
                    heading,
                    alias,
                });
                from = end;
            }
            None => from = start + 2,
        }
    }
    links
}

/// The 0-based line of byte `at` in `body`.
pub fn line_of(body: &str, at: usize) -> u32 {
    body.as_bytes()[..at.min(body.len())]
        .iter()
        .filter(|&&b| b == b'\n')
        .count() as u32
}

/// Line `line` (0-based) of `body`, without its line break; empty past the
/// end.
pub fn line_text(body: &str, line: u32) -> &str {
    body.split('\n')
        .nth(line as usize)
        .unwrap_or("")
        .trim_end_matches('\r')
}

/// Every link in a body: ID links (anywhere, as backlinks
/// always found them) and wiki-links outside code, in order of where they
/// are.
pub fn note_links(body: &str) -> Vec<NoteLink> {
    let mut found: Vec<(usize, NoteLink)> = Vec::new();
    if body.contains(NOTE_LINK_PREFIX) {
        let prefix = NOTE_LINK_PREFIX;
        for (at, _) in body.match_indices(prefix) {
            let rest = &body[at + prefix.len()..];
            let end = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '-'))
                .unwrap_or(rest.len());
            if end == 0 {
                continue;
            }
            found.push((
                at,
                NoteLink {
                    target: LinkTarget::Note(format!("note:{}", &rest[..end])),
                    heading: None,
                    line: line_of(body, at),
                    lines: line_anchor(&rest[end..]),
                },
            ));
        }
    }
    for link in wiki_links(body) {
        found.push((
            link.range.start,
            NoteLink {
                target: LinkTarget::Title(normalize_title(&link.title)),
                heading: link.heading,
                line: line_of(body, link.range.start),
                lines: None,
            },
        ));
    }
    found.sort_by_key(|(at, _)| *at);
    found.into_iter().map(|(_, link)| link).collect()
}

/// Whether `body` links to note `id` with an ID link.
pub fn links_to_id(body: &str, id: &str) -> bool {
    linked_note_ids(body).iter().any(|linked| linked == id)
}

/// Rewrites the wiki-links to `old` (by normalized title) to point at
/// `new`, keeping their headings and aliases. A link without an alias that
/// read as the old title now reads as the new one. Returns the new body
/// and how many links changed.
pub fn retitle_wiki_links(body: &str, old: &str, new: &str) -> (String, usize) {
    let old = normalize_title(old);
    let new = new.trim();
    if old.is_empty() || new.is_empty() || new.contains(['[', ']', '|', '#', '\n']) {
        return (body.to_owned(), 0);
    }
    let mut out = String::with_capacity(body.len());
    let mut last = 0;
    let mut changed = 0;
    for link in wiki_links(body) {
        if normalize_title(&link.title) != old {
            continue;
        }
        out.push_str(&body[last..link.range.start]);
        out.push_str("[[");
        out.push_str(new);
        if let Some(heading) = &link.heading {
            out.push('#');
            out.push_str(heading);
        }
        if let Some(alias) = &link.alias {
            out.push('|');
            out.push_str(alias);
        }
        out.push_str("]]");
        last = link.range.end;
        changed += 1;
    }
    out.push_str(&body[last..]);
    (out, changed)
}

/// A line of Markdown as plain words: emphasis and code markers, link
/// targets, wiki-link brackets and list, quote and heading markers taken
/// out. For showing where a link or mention is.
pub fn plain_line(line: &str) -> String {
    let mut text = line.trim();
    // Block markers.
    while let Some(rest) = text.strip_prefix('>') {
        text = rest.trim_start();
    }
    let hashes = text.bytes().take_while(|&b| b == b'#').count();
    if (1..=6).contains(&hashes) && text[hashes..].starts_with([' ', '\t']) {
        text = text[hashes..].trim_start();
    }
    for marker in [
        "- [ ] ", "- [x] ", "- [X] ", "* [ ] ", "* [x] ", "- ", "* ", "+ ",
    ] {
        if let Some(rest) = text.strip_prefix(marker) {
            text = rest.trim_start();
            break;
        }
    }
    let mut out = String::with_capacity(text.len());
    let mut chars = text.char_indices().peekable();
    while let Some((at, c)) = chars.next() {
        match c {
            '*' | '_' | '`' | '~' => {}
            '[' if text[at..].starts_with("[[") => {
                // A wiki-link reads as its label.
                if let Some(close) = text[at + 2..].find("]]")
                    && let Some((title, heading, alias)) =
                        parse_target(&text[at + 2..at + 2 + close])
                {
                    let label = WikiLink {
                        range: 0..0,
                        title,
                        heading,
                        alias,
                    }
                    .label();
                    out.push_str(&label);
                    let end = at + 2 + close + 2;
                    while chars.peek().is_some_and(|(i, _)| *i < end) {
                        chars.next();
                    }
                } else {
                    out.push(c);
                }
            }
            ']' if text[at..].starts_with("](") => {
                // Skip a link's target.
                let target = &text[at + 2..];
                let end = target.find(')').map_or(text.len(), |e| at + 2 + e + 1);
                while chars.peek().is_some_and(|(i, _)| *i < end) {
                    chars.next();
                }
            }
            '[' => {}
            '!' if text[at..].starts_with("![") => {}
            _ => out.push(c),
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The lines (0-based) of a body that mention `title` as words of their
/// own, outside links and code: unlinked mentions. Case-insensitive.
pub fn mention_lines(body: &str, title: &str) -> Vec<u32> {
    let needle = normalize_title(title);
    if needle.chars().count() < 2 {
        return Vec::new();
    }
    let mut lines = Vec::new();
    let mut fence: Option<(u8, usize)> = None;
    for (ix, line) in body.split('\n').enumerate() {
        if let Some(open) = fence_marker(line.trim_start()) {
            fence = match fence {
                None => Some(open),
                Some((ch, len)) if open.0 == ch && open.1 >= len => None,
                other => other,
            };
            continue;
        }
        if fence.is_some() {
            continue;
        }
        if !free_mentions(&line.to_lowercase(), &needle).is_empty() {
            lines.push(ix as u32);
        }
    }
    lines
}

/// Where `needle` occurs in `hay` as words of its own (no word character
/// either side) outside links and inline code. `hay` and `needle` are
/// compared as they are; lowercase both to ignore case.
fn free_mentions(hay: &str, needle: &str) -> Vec<usize> {
    let covered = covered_ranges(hay);
    hay.match_indices(needle)
        .map(|(at, _)| at)
        .filter(|&at| {
            let before = hay[..at].chars().next_back();
            let after = hay[at + needle.len()..].chars().next();
            !before.is_some_and(is_word) && !after.is_some_and(is_word)
        })
        .filter(|&at| !covered.iter().any(|r| r.start <= at && at < r.end))
        .collect()
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// The parts of a line that are links (`[text](target)` or wiki-links) or
/// inline code.
pub(crate) fn covered_ranges(line: &str) -> Vec<Range<usize>> {
    let mut covered: Vec<Range<usize>> = inline_code_spans(line);
    for link in wiki_links(line) {
        covered.push(link.range);
    }
    let mut from = 0;
    while let Some(open) = line[from..].find('[') {
        let open = from + open;
        let Some(close) = line[open..].find("](") else {
            break;
        };
        let close = open + close;
        let end = line[close..]
            .find(')')
            .map_or(line.len(), |e| close + e + 1);
        covered.push(open..end);
        from = end.max(open + 1);
    }
    covered
}

/// Turns the first unlinked mention of `title` on line `line` (0-based) of
/// `body` into a link to `uri`, keeping the mention's words as they are
/// written. `None` when that line has no such mention.
pub fn link_mention(body: &str, line: u32, title: &str, uri: &str) -> Option<String> {
    let needle = title.trim().to_lowercase();
    if needle.is_empty() {
        return None;
    }
    let start: usize = body
        .split('\n')
        .take(line as usize)
        .map(|l| l.len() + 1)
        .sum();
    if start > body.len() {
        return None;
    }
    let text = body[start..].split('\n').next().unwrap_or("");
    let lower = text.to_lowercase();
    // Positions in the lowered line are the line's only when lowering kept
    // every character's length.
    if lower.len() != text.len() {
        return None;
    }
    let at = *free_mentions(&lower, &needle).first()?;
    let words = &text[at..at + needle.len()];
    let mut out = String::with_capacity(body.len() + uri.len() + 4);
    out.push_str(&body[..start + at]);
    out.push('[');
    out.push_str(words);
    out.push_str("](");
    out.push_str(uri);
    out.push(')');
    out.push_str(&body[start + at + needle.len()..]);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn parses_the_three_shapes() {
        let links = wiki_links("See [[Alpha]], [[Beta#Setup]] and [[ Gamma | the g ]].");
        assert_eq!(links.len(), 3);
        assert_eq!(links[0].title, "Alpha");
        assert_eq!(
            (links[1].title.as_str(), links[1].heading.as_deref()),
            ("Beta", Some("Setup"))
        );
        assert_eq!(
            (links[2].title.as_str(), links[2].alias.as_deref()),
            ("Gamma", Some("the g"))
        );
        assert_eq!(links[2].label(), "the g");
        assert_eq!(links[1].label(), "Beta › Setup");
    }

    #[test]
    fn code_and_embeds_are_not_links() {
        let body = "`[[Inline]]` ![[Embed]]\n```\n[[Fenced]]\n```\n[[Real]] [[]] [[ |x]]";
        let titles: Vec<String> = wiki_links(body).into_iter().map(|l| l.title).collect();
        assert_eq!(titles, ["Real"]);
    }

    #[test]
    fn an_id_link_can_name_lines() {
        assert_eq!(line_anchor("#L12"), Some((11, 11)));
        assert_eq!(line_anchor("#L12-14)"), Some((11, 13)));
        assert_eq!(line_anchor("#L0"), None);
        assert_eq!(line_anchor("#Lx"), None);
        assert_eq!(line_anchor("#Heading"), None);
        let links = note_links("[Pricing](inkdrop://note/abc#L12-14)");
        assert_eq!(links[0].lines, Some((11, 13)));
        assert_eq!(line_anchor_text(11, 13), "L12-14");
        assert_eq!(line_anchor_text(11, 11), "L12");
    }

    #[test]
    fn note_links_find_both_kinds_in_order() {
        let body = "[[Two]]\nx [a](inkdrop://note/abc) y\n";
        let links = note_links(body);
        assert_eq!(
            links,
            vec![
                NoteLink {
                    target: LinkTarget::Title("two".into()),
                    heading: None,
                    line: 0,
                    lines: None,
                },
                NoteLink {
                    target: LinkTarget::Note("note:abc".into()),
                    heading: None,
                    line: 1,
                    lines: None,
                },
            ]
        );
    }

    #[test]
    fn retitles_links_keeping_heading_and_alias() {
        let (body, n) = retitle_wiki_links(
            "[[old note]] [[Old Note#H|a]] [[Other]] `[[Old Note]]`",
            "Old  Note",
            "New",
        );
        assert_eq!(n, 2);
        assert_eq!(body, "[[New]] [[New#H|a]] [[Other]] `[[Old Note]]`");
    }

    #[test]
    fn plain_lines_read_as_words() {
        assert_eq!(
            plain_line("- [ ] Call **Ana** about [the plan](https://x.y) and [[Budget|money]]"),
            "Call Ana about the plan and money"
        );
        assert_eq!(plain_line("## A `code` heading"), "A code heading");
    }

    #[test]
    fn mentions_skip_links_code_and_partial_words() {
        let body = "Roadmap is due.\n[[Roadmap]] here\nroadmaps\n`Roadmap`\nthe ROADMAP!\nroadmaps [[Roadmap]]";
        assert_eq!(mention_lines(body, "Roadmap"), vec![0, 4]);
    }

    #[test]
    fn links_a_mention_in_place() {
        let body = "intro\nSee [roadmap](x) and the Roadmap, roadmap.\n";
        assert_eq!(
            link_mention(body, 1, "roadmap", "inkdrop://note/r").as_deref(),
            Some("intro\nSee [roadmap](x) and the [Roadmap](inkdrop://note/r), roadmap.\n")
        );
        assert_eq!(link_mention(body, 0, "roadmap", "u"), None);
    }

    proptest! {
        /// After linking a mention its line has one free mention fewer, and
        /// taking the link's brackets and target out again gives the body
        /// back.
        #[test]
        fn prop_linking_a_mention_takes_one_away(words in prop::collection::vec(prop_oneof![Just("plan"), Just("Plan"), Just("plans"), Just("[[Plan]]"), Just("x")], 1..8), before in "[a-z\n]{0,10}") {
            let line = words.join(" ");
            let body = format!("{before}\n{line}\nend");
            let at = before.matches('\n').count() as u32 + 1;
            let free = |b: &str| free_mentions(&line_text(b, at).to_lowercase(), "plan").len();
            match link_mention(&body, at, "Plan", "inkdrop://note/p") {
                Some(linked) => {
                    prop_assert_eq!(free(&linked) + 1, free(&body));
                    let d = body.bytes().zip(linked.bytes()).take_while(|(a, b)| a == b).count();
                    prop_assert_eq!(linked.as_bytes()[d], b'[');
                    let back = format!("{}{}", &linked[..d], &linked[d + 1..]).replacen("](inkdrop://note/p)", "", 1);
                    prop_assert_eq!(back, body);
                }
                None => prop_assert_eq!(free(&body), 0),
            }
        }
    }

    fn title() -> impl Strategy<Value = String> {
        "[A-Za-z][A-Za-z0-9 ]{0,12}[A-Za-z0-9]".prop_map(|s| s)
    }

    proptest! {
        /// Round trip: a link written from its parts parses back to them.
        #[test]
        fn prop_written_links_parse_back(t in title(), h in prop::option::of(title()), a in prop::option::of(title()), pre in "[a-z ]{0,10}", post in "[a-z ]{0,10}") {
            let mut link = format!("[[{t}");
            if let Some(h) = &h { link.push('#'); link.push_str(h); }
            if let Some(a) = &a { link.push('|'); link.push_str(a); }
            link.push_str("]]");
            let body = format!("{pre}{link}{post}");
            let links = wiki_links(&body);
            prop_assert_eq!(links.len(), 1);
            prop_assert_eq!(&body[links[0].range.clone()], link.as_str());
            prop_assert_eq!(links[0].title.as_str(), t.trim());
            prop_assert_eq!(links[0].heading.as_deref(), h.as_deref().map(str::trim));
            prop_assert_eq!(links[0].alias.as_deref(), a.as_deref().map(str::trim));
        }

        /// Invariants: links are in order, disjoint, start with `[[` and end
        /// with `]]`, whatever the text.
        #[test]
        fn prop_links_are_ordered_and_bracketed(body in "[\\[\\]a-z#|`\n! ]{0,80}") {
            let links = wiki_links(&body);
            let mut last = 0;
            for link in &links {
                prop_assert!(link.range.start >= last);
                prop_assert!(body[link.range.clone()].starts_with("[["));
                prop_assert!(body[link.range.clone()].ends_with("]]"));
                last = link.range.end;
            }
        }

        /// Normalizing is idempotent and ignores case and spacing.
        #[test]
        fn prop_normalize_is_idempotent(t in "[A-Za-z \t]{0,20}") {
            let once = normalize_title(&t);
            prop_assert_eq!(normalize_title(&once), once.clone());
            prop_assert_eq!(normalize_title(&t.to_uppercase()), once);
        }

        /// Retitling then retitling back restores every link's title, and
        /// counts the links to the old title.
        #[test]
        fn prop_retitle_counts_and_inverts(old in title(), new in title(), others in prop::collection::vec(prop::option::of(title()), 0..5)) {
            prop_assume!(normalize_title(&old) != normalize_title(&new));
            let body: String = others.iter().map(|o| match o {
                Some(t) => format!("[[{t}]] "),
                None => format!("[[{old}]] "),
            }).collect();
            let expected = wiki_links(&body).iter().filter(|l| normalize_title(&l.title) == normalize_title(&old)).count();
            let (renamed, n) = retitle_wiki_links(&body, &old, &new);
            prop_assert_eq!(n, expected);
            prop_assert_eq!(wiki_links(&renamed).iter().filter(|l| normalize_title(&l.title) == normalize_title(&old)).count(), 0);
            prop_assert_eq!(wiki_links(&renamed).len(), wiki_links(&body).len());
        }
    }
}
