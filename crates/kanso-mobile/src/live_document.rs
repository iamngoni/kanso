//! Live mode on a phone, for the whole text at once: the engine's
//! [`live::live_layout`] in UTF-16 units, drawn the way the phone's preview
//! draws a note. The phone's preview has its own bullets (three, by depth)
//! and numbers (each item's list's start plus its place, at every depth),
//! so the markers say those rather than the engine's, which are the
//! desktop preview's. The engine is left as it is: the desktop draws from
//! it as it stands.

use std::collections::HashMap;
use std::ops::Range;

use kanso_engine::live::{self, BlockKind, LinkTarget, LiveSpan, LiveStyle, MarkerKind};
use kanso_engine::text::Utf16Index;
use markdown::mdast::Node;

use crate::model::{
    LiveDocBlock, LiveDocBlockKind, LiveDocLink, LiveDocLinkKind, LiveDocMarker, LiveDocMarkerKind,
    LiveDocStyle, LiveDocStyleKind, LiveDocWidget, LiveDocument, Range16,
};

/// The phone preview's bullets, by list depth (`depth % 3`).
const BULLETS: [&str; 3] = ["•", "◦", "▪"];

/// The bullet the phone's preview draws for an item `depth` lists deep.
pub(crate) fn bullet(depth: usize) -> &'static str {
    BULLETS[depth % BULLETS.len()]
}

/// What Live mode draws for `text`, as of `version`, with carets and
/// selections at `reveal` (UTF-16 units of `text`; a range can run
/// backwards and is cut to the text). `missing` says whether a wiki-link's
/// title names no note. `index` is `text`'s.
pub(crate) fn live_document(
    text: &str,
    index: &Utf16Index,
    version: u64,
    reveal: &[Range16],
    missing: &dyn Fn(&str) -> bool,
    card_marks: bool,
) -> LiveDocument {
    let units = index.len_utf16() as u32;
    let to_byte = |unit: u32| index.utf16_to_byte(text, unit.min(units) as usize);
    let reveal: Vec<Range<usize>> = reveal
        .iter()
        .map(|range| to_byte(range.start)..to_byte(range.end))
        .collect();
    let layout = live::live_layout(text, &reveal);
    let at = |byte: usize| index.byte_to_utf16(text, byte) as u32;
    let range = |r: &Range<usize>| Range16 {
        start: at(r.start),
        end: at(r.end),
    };
    let lines = LineStarts::new(text);
    let numbers = numbers_of(text, &layout.markers);

    let conceal = layout.conceal.iter().map(range).collect();
    let blocks = layout
        .blocks
        .iter()
        .map(|block| {
            let (kind, level, language) = kind_of(&block.kind);
            LiveDocBlock {
                kind,
                level,
                language,
                first_line: block.lines.start as u32,
                end_line: block.lines.end as u32,
                source: range(&block.source),
                list_depth: block.list_depth as u32,
                quote_depth: block.quote_depth as u32,
                item_start: block.item_start,
                top: block.top.map(|top| top as u32),
                revealed: block.revealed,
            }
        })
        .collect();
    let markers = layout
        .markers
        .iter()
        .map(|marker| {
            let (kind, depth, text, checked) = match &marker.kind {
                MarkerKind::Bullet { depth, .. } => (
                    LiveDocMarkerKind::Bullet,
                    *depth,
                    bullet(*depth).to_owned(),
                    false,
                ),
                MarkerKind::Number { depth, text: drawn } => (
                    LiveDocMarkerKind::Number,
                    *depth,
                    first_digit(text, &marker.range)
                        .and_then(|digit| numbers.get(&digit).cloned())
                        .unwrap_or_else(|| drawn.clone()),
                    false,
                ),
                MarkerKind::Task { depth, checked } => {
                    (LiveDocMarkerKind::Task, *depth, String::new(), *checked)
                }
                MarkerKind::Hidden { depth } => {
                    (LiveDocMarkerKind::Hidden, *depth, String::new(), false)
                }
                MarkerKind::Quote { depth } => {
                    (LiveDocMarkerKind::Quote, *depth, String::new(), false)
                }
                MarkerKind::Indent => (LiveDocMarkerKind::Indent, 0, String::new(), false),
                MarkerKind::Rule => (LiveDocMarkerKind::Rule, 0, String::new(), false),
                MarkerKind::Underline => (LiveDocMarkerKind::Underline, 0, String::new(), false),
            };
            LiveDocMarker {
                range: range(&marker.range),
                line: marker.line as u32,
                kind,
                depth: depth as u32,
                text,
                checked,
            }
        })
        .collect();
    let mut styles: Vec<LiveDocStyle> = layout
        .styles
        .iter()
        .map(|(r, style)| {
            let (kind, level) = match style {
                LiveStyle::Strong => (LiveDocStyleKind::Strong, 0),
                LiveStyle::Emphasis => (LiveDocStyleKind::Emphasis, 0),
                LiveStyle::Strikethrough => (LiveDocStyleKind::Strikethrough, 0),
                LiveStyle::Code => (LiveDocStyleKind::Code, 0),
                LiveStyle::Link => (LiveDocStyleKind::Link, 0),
                LiveStyle::Heading(level) => (LiveDocStyleKind::Heading, *level),
            };
            LiveDocStyle {
                range: range(r),
                kind,
                level,
            }
        })
        .chain(layout.spans.iter().map(|(r, span)| {
            let (kind, level) = match span {
                LiveSpan::Quote { depth } => (
                    LiveDocStyleKind::Quote,
                    u8::try_from(*depth).unwrap_or(u8::MAX),
                ),
                LiveSpan::TaskDone => (LiveDocStyleKind::TaskDone, 0),
                LiveSpan::InlineMath => (LiveDocStyleKind::InlineMath, 0),
            };
            LiveDocStyle {
                range: range(r),
                kind,
                level,
            }
        }))
        .collect();
    if card_marks && text.contains(" :: ") {
        styles.extend(
            kanso_core::cards::card_marks(text)
                .into_iter()
                .map(|mark| LiveDocStyle {
                    range: range(&mark.separator),
                    kind: LiveDocStyleKind::CardSeparator,
                    level: 0,
                }),
        );
    }
    // Stable: a span keeps its place among those starting with it.
    styles.sort_by_key(|style| style.range.start);
    let links = layout
        .links
        .iter()
        .map(|link| {
            let (kind, target, heading, gone) = match &link.target {
                LinkTarget::Url(url) => (LiveDocLinkKind::Url, url.clone(), None, false),
                LinkTarget::Reference(label) => {
                    (LiveDocLinkKind::Reference, label.clone(), None, false)
                }
                LinkTarget::Wiki { title, heading } => (
                    LiveDocLinkKind::Wiki,
                    title.clone(),
                    heading.clone(),
                    missing(title),
                ),
            };
            LiveDocLink {
                range: range(&link.range),
                text: range(&link.text),
                kind,
                target,
                heading,
                missing: gone,
            }
        })
        .collect();
    let widgets = layout
        .widgets
        .iter()
        .map(|widget| {
            let (kind, _, language) = kind_of(&widget.kind);
            LiveDocWidget {
                kind,
                language,
                first_line: widget.lines.start as u32,
                end_line: widget.lines.end as u32,
                lines: range(&lines.whole(text, widget.lines.clone())),
                source: range(&widget.source),
            }
        })
        .collect();
    LiveDocument {
        version,
        conceal,
        blocks,
        markers,
        styles,
        links,
        widgets,
    }
}

fn kind_of(kind: &BlockKind) -> (LiveDocBlockKind, u8, Option<String>) {
    match kind {
        BlockKind::Paragraph => (LiveDocBlockKind::Paragraph, 0, None),
        BlockKind::Heading(level) => (LiveDocBlockKind::Heading, *level, None),
        BlockKind::Rule => (LiveDocBlockKind::Rule, 0, None),
        BlockKind::Code { lang } => (LiveDocBlockKind::Code, 0, lang.clone()),
        BlockKind::Table => (LiveDocBlockKind::Table, 0, None),
        BlockKind::Math => (LiveDocBlockKind::Math, 0, None),
        BlockKind::Html => (LiveDocBlockKind::Html, 0, None),
        BlockKind::FrontMatter => (LiveDocBlockKind::FrontMatter, 0, None),
        BlockKind::Image => (LiveDocBlockKind::Image, 0, None),
        BlockKind::Definition => (LiveDocBlockKind::Definition, 0, None),
        BlockKind::Blank => (LiveDocBlockKind::Blank, 0, None),
    }
}

/// Where a text's lines start, in bytes.
pub(crate) struct LineStarts(Vec<usize>);

impl LineStarts {
    pub(crate) fn new(text: &str) -> Self {
        LineStarts(
            std::iter::once(0)
                .chain(text.match_indices('\n').map(|(at, _)| at + 1))
                .collect(),
        )
    }

    pub(crate) fn count(&self) -> usize {
        self.0.len()
    }

    /// The line (0-based) byte `at` is on.
    pub(crate) fn line_of(&self, at: usize) -> usize {
        self.0
            .partition_point(|&start| start <= at)
            .saturating_sub(1)
    }

    /// Lines `lines` of `text` (end exclusive, cut to the text), without
    /// the last one's line break.
    pub(crate) fn whole(&self, text: &str, lines: Range<usize>) -> Range<usize> {
        let count = self.0.len();
        let first = lines.start.min(count - 1);
        let last = lines.end.saturating_sub(1).clamp(first, count - 1);
        let start = self.0[first];
        let mut end = self.0.get(last + 1).map_or(text.len(), |next| next - 1);
        if end > start && text.as_bytes()[end - 1] == b'\r' {
            end -= 1;
        }
        start..end.max(start)
    }
}

/// The first ASCII digit in `range` of `text`: where an ordered item's
/// number starts.
fn first_digit(text: &str, range: &Range<usize>) -> Option<usize> {
    let bytes = text.as_bytes().get(range.clone())?;
    let at = bytes.iter().position(u8::is_ascii_digit)?;
    Some(range.start + at)
}

/// Each ordered item's number as the phone's preview draws it (its list's
/// start plus its place), by where the number is written. Empty when no
/// number is drawn below the top level, where the engine's already are the
/// preview's.
fn numbers_of(text: &str, markers: &[live::LiveMarker]) -> HashMap<usize, String> {
    let mut numbers = HashMap::new();
    let nested = markers
        .iter()
        .any(|marker| matches!(marker.kind, MarkerKind::Number { depth, .. } if depth > 0));
    if !nested {
        return numbers;
    }
    let Ok(root) = kanso_render::markdown_safe::to_mdast(text, &parse_options()) else {
        return numbers;
    };
    let mut stack = vec![&root];
    while let Some(node) = stack.pop() {
        if let Node::List(list) = node
            && list.ordered
        {
            let start = list.start.unwrap_or(1);
            for (place, item) in list.children.iter().enumerate() {
                let Some(position) = item.position() else {
                    continue;
                };
                let line_end = text[position.start.offset..]
                    .find('\n')
                    .map_or(text.len(), |at| position.start.offset + at);
                if let Some(digit) = first_digit(text, &(position.start.offset..line_end)) {
                    let number = start.saturating_add(u32::try_from(place).unwrap_or(u32::MAX));
                    numbers.insert(digit, format!("{number}."));
                }
            }
        }
        stack.extend(node.children().into_iter().flatten());
    }
    numbers
}

/// The constructs the engine's Live mode reads (GFM, front matter, math),
/// so the lists found are the ones it marks.
fn parse_options() -> markdown::ParseOptions {
    let mut options = markdown::ParseOptions::gfm();
    options.constructs.frontmatter = true;
    options.constructs.math_text = true;
    options.constructs.math_flow = true;
    options
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_lines_drop_the_last_line_break() {
        let text = "a\r\nbc\n\nd";
        let lines = LineStarts::new(text);
        assert_eq!(lines.count(), 4);
        assert_eq!(&text[lines.whole(text, 0..1)], "a");
        assert_eq!(&text[lines.whole(text, 1..3)], "bc\n");
        assert_eq!(&text[lines.whole(text, 3..4)], "d");
        assert_eq!(&text[lines.whole(text, 3..9)], "d");
        assert_eq!(lines.line_of(0), 0);
        assert_eq!(lines.line_of(3), 1);
        assert_eq!(lines.line_of(text.len()), 3);
    }

    #[test]
    fn nested_numbers_count_from_their_lists_start() {
        let text = "1. a\n\n   3. b\n   3. c\n\n2. d\n";
        let index = Utf16Index::new(text);
        let doc = live_document(text, &index, 1, &[], &|_| false, false);
        let numbers: Vec<&str> = doc
            .markers
            .iter()
            .filter(|marker| marker.kind == LiveDocMarkerKind::Number)
            .map(|marker| marker.text.as_str())
            .collect();
        assert_eq!(numbers, vec!["1.", "3.", "4.", "2."]);
    }

    #[test]
    fn the_separator_of_a_card_line_is_a_dim_style_when_asked_for() {
        let text = "café :: thé\n`x :: y`\n";
        let index = Utf16Index::new(text);
        let separators = |on: bool| -> Vec<(u32, u32)> {
            live_document(text, &index, 1, &[], &|_| false, on)
                .styles
                .iter()
                .filter(|s| s.kind == LiveDocStyleKind::CardSeparator)
                .map(|s| (s.range.start, s.range.end))
                .collect()
        };
        assert_eq!(separators(true), vec![(5, 7)], "UTF-16 units, code skipped");
        assert!(separators(false).is_empty(), "off in Settings");
    }
}
