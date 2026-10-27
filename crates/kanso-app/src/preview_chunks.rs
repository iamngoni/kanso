//! Splits a note's Markdown into chunks the preview lays out on their own,
//! so an edit re-parses and re-lays out only the chunk it falls in.
//!
//! A chunk starts at a top-level block whose rendering does not depend on
//! the blocks before it: a paragraph that starts with a letter, or an ATX
//! heading after ordinary content (whose gap above is then known). Everything else stays with the chunk
//! before it. Notes with link reference or footnote definitions are not
//! split: a reference in one chunk could not see a definition in another.
//!
//! Shown alone, a chunk's first and last blocks lose the gaps the blocks
//! around them would give them; [`Start`] and [`End`] say what those gaps
//! are, and the preview puts them back.

use std::collections::HashMap;
use std::hash::{Hash as _, Hasher as _};
use std::ops::Range;

use markdown::ParseOptions;
use markdown::mdast::Node;

/// Below this size a note is split on the main thread, so it shows at once.
pub const SPLIT_SYNC_BYTES: usize = 4 * 1024;

/// How a chunk starts, for the room above it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Start {
    /// The note's first chunk, or a paragraph: nothing above it.
    Flush,
    /// A heading of this level after ordinary content: the rest of its
    /// section gap goes above it.
    Heading(u8),
}

/// How a chunk ends, for the room below it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum End {
    /// The note's last chunk: nothing below it.
    Last,
    /// Ordinary content: a paragraph gap below it.
    Gap,
    /// A heading of this level: a heading's gap below it.
    Heading(u8),
}

/// One chunk: where it is in the note, and how it starts and ends.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chunk {
    pub range: Range<usize>,
    pub start: Start,
    pub end: End,
}

/// The parse options gpui-base's Markdown view uses, so blocks are found
/// where it finds them.
pub(crate) fn parse_options() -> ParseOptions {
    let mut options = ParseOptions::gfm();
    // Front matter is a block, so the preview can hide it instead of
    // drawing its `---` lines as a rule and a heading.
    options.constructs.frontmatter = true;
    options.constructs.math_text = true;
    options.constructs.math_flow = true;
    options
}

/// `text` as a text view can show it: itself, or, when markdown-rs panics
/// on it, the same text as plain paragraphs. For Markdown shown outside the
/// preview's chunks: a quote's body, a version in History, release notes.
/// What a text was checked to be is kept, so a view drawn every frame
/// parses its text once.
pub fn showable(text: &str) -> std::borrow::Cow<'_, str> {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    let key = hasher.finish();
    let known = SHOWABLE.lock().ok().and_then(|c| c.get(&key).copied());
    let parses = known.unwrap_or_else(|| {
        let parses = kanso_render::markdown_safe::parses(text, &parse_options());
        if let Ok(mut checked) = SHOWABLE.lock() {
            if checked.len() >= CHECKED_KEPT {
                checked.clear();
            }
            checked.insert(key, parses);
        }
        parses
    });
    if parses {
        text.into()
    } else {
        kanso_render::markdown_safe::plain_paragraphs(text).into()
    }
}

/// Texts [`showable`] checked: a hash of each, and whether it parses.
static SHOWABLE: std::sync::LazyLock<std::sync::Mutex<HashMap<u64, bool>>> =
    std::sync::LazyLock::new(Default::default);

/// Whether a block renders as ordinary content: what a heading after it
/// counts as following. Display math and fenced diagrams are blocks like
/// any other: each sits whole inside one chunk, with a paragraph gap below.
fn is_ordinary(node: &Node) -> bool {
    matches!(
        node,
        Node::Paragraph(_)
            | Node::List(_)
            | Node::Code(_)
            | Node::Math(_)
            | Node::Blockquote(_)
            | Node::Table(_)
    )
}

/// The chunks of `text`, in order; together they cover it exactly. The
/// preview goes through [`prepare`], which shows text markdown-rs panics on
/// as plain paragraphs.
#[cfg(test)]
pub fn split(text: &str) -> Vec<Chunk> {
    split_parsed(text).unwrap_or_else(|| whole(text))
}

/// The one chunk that is all of `text`.
fn whole(text: &str) -> Vec<Chunk> {
    vec![Chunk {
        range: 0..text.len(),
        start: Start::Flush,
        end: End::Last,
    }]
}

/// The chunks of `text`, or `None` when markdown-rs panics on it: the text
/// view, which parses the same way, would too. Each chunk returned parses
/// alone (see [`reads_the_same`]), so no chunk can make it panic.
fn split_parsed(text: &str) -> Option<Vec<Chunk>> {
    let whole = || Some(whole(text));
    let root = match kanso_render::markdown_safe::to_mdast(text, &parse_options()) {
        Ok(root) => root,
        Err(kanso_render::markdown_safe::Failure::Panicked) => return None,
        Err(_) => return whole(),
    };
    let Some(children) = root.children() else {
        return whole();
    };
    if children
        .iter()
        .any(|n| matches!(n, Node::Definition(_) | Node::FootnoteDefinition(_)))
    {
        return whole();
    }
    let mut chunks = candidates(text, children);
    // A split is kept only where each side parses alone to the blocks the
    // whole note has there: Markdown has corners (a list that may interrupt
    // a paragraph only inside another list's context, say) that the rules
    // above don't see. A chunk that doesn't joins the one before it.
    let mut ix = 0;
    while ix < chunks.len() {
        if chunks.len() == 1 || reads_the_same(text, children, &chunks[ix]) {
            ix += 1;
            continue;
        }
        let merge_into = ix.saturating_sub(1);
        let later = chunks.remove(merge_into + 1);
        let earlier = &mut chunks[merge_into];
        earlier.range.end = later.range.end;
        earlier.end = later.end;
        ix = merge_into;
    }
    Some(chunks)
}

/// Hashes the shape of `node` into `hasher`: its kind, where it is from
/// `base`, what changes how it renders, and the same for its children.
fn hash_shape(node: &Node, base: usize, hasher: &mut impl std::hash::Hasher) {
    std::mem::discriminant(node).hash(hasher);
    if let Some(p) = node.position() {
        (
            p.start.offset.wrapping_sub(base),
            p.end.offset.wrapping_sub(base),
        )
            .hash(hasher);
    }
    match node {
        Node::List(l) => (l.ordered, l.start, l.spread).hash(hasher),
        Node::ListItem(i) => (i.spread, i.checked).hash(hasher),
        Node::Heading(h) => h.depth.hash(hasher),
        Node::Code(c) => (&c.lang, &c.meta).hash(hasher),
        Node::Math(m) => m.meta.hash(hasher),
        Node::Table(t) => format!("{:?}", t.align).hash(hasher),
        _ => {}
    }
    for child in node.children().into_iter().flatten() {
        hash_shape(child, base, hasher);
    }
}

/// Whether `chunk` parsed alone has the blocks the whole note (whose
/// top-level blocks are `children`) has in it.
fn reads_the_same(text: &str, children: &[Node], chunk: &Chunk) -> bool {
    let shapes = |nodes: &mut dyn Iterator<Item = &Node>, base: usize| -> Vec<u64> {
        nodes
            .map(|node| {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                hash_shape(node, base, &mut hasher);
                hasher.finish()
            })
            .collect()
    };
    let in_note = shapes(
        &mut children.iter().filter(|n| {
            n.position()
                .is_some_and(|p| chunk.range.contains(&p.start.offset))
        }),
        chunk.range.start,
    );
    let source = &text[chunk.range.clone()];
    // How a text parses alone never changes, so a chunk checked before
    // (typing elsewhere in the note) isn't parsed again.
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    source.hash(&mut hasher);
    let key = (hasher.finish(), in_note);
    if let Some(same) = CHECKED.lock().ok().and_then(|c| c.get(&key).copied()) {
        return same;
    }
    let same = kanso_render::markdown_safe::to_mdast(source, &parse_options())
        .is_ok_and(|alone| shapes(&mut alone.children().into_iter().flatten(), 0) == key.1);
    if let Ok(mut checked) = CHECKED.lock() {
        if checked.len() >= CHECKED_KEPT {
            checked.clear();
        }
        checked.insert(key, same);
    }
    same
}

/// Chunks checked by [`reads_the_same`]: a hash of the text and the shapes
/// of its blocks in the note, and whether it parsed alone to those.
static CHECKED: std::sync::LazyLock<std::sync::Mutex<Checked>> =
    std::sync::LazyLock::new(Default::default);
type Checked = HashMap<(u64, Vec<u64>), bool>;
/// How many checks are kept before they are forgotten.
const CHECKED_KEPT: usize = 8192;

/// Where the note could split, before checking each side parses alone.
fn candidates(text: &str, children: &[Node]) -> Vec<Chunk> {
    let mut chunks: Vec<Chunk> = Vec::new();
    let mut start = 0;
    let mut kind = Start::Flush;
    for (ix, node) in children.iter().enumerate().skip(1) {
        let prev = &children[ix - 1];
        let end = match prev {
            Node::Heading(h) => End::Heading(h.depth),
            _ if is_ordinary(prev) => End::Gap,
            _ => continue,
        };
        let Some(at) = node.position().map(|p| p.start.offset) else {
            continue;
        };
        // Only blocks that read the same alone: a paragraph that starts
        // with a letter (one starting with `-`, `*` or a digit can be a
        // rule or a list once the block before it is gone), and an ATX
        // heading (a setext one depends on the line under it).
        let first = text[at..].chars().next();
        let next = match node {
            Node::Paragraph(_) if first.is_some_and(char::is_alphabetic) => Start::Flush,
            Node::Heading(h) if end == End::Gap && first == Some('#') => Start::Heading(h.depth),
            _ => continue,
        };
        // The block starts at its first character; any indentation before
        // it stays with the chunk before.
        let at = text[..at].rfind('\n').map_or(0, |nl| nl + 1);
        if at <= start || !text.is_char_boundary(at) {
            continue;
        }
        chunks.push(Chunk {
            range: start..at,
            start: kind,
            end,
        });
        start = at;
        kind = next;
    }
    chunks.push(Chunk {
        range: start..text.len(),
        start: kind,
        end: End::Last,
    });
    chunks
}

/// A chunk ready for the preview: what it shows, how it starts and ends,
/// and a hash of all three to compare by.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prepared {
    pub text: String,
    pub start: Start,
    pub end: End,
    pub key: u64,
}

/// Splits `text` and prepares each chunk. Runs off the main thread for
/// long notes.
pub fn prepare(text: &str) -> Vec<Prepared> {
    #[cfg(test)]
    let started = std::time::Instant::now();
    let Some(chunks) = split_parsed(text) else {
        // Text markdown-rs panics on shows as plain paragraphs.
        let shown = kanso_render::markdown_safe::plain_paragraphs(text);
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        shown.hash(&mut hasher);
        return vec![Prepared {
            key: hasher.finish(),
            text: shown,
            start: Start::Flush,
            end: End::Last,
        }];
    };
    let prepared = chunks
        .iter()
        .map(|chunk| {
            let shown = &text[chunk.range.clone()];
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            shown.hash(&mut hasher);
            chunk.start.hash(&mut hasher);
            chunk.end.hash(&mut hasher);
            Prepared {
                key: hasher.finish(),
                text: shown.to_owned(),
                start: chunk.start,
                end: chunk.end,
            }
        })
        .collect();
    #[cfg(test)]
    PREPARE_NANOS.fetch_add(
        started.elapsed().as_nanos() as u64,
        std::sync::atomic::Ordering::Relaxed,
    );
    prepared
}

/// How long [`prepare`] has run in all, for the latency benchmark to tell
/// work done off the main thread in the app from work done on it.
#[cfg(test)]
pub static PREPARE_NANOS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Which of `old` (by key) to keep: the shared prefix and suffix, and the
/// range of `old` that `new[new_range]` replaces.
#[derive(Debug, PartialEq, Eq)]
pub struct Splice {
    pub old_range: Range<usize>,
    pub new_range: Range<usize>,
}

/// The smallest middle range to replace to turn `old` into `new`.
pub fn diff(old: &[u64], new: &[u64]) -> Splice {
    let prefix = old.iter().zip(new).take_while(|(a, b)| a == b).count();
    let suffix = old[prefix..]
        .iter()
        .rev()
        .zip(new[prefix..].iter().rev())
        .take_while(|(a, b)| a == b)
        .count();
    Splice {
        old_range: prefix..old.len() - suffix,
        new_range: prefix..new.len() - suffix,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    /// Top-level blocks as gpui-base's parser sees them, without their
    /// positions: everything that decides how they render.
    fn blocks(text: &str) -> Option<Vec<serde_json::Value>> {
        fn strip(value: &mut serde_json::Value) {
            match value {
                serde_json::Value::Object(map) => {
                    map.remove("position");
                    map.values_mut().for_each(strip);
                }
                serde_json::Value::Array(items) => items.iter_mut().for_each(strip),
                _ => {}
            }
        }
        let root = kanso_render::markdown_safe::to_mdast(text, &parse_options()).ok()?;
        Some(
            root.children()
                .unwrap()
                .iter()
                .filter(|n| !matches!(n, Node::Definition(_)))
                .map(|n| {
                    let mut value = serde_json::to_value(n).unwrap();
                    strip(&mut value);
                    value
                })
                .collect(),
        )
    }

    /// Markdown made of the blocks notes are made of, in any order, with
    /// the awkward cases: fences with blank lines inside, loose lists, lazy
    /// continuation, setext headings, rules, HTML blocks and math.
    fn gen_markdown() -> impl Strategy<Value = String> {
        let block = prop_oneof![
            "[A-Za-z][a-z ]{0,30}".prop_map(|s| format!("{s}\n")),
            "[A-Za-z][a-z ]{0,20}".prop_map(|s| format!("{s}\nand more of it\n")),
            (1u8..=6, "[a-z ]{1,12}")
                .prop_map(|(l, s)| format!("{} {s}\n", "#".repeat(l as usize))),
            "[a-z]{1,8}".prop_map(|s| format!("{s}\n===\n")),
            Just("- a\n- b\n".to_owned()),
            Just("- a\n\n- b\n".to_owned()),
            Just("1. one\n\n2. two\n".to_owned()),
            Just("- item\nlazy line\n".to_owned()),
            Just("- [x] done\n- [ ] open\n".to_owned()),
            Just("> quote\n> more\n".to_owned()),
            Just("> quote\nlazy\n".to_owned()),
            Just("```\ncode\n\nAfter blank\n# not a heading\n```\n".to_owned()),
            Just("~~~rust\nfn x() {}\n\n~~~\n".to_owned()),
            Just("    indented code\n\n    more\n".to_owned()),
            Just("| a | b |\n|---|---|\n| 1 | 2 |\n".to_owned()),
            Just("---\n".to_owned()),
            Just("***\n".to_owned()),
            Just("<!-- a comment\n\nParagraph inside -->\n".to_owned()),
            Just("<div>\nhtml\n</div>\n".to_owned()),
            Just("$$\nx = 1\n\nY\n$$\n".to_owned()),
            Just("![image](inkdrop://file:abc)\n".to_owned()),
            Just("Text with $5 and $10\n".to_owned()),
            Just("<script>\nvar a;\n\nWords inside\n</script>\n".to_owned()),
            Just("<pre>\nkept\n\nAlso kept\n</pre>\n".to_owned()),
            Just("Name | Value\n--- | ---\nOne | 1\n".to_owned()),
            Just("- outer\n  - inner\n\n    ```\n    code\n\n    ```\n".to_owned()),
            Just("```\nunclosed fence\n\nWords\n".to_owned()),
            Just("Para\n---\n".to_owned()),
            Just("  Indented para\n".to_owned()),
            Just("Hard break  \nnext\n".to_owned()),
            Just("* star list\n\n* second\n".to_owned()),
            Just("+ plus\n".to_owned()),
            Just("10) ten\n".to_owned()),
            Just("Über straße\n".to_owned()),
        ];
        (
            prop::collection::vec((block, prop::sample::select(vec!["", "\n", "\n\n"])), 0..14),
            any::<bool>(),
        )
            .prop_map(|(blocks, trailing)| {
                let mut text: String = blocks
                    .into_iter()
                    .map(|(b, gap)| format!("{b}{gap}"))
                    .collect();
                if !trailing {
                    text.pop();
                }
                text
            })
    }

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 1024,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    proptest! {
        #![proptest_config(config())]

        /// Round-trip: the chunks cover the text exactly, in order, none
        /// empty but a lone chunk of an empty note.
        #[test]
        fn prop_chunks_concatenate_to_the_text(text in gen_markdown()) {
            let chunks = split(&text);
            let joined: String = chunks.iter().map(|c| &text[c.range.clone()]).collect();
            prop_assert_eq!(&joined, &text);
            prop_assert_eq!(chunks[0].start, Start::Flush);
            prop_assert_eq!(chunks.last().unwrap().end, End::Last);
            prop_assert!(chunks[..chunks.len() - 1].iter().all(|c| c.end != End::Last));
            if chunks.len() > 1 {
                prop_assert!(chunks.iter().all(|c| !c.range.is_empty()));
            }
        }

        /// Equivalence: parsing each chunk as shown gives the same blocks as
        /// parsing the whole note, so the preview shows the same thing.
        #[test]
        fn prop_chunks_parse_to_the_notes_blocks(text in gen_markdown()) {
            // Text markdown-rs panics on isn't split (see the next property).
            let Some(whole) = blocks(&text) else {
                return Ok(());
            };
            let chunks = split(&text);
            let mut parts = Vec::new();
            for chunk in &chunks {
                let part = blocks(&text[chunk.range.clone()]);
                prop_assert!(part.is_some(), "a chunk doesn't parse");
                parts.extend(part.unwrap_or_default());
            }
            prop_assert_eq!(parts, whole);
        }

        /// Safety: whatever the note, every chunk the preview hands its text
        /// view parses without markdown-rs panicking, and nothing is lost:
        /// text it panics on shows as one chunk of plain paragraphs.
        #[test]
        fn prop_every_chunk_shown_parses(text in gen_markdown()) {
            let prepared = prepare(&text);
            for chunk in &prepared {
                prop_assert!(
                    kanso_render::markdown_safe::parses(&chunk.text, &parse_options()),
                    "{:?}", chunk.text
                );
            }
            if !kanso_render::markdown_safe::parses(&text, &parse_options()) {
                prop_assert_eq!(prepared.len(), 1);
                prop_assert_eq!(
                    &prepared[0].text,
                    &kanso_render::markdown_safe::plain_paragraphs(&text)
                );
            }
        }

        /// Invariant: a chunk starting with a heading starts with one after
        /// ordinary content, and its level is the heading's.
        #[test]
        fn prop_heading_starts_name_their_level(text in gen_markdown()) {
            for chunk in split(&text) {
                if let Start::Heading(level) = chunk.start {
                    let source = &text[chunk.range.clone()];
                    let hashes = source.trim_start().chars().take_while(|c| *c == '#').count();
                    prop_assert_eq!(hashes, level as usize, "{:?}", source);
                }
            }
        }

        /// Incremental equals full: keeping the prefix and suffix the diff
        /// finds and taking the middle from the new list gives the new list.
        #[test]
        fn prop_diff_splices_old_into_new(
            old in prop::collection::vec(0u64..6, 0..12),
            new in prop::collection::vec(0u64..6, 0..12),
        ) {
            let Splice { old_range, new_range } = diff(&old, &new);
            let mut spliced = old.clone();
            spliced.splice(old_range.clone(), new[new_range.clone()].iter().copied());
            prop_assert_eq!(&spliced, &new);
            prop_assert_eq!(old.len() - old_range.len(), new.len() - new_range.len());
        }

        /// Metamorphic: typing inside one chunk of a note changes only the
        /// chunks around it, never the ones far from the edit.
        #[test]
        fn prop_an_edit_keeps_distant_chunks(sections in 3usize..10, at in 0usize..1000, typed in "[a-z]{1,4}") {
            let text: String = (0..sections)
                .map(|n| format!("## Section {n}\n\nSome text for section {n}.\n\n- a point\n- another\n\n"))
                .collect();
            let lines: Vec<usize> = text.match_indices('\n').map(|(i, _)| i).collect();
            let at = lines[at % lines.len()];
            let mut edited = text.clone();
            edited.insert_str(at, &typed);
            let before: Vec<u64> = prepare(&text).iter().map(|c| c.key).collect();
            let after: Vec<u64> = prepare(&edited).iter().map(|c| c.key).collect();
            let splice = diff(&before, &after);
            prop_assert!(splice.old_range.len() <= 2, "{:?}", splice);
            prop_assert!(splice.new_range.len() <= 2, "{:?}", splice);
        }
    }

    /// markdown 1.0.0 panics on this ("Cannot push to non-parent"); the
    /// preview shows it as plain text instead of going down with it.
    #[test]
    fn text_markdown_rs_panics_on_shows_as_plain_paragraphs() {
        let text = "Para\n---\n---\na\n===";
        assert!(!kanso_render::markdown_safe::parses(text, &parse_options()));
        let prepared = prepare(text);
        assert_eq!(prepared.len(), 1);
        assert_eq!(
            prepared[0].text,
            "Para\\\n\\-\\-\\-\\\n\\-\\-\\-\\\na\\\n\\=\\=\\="
        );
        assert_eq!(split(text).len(), 1);
        assert_eq!(showable(text), prepared[0].text);
        assert_eq!(showable("# Fine"), "# Fine");
    }

    #[test]
    fn splits_before_paragraphs_and_headings_after_content() {
        let text = "# Title\nIntro.\n\n## A\n\n- x\n\n## B\n### C\n\nEnd.";
        let starts: Vec<(&str, Start, End)> = split(text)
            .into_iter()
            .map(|c| (&text[c.range], c.start, c.end))
            .collect();
        assert_eq!(
            starts,
            [
                ("# Title\n", Start::Flush, End::Heading(1)),
                ("Intro.\n\n", Start::Flush, End::Gap),
                ("## A\n\n- x\n\n", Start::Heading(2), End::Gap),
                // C follows a heading, so it stays with B.
                ("## B\n### C\n\n", Start::Heading(2), End::Heading(3)),
                ("End.", Start::Flush, End::Last),
            ]
        );
    }

    #[test]
    fn does_not_split_inside_fences_quotes_or_lists() {
        let text = "Intro.\n\n```\na\n\nb\n```\n\n> q\n\n- one\n\n  continued\n";
        let chunks = split(text);
        assert_eq!(chunks.len(), 1, "{chunks:?}");
    }

    #[test]
    fn notes_with_definitions_are_not_split() {
        let text = "See [the site][s].\n\nMore.\n\n[s]: https://example.com\n";
        assert_eq!(split(text).len(), 1);
        let text = "A note[^1].\n\nMore.\n\n[^1]: The footnote.\n";
        assert_eq!(split(text).len(), 1);
    }

    #[test]
    fn empty_and_single_block_notes_are_one_chunk() {
        assert_eq!(
            split(""),
            [Chunk {
                range: 0..0,
                start: Start::Flush,
                end: End::Last
            }]
        );
        assert_eq!(split("Just text").len(), 1);
    }

    #[test]
    fn a_rule_or_html_keeps_what_follows() {
        // What a rule or an HTML block leaves below itself isn't a gap the
        // preview knows, so the paragraph after it stays in its chunk.
        assert_eq!(split("One.\n\n---\n\nTwo.").len(), 1);
        assert_eq!(split("One.\n\n<div>x</div>\n\nTwo.").len(), 1);
    }

    /// Math and diagrams sit whole inside one chunk, and count as ordinary
    /// content: a heading after display math starts a chunk of its own.
    #[test]
    fn math_and_diagrams_stay_whole() {
        let text = "Intro.\n\n$$\nx^2\n\n\\frac12\n$$\n\n## Next\n\n```mermaid\ngraph TD\n\nA-->B\n```\n\nAfter.";
        let chunks = split(text);
        let math = text.find("$$").unwrap()..text.rfind("$$").unwrap() + 2;
        let fence = text.find("```").unwrap()..text.rfind("```").unwrap() + 3;
        for block in [math, fence] {
            assert!(
                chunks
                    .iter()
                    .any(|c| c.range.start <= block.start && block.end <= c.range.end),
                "{chunks:?}"
            );
        }
        assert!(
            chunks.iter().any(|c| c.start == Start::Heading(2)),
            "{chunks:?}"
        );
    }

    #[test]
    fn diff_examples() {
        assert_eq!(
            diff(&[1, 2, 3], &[1, 9, 3]),
            Splice {
                old_range: 1..2,
                new_range: 1..2
            }
        );
        assert_eq!(
            diff(&[1, 2, 3], &[1, 2, 7, 3]),
            Splice {
                old_range: 2..2,
                new_range: 2..3
            }
        );
        assert_eq!(
            diff(&[], &[4]),
            Splice {
                old_range: 0..0,
                new_range: 0..1
            }
        );
        assert_eq!(
            diff(&[1, 1], &[1]),
            Splice {
                old_range: 1..2,
                new_range: 1..1
            }
        );
    }
}
