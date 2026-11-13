//! Live mode in the note's editor: the Markdown drawn in place, the way the
//! Preview draws it, as Typora does. The engine ([`kanso_engine::live`])
//! says which marks to hide and what to draw for them, which spans to style,
//! where the blocks are and which complex blocks to draw as widgets; this
//! turns that into the editor's presentation (hidden ranges, bullets and
//! task boxes, heading sizes, the Preview's gaps, insets, quote bars and
//! rules, and the blocks drawn by the Preview's own renderer) and text
//! decorations, with the Preview's metrics and colours.
//!
//! The numbers are the Preview's: [`kanso_render::typography`] for what it
//! shares with an exported page, and the text view's own (`node.rs` in
//! `vendor/gpui-base`) for the rest, named here where they are used.
//!
//! The layout is worked out again whenever the text changes or the caret
//! moves, at once, so a hidden range never outlives the text it was hidden
//! in.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::Range;
use std::rc::Rc;

use gpui_kit::base::input::{
    BlockWidget, BlockWidgetRenderer, ChipHeight, ChipIcon, EditorPresentation, Ornament,
    Replacement, ReplacementChip, ReplacementColumn, ReplacementImage, TextDecoration,
    TextDecorationCollection,
};
use gpui_kit::base::text::TextViewState;
use gpui_kit::component::input::EditorState;
use gpui_kit::*;
use kanso_engine::live::{
    BlockKind, LinkTarget, LiveBlock, LiveLayout, LiveSpan, LiveStyle, MarkerKind, live_layout,
};
use kanso_render::typography;

pub use crate::preview_rich::TypesetMath;

/// A quote's bar, and the room between it and the quote's text, in points
/// (`preview::quote_extensions`).
const QUOTE_BAR: f32 = 3.;
const QUOTE_PAD: f32 = 14.;
/// How strongly a quote's bar shows its colour (`preview::quote_extensions`).
const QUOTE_BAR_OPACITY: f32 = 0.45;
/// A list's indent per level, and the room right of its marker, in ems of
/// the body (`render_list_item`: `body * 1.4`, `indent * 0.3`).
const LIST_INDENT: f32 = 1.4;
const MARKER_GAP: f32 = 0.3;
/// The room above every list item but a list's first, in ems of the body.
const ITEM_GAP: f32 = 0.25;
/// A task's box and the room right of it, in rems (`rems(0.9375)`, `mr_2`),
/// and the box's corners and border, in pixels.
const CHECK_SIZE: f32 = 0.9375;
const CHECK_GAP: f32 = 0.5;
const CHECK_RADIUS: f32 = 4.;
const CHECK_BORDER: f32 = 1.5;
/// The room above and below a rule, in ems of the body (`rule_space`).
const RULE_SPACE: f32 = 1.6;
/// The gap above a heading right after another, relative to its size.
const HEADING_AFTER_HEADING: f32 = 0.25;
/// Inline code's size, relative to the text around it (`mark_highlight`).
const CODE_SIZE: f32 = 0.875;
/// A link's underline, and an unresolved wiki-link's colour and underline,
/// as opacities of the link colour (`link_underline`,
/// `wikilinks::UnresolvedLink`).
const LINK_UNDERLINE: f32 = 0.4;
const UNRESOLVED_TEXT: f32 = 0.78;
const UNRESOLVED_UNDERLINE: f32 = 0.55;
/// The room either side of a code span, in pixels (`INLINE_CODE_PADDING`).
const CODE_PADDING: f32 = 4.;
/// The room between a due chip's icon and its words, in points.
const DUE_ICON_GAP: f32 = 4.;
/// A due chip's text size, relative to the line's (`due::DueChip`).
const DUE_SIZE: f32 = 0.8;
/// How tall a run of hidden lines is drawn when nothing is to be between
/// the blocks around it: as good as nothing (a widget must have a height).
const NO_GAP: f32 = 0.0001;

/// A heading's line height, relative to its size, as the text view sets it
/// (`heading_metrics` in `node.rs`): tighter than the body's.
pub fn heading_line_height(level: u8) -> f32 {
    if level == 1 { 1.25 } else { 1.35 }
}

/// The sizes Live draws at, in the window's pixels: the Preview's text
/// size, the rem the component library's sizes follow, and the app's zoom.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LiveMetrics {
    pub body: f32,
    pub rem: f32,
    pub zoom: f32,
    /// The body face's ascent and descent at the body size: where a row's
    /// baseline is, which a formula sits on.
    pub ascent: f32,
    pub descent: f32,
    /// The window's device pixels per pixel, which the preview's layout
    /// rounds its boxes to; `0.` rounds nothing.
    pub scale: f32,
}

impl LiveMetrics {
    /// `length` as the layout sets an authored length: rounded to whole
    /// device pixels, halves toward zero.
    fn snap(&self, length: f32) -> f32 {
        if self.scale <= 0. {
            return length;
        }
        let device = length * self.scale;
        (device.abs() - 0.5).ceil().copysign(device) / self.scale
    }

    /// Where a row's baseline is, from its top: the body text centred in it.
    fn baseline(&self) -> f32 {
        (self.line() - self.ascent - self.descent) / 2. + self.ascent
    }

    /// How much taller above its baseline a line holding an inline element
    /// with `baseline` and `height` is, and below it, as the text view's
    /// line box grows around one (`inline_flow.rs`, `layout_measured_flow`).
    fn grows(&self, baseline: f32, height: f32) -> (f32, f32) {
        let above = self.baseline();
        let below = self.line() - above;
        (
            (baseline - above).max(0.),
            (height - baseline - below).max(0.),
        )
    }

    /// The baseline and height of an element the text view lays out inline
    /// `height` tall with no baseline of its own: its bottom keeps a plain
    /// line's room below the baseline, that line as tall as the window's
    /// (rounded) line height (`MeasuredInlineObject::measure`).
    fn element_metrics(&self, height: f32) -> (f32, f32) {
        let rounded = (self.body * typography::LINE_HEIGHT)
            .round()
            .max(self.ascent + self.descent);
        let below = rounded - ((rounded - self.ascent - self.descent) / 2. + self.ascent);
        ((height - below).max(0.), height)
    }

    /// Where an inline element's text sits, from its top: centred in the
    /// window's (rounded) line height, which the element is set in.
    fn element_text_baseline(&self) -> f32 {
        let rounded = (self.body * typography::LINE_HEIGHT).round();
        (rounded - self.ascent - self.descent) / 2. + self.ascent
    }

    /// Where a due chip's words sit, from its top (`due::DueChip`): a line
    /// `size * 1.2` tall (snapped as text is), centred in the chip `size *
    /// 1.55` tall on whole device pixels, the words centred in that line.
    /// The chip's face is the body's at `size`, so its metrics scale.
    fn due_text_baseline(&self, size: f32) -> f32 {
        let ratio = size / self.body;
        let (ascent, descent) = (self.ascent * ratio, self.descent * ratio);
        let line = self.snap(size * 1.2);
        let top = self.snap((self.snap(size * 1.55) - line) / 2.);
        top + (line - ascent - descent) / 2. + ascent
    }

    /// The height of a row of body text: the text view's line height,
    /// snapped to whole device pixels as its text layout snaps it
    /// (`plain_line_height` in `inline_flow.rs`).
    pub fn line(&self) -> f32 {
        self.snap(self.body * typography::LINE_HEIGHT)
    }
    /// A list's indent, and the room right of its marker: authored lengths
    /// of the text view's, so rounded to device pixels apart.
    fn indent(&self) -> f32 {
        self.snap(self.body * LIST_INDENT)
    }
    fn marker_gap(&self) -> f32 {
        self.snap(self.body * LIST_INDENT * MARKER_GAP)
    }
    fn paragraph_gap(&self) -> f32 {
        self.rem * typography::PARAGRAPH_GAP
    }
    fn rule_space(&self) -> f32 {
        self.body * RULE_SPACE
    }
    fn heading_size(&self, level: u8) -> f32 {
        self.body * typography::heading(level).scale
    }
    /// How far each quote insets its text.
    fn quote_step(&self) -> f32 {
        (QUOTE_BAR + QUOTE_PAD) * self.zoom
    }
    /// A task's box and the room right of it.
    fn task_column(&self) -> f32 {
        self.rem * (CHECK_SIZE + CHECK_GAP)
    }
}

/// The Preview's colours (its theme's, as `preview::text_style` maps them)
/// and the faces Live draws in.
#[derive(Clone, Debug, PartialEq)]
pub struct LivePalette {
    pub text: Hsla,
    pub heading: Option<Hsla>,
    pub link: Hsla,
    pub quote: Hsla,
    /// Finished tasks and list markers.
    pub muted: Hsla,
    pub border: Hsla,
    pub code_background: Hsla,
    pub code_text: Option<Hsla>,
    /// A ticked box's check mark: white on a light theme, black on a dark.
    pub check: Hsla,
    /// The face inline code is set in.
    pub mono: SharedString,
    /// The page the text is on, which translucent colours are drawn over.
    pub background: Hsla,
    /// A code span's corners (the text view's small radius).
    pub code_radius: Pixels,
    /// The calendar icon a due chip shows, as the icon theme draws it.
    pub calendar: SharedString,
}

impl LivePalette {
    /// The Preview's colours in the theme in force.
    pub fn from_theme(cx: &App) -> LivePalette {
        use gpui_kit::component::ActiveTheme as _;
        let p = crate::themes::preview_colors(cx);
        let dark = cx.theme().is_dark();
        LivePalette {
            text: p.text.into(),
            heading: p.heading.map(Into::into),
            link: p.link.into(),
            quote: p.quote.into(),
            muted: p.task_done.into(),
            border: p.border.into(),
            code_background: p.inline_code_background.into(),
            code_text: p.inline_code_text.map(Into::into),
            check: if dark {
                gpui_kit::black()
            } else {
                gpui_kit::white()
            },
            mono: cx.theme().mono_font_family.clone(),
            background: crate::theme::Colors::get(cx).editor.into(),
            code_radius: gpui_kit::base::Theme::global(cx).tokens.radius.sm,
            calendar: gpui_kit::component::resolve_icon_path(
                gpui_kit::component::IconNamed::path(gpui_kit::assets::IconName::Calendar),
                cx,
            ),
        }
    }

    /// How `style` draws its span (links are drawn by [`Self::link`]).
    fn style(&self, style: LiveStyle) -> HighlightStyle {
        match style {
            LiveStyle::Strong => HighlightStyle {
                font_weight: Some(FontWeight::BOLD),
                ..Default::default()
            },
            LiveStyle::Emphasis => HighlightStyle {
                font_style: Some(FontStyle::Italic),
                ..Default::default()
            },
            LiveStyle::Strikethrough => HighlightStyle {
                strikethrough: Some(StrikethroughStyle {
                    thickness: gpui_kit::px(1.),
                    color: None,
                }),
                ..Default::default()
            },
            LiveStyle::Code => self.code(),
            LiveStyle::Link => self.link(true),
            LiveStyle::Heading(level) => {
                // Every level semibold, the first bold; the sixth muted
                // unless the theme colours headings.
                let color = self.heading.or((level >= 6).then_some(self.muted));
                HighlightStyle {
                    font_weight: Some(if level == 1 {
                        FontWeight::BOLD
                    } else {
                        FontWeight::SEMIBOLD
                    }),
                    color,
                    ..Default::default()
                }
            }
        }
    }

    /// `color` at `opacity`, as drawn over the page: the editor would blend
    /// a translucent colour over the text's own.
    fn over_page(&self, color: Hsla, opacity: f32) -> Hsla {
        self.background.blend(color.opacity(opacity))
    }

    /// Inline code: on its fill, in its colour when the theme gives one.
    fn code(&self) -> HighlightStyle {
        HighlightStyle {
            background_color: Some(self.code_background),
            color: self.code_text,
            ..Default::default()
        }
    }

    /// A link's text: in the link colour with a hairline under it, or, for a
    /// wiki-link to no note, quieter.
    fn link(&self, resolved: bool) -> HighlightStyle {
        let (text, line) = if resolved {
            // The underline translucent, as the text view sets it: text runs
            // draw it the same way in both.
            (self.link, self.link.opacity(LINK_UNDERLINE))
        } else {
            (
                self.over_page(self.link, UNRESOLVED_TEXT),
                self.over_page(self.link, UNRESOLVED_UNDERLINE),
            )
        };
        HighlightStyle {
            color: Some(text),
            underline: Some(UnderlineStyle {
                thickness: gpui_kit::px(1.),
                color: Some(line),
                wavy: false,
            }),
            ..Default::default()
        }
    }

    /// How a span of `span` draws.
    fn span(&self, span: LiveSpan) -> Option<HighlightStyle> {
        match span {
            LiveSpan::Quote { .. } => Some(HighlightStyle::color(self.quote)),
            LiveSpan::TaskDone => Some(HighlightStyle::color(self.muted)),
            LiveSpan::InlineMath => None,
        }
    }
}

/// A due date drawn as a chip: its range, its words and its colours.
#[derive(Clone, Debug, PartialEq)]
pub struct DueLabel {
    pub range: Range<usize>,
    pub label: String,
    pub text: Hsla,
    pub fill: Hsla,
}

/// A run of lines drawn as something else: a block the Preview's renderer
/// draws, or the gap between two blocks where the lines are blank.
#[derive(Clone, Debug, PartialEq)]
pub struct WidgetPlan {
    pub lines: Range<usize>,
    /// How far in its block sits (its list's indent), past its quotes' bars.
    pub inset: f32,
    /// How many quotes it is in: their bars are drawn beside it.
    pub quotes: usize,
    pub kind: WidgetKind,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WidgetKind {
    /// Empty room this tall.
    Gap(f32),
    /// Markdown for the Preview's renderer, its quote marks and indent taken
    /// off, and an estimate of its height.
    Markdown { source: String, height: f32 },
}

impl WidgetPlan {
    /// Names what it draws, so its measured height is kept.
    pub fn key(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.lines.start.hash(&mut hasher);
        self.quotes.hash(&mut hasher);
        self.inset.to_bits().hash(&mut hasher);
        match &self.kind {
            WidgetKind::Gap(height) => height.to_bits().hash(&mut hasher),
            WidgetKind::Markdown { source, .. } => source.hash(&mut hasher),
        }
        hasher.finish()
    }

    fn height(&self) -> f32 {
        match &self.kind {
            WidgetKind::Gap(height) => *height,
            WidgetKind::Markdown { height, .. } => *height,
        }
    }
}

/// What the editor draws for a layout: its presentation (the widgets'
/// renderers aside), the widgets, and the decorations.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LivePlan {
    pub presentation: EditorPresentation,
    pub widgets: Vec<WidgetPlan>,
    pub decorations: Vec<TextDecoration>,
}

/// What a block is to the gaps around it, as the text view tells them.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Flow {
    Heading(u8),
    Rule,
    Other,
}

impl Flow {
    fn of(block: &LiveBlock) -> Flow {
        match block.kind {
            BlockKind::Heading(level) => Flow::Heading(level),
            BlockKind::Rule => Flow::Rule,
            _ => Flow::Other,
        }
    }

    /// What a top-level block is: a heading or a rule only when it is one
    /// itself, not in a list or a quote.
    fn top(block: &LiveBlock) -> Flow {
        if block.list_depth == 0 && block.quote_depth == 0 {
            Flow::of(block)
        } else {
            Flow::Other
        }
    }
}

/// The gap below a block of `flow` when another follows it.
fn bottom(flow: Flow, m: &LiveMetrics) -> f32 {
    match flow {
        Flow::Heading(level) => m.heading_size(level) * typography::HEADING_BOTTOM_GAP,
        Flow::Rule => m.rule_space(),
        Flow::Other => m.paragraph_gap(),
    }
}

/// The room between a block of `prev` and the next, of `next`, in one flow:
/// the gap below the first, which the room above the second takes in (the
/// two collapse like CSS margins).
///
/// The text view sets the two as padding, each rounded to whole device
/// pixels as the layout rounds an authored length, so they are rounded
/// apart here too.
fn flow_gap(prev: Flow, next: Flow, m: &LiveMetrics) -> f32 {
    m.snap(bottom(prev, m)) + m.snap(top(prev, next, m))
}

/// The room above a block of `next` after one of `prev`, past the gap below
/// that one (`node.rs`, `render_block`).
fn top(prev: Flow, next: Flow, m: &LiveMetrics) -> f32 {
    match next {
        Flow::Heading(level) => {
            let size = m.heading_size(level);
            let section = size * typography::heading(level).gap;
            match prev {
                Flow::Heading(_) => size * HEADING_AFTER_HEADING,
                Flow::Rule => (section - m.rule_space()).max(0.),
                Flow::Other => (section - m.paragraph_gap()).max(0.),
            }
        }
        Flow::Rule => (m.rule_space() - bottom(prev, m)).max(0.),
        Flow::Other => 0.,
    }
}

/// The room between two blocks of one top-level block (a list or a quote).
fn inner_gap(a: &LiveBlock, b: &LiveBlock, m: &LiveMetrics) -> f32 {
    let item = |b: &LiveBlock, a: &LiveBlock| {
        if b.item_start && b.list_depth <= a.list_depth {
            m.snap(m.body * ITEM_GAP)
        } else {
            0.
        }
    };
    if b.quote_depth > a.quote_depth {
        // A quote in the flow: the block before it keeps its gap below.
        return if a.list_depth == 0 {
            m.snap(bottom(Flow::of(a), m))
        } else {
            0.
        };
    }
    if b.quote_depth < a.quote_depth {
        // Past a quote, which keeps a paragraph's gap below.
        return if b.list_depth == 0 {
            flow_gap(Flow::Other, Flow::of(b), m)
        } else {
            item(b, a)
        };
    }
    match (a.list_depth, b.list_depth) {
        (0, 0) => flow_gap(Flow::of(a), Flow::of(b), m),
        (0, _) => m.snap(bottom(Flow::of(a), m)),
        (_, 0) => flow_gap(Flow::Other, Flow::of(b), m),
        _ => item(b, a),
    }
}

/// Whether a block is drawn at all: blank lines and link definitions are
/// not (the Preview leaves definitions out).
fn shown(block: &LiveBlock) -> bool {
    !matches!(block.kind, BlockKind::Blank | BlockKind::Definition)
}

/// What Live draws for `layout` of `text`: `resolved` says whether a
/// wiki-link's note exists, `due` gives the due dates to draw as chips.
pub fn plan(
    text: &str,
    layout: &LiveLayout,
    m: &LiveMetrics,
    palette: &LivePalette,
    resolved: &dyn Fn(&str) -> bool,
    due: &[DueLabel],
    math: &[(Range<usize>, TypesetMath)],
) -> LivePlan {
    let lines = line_starts(text);
    let line_of = |at: usize| lines.partition_point(|&s| s <= at) - 1;
    let mut presentation = EditorPresentation::default();
    let mut space: HashMap<usize, (f32, f32)> = HashMap::new();
    let mut insets: Vec<(usize, f32)> = Vec::new();
    let mut widgets: Vec<WidgetPlan> = Vec::new();
    let widget_lines: Vec<Range<usize>> = layout.widgets.iter().map(|w| w.lines.clone()).collect();
    let on_widget = |line: usize| widget_lines.iter().any(|w| w.contains(&line));

    // Markers by line.
    let mut markers: HashMap<usize, Vec<&kanso_engine::live::LiveMarker>> = HashMap::new();
    for marker in &layout.markers {
        markers.entry(marker.line).or_default().push(marker);
    }

    // Where each block's text starts.
    let list_x = |block: &LiveBlock| -> f32 {
        let depth = block.list_depth as f32;
        if block.list_depth == 0 {
            return 0.;
        }
        if block.item_start {
            let first = block.lines.start;
            let kind = markers
                .get(&first)
                .and_then(|ms| ms.iter().find(|mk| is_list_marker(&mk.kind)))
                .map(|mk| mk.kind.clone());
            match kind {
                Some(MarkerKind::Task { .. }) => (depth - 1.) * m.indent() + m.task_column(),
                Some(MarkerKind::Hidden { .. }) => (depth - 1.) * m.indent(),
                _ => depth * m.indent(),
            }
        } else {
            depth * m.indent()
        }
    };

    // Gaps, collapsed blank runs, insets and bars, block by block.
    let blocks = &layout.blocks;
    let mut prev: Option<usize> = None;
    let mut ix = 0;
    while ix < blocks.len() {
        if !shown(&blocks[ix]) {
            // A run of hidden lines up to the next block drawn.
            let start = ix;
            while ix < blocks.len() && !shown(&blocks[ix]) {
                ix += 1;
            }
            let run = blocks[start].lines.start..blocks[ix - 1].lines.end;
            let next = blocks.get(ix);
            let revealed = blocks[start..ix].iter().any(|b| b.revealed);
            let gap = match (prev.map(|p| &blocks[p]), next) {
                (Some(a), Some(b)) => gap_between(a, b, m),
                _ => 0.,
            };
            if next.is_none() || revealed {
                // Lines after the last block, or with the caret on them:
                // drawn as they are.
                for block in &blocks[start..ix] {
                    for line in block.lines.clone() {
                        if !block.revealed {
                            insets.push((line, quote_x(block, m)));
                        }
                    }
                }
                continue;
            }
            let quotes = match prev.map(|p| &blocks[p]) {
                Some(a) => blocks[start..ix]
                    .iter()
                    .map(|b| b.quote_depth)
                    .chain([a.quote_depth, next.map_or(0, |b| b.quote_depth)])
                    .min()
                    .unwrap_or(0),
                None => 0,
            };
            widgets.push(WidgetPlan {
                lines: run,
                inset: 0.,
                quotes,
                kind: WidgetKind::Gap(gap.max(NO_GAP)),
            });
            continue;
        }
        let b = &blocks[ix];
        if let Some(p) = prev {
            let a = &blocks[p];
            let gap = gap_between(a, b, m);
            let adjacent = a.lines.end == b.lines.start;
            if adjacent && gap > 0. {
                if b.quote_depth > a.quote_depth {
                    space.entry(a.lines.end - 1).or_default().1 += gap;
                } else {
                    space.entry(b.lines.start).or_default().0 += gap;
                }
            }
        }
        // The block's lines.
        let quote = quote_x(b, m);
        let x = quote + list_x(b);
        if layout
            .widgets
            .iter()
            .any(|w| w.lines.start == b.lines.start)
        {
            let w = layout
                .widgets
                .iter()
                .find(|w| w.lines.start == b.lines.start)
                .expect("found above");
            let source = widget_source(text, &lines, &w.lines, b.quote_depth);
            let height = estimate(&w.kind, w.lines.len(), m);
            widgets.push(WidgetPlan {
                lines: w.lines.clone(),
                inset: list_x(b),
                quotes: b.quote_depth,
                kind: WidgetKind::Markdown { source, height },
            });
        } else {
            for line in b.lines.clone() {
                if on_widget(line) {
                    continue;
                }
                if b.revealed {
                    // The source as written: its marks show where they are.
                    continue;
                }
                if x > 0. {
                    insets.push((line, x));
                }
                for depth in 0..b.quote_depth {
                    presentation.line_ornaments.push((
                        line,
                        Ornament::Bar {
                            x: gpui_kit::px(depth as f32 * m.quote_step()),
                            width: gpui_kit::px(QUOTE_BAR * m.zoom),
                            color: palette.quote.opacity(QUOTE_BAR_OPACITY),
                        },
                    ));
                }
            }
            match b.kind {
                BlockKind::Heading(level) => {
                    let scale = typography::heading(level).scale;
                    let size = m.heading_size(level);
                    let factor = m.snap(size * heading_line_height(level)) / (m.line() * scale);
                    for line in b.lines.clone() {
                        let underline = markers
                            .get(&line)
                            .is_some_and(|ms| ms.iter().any(|mk| mk.kind == MarkerKind::Underline));
                        if underline {
                            presentation.line_heights.push((line, NO_GAP / m.line()));
                            continue;
                        }
                        if scale != 1. {
                            presentation.line_scales.push((line, scale));
                        }
                        if factor != 1. {
                            presentation.line_heights.push((line, factor));
                        }
                    }
                }
                BlockKind::Rule if !b.revealed => {
                    let line = b.lines.start;
                    // A hairline, the row no taller than it.
                    presentation.line_heights.push((line, 1. / m.line()));
                    presentation.line_ornaments.push((
                        line,
                        Ornament::Rule {
                            thickness: gpui_kit::px(1.),
                            color: palette.border,
                        },
                    ));
                }
                _ => {}
            }
        }
        prev = Some(ix);
        ix += 1;
    }

    // The block marks: hidden, or drawn as bullets, numbers and boxes.
    for marker in &layout.markers {
        let range = marker.range.clone();
        match &marker.kind {
            MarkerKind::Bullet { glyph, .. } => {
                presentation
                    .replacements
                    .push(marker_column(range, glyph.to_string(), m, palette))
            }
            MarkerKind::Number { text, .. } => {
                presentation
                    .replacements
                    .push(marker_column(range, text.clone(), m, palette))
            }
            MarkerKind::Task { checked, .. } => {
                presentation.replacements.push(Replacement {
                    clickable: true,
                    column: Some(ReplacementColumn {
                        width: gpui_kit::px(m.task_column()),
                        gap: gpui_kit::px(0.),
                        hanging: true,
                    }),
                    ..Replacement::new(range, "")
                });
                let x = insets
                    .iter()
                    .rev()
                    .find(|(l, _)| *l == marker.line)
                    .map_or(0., |(_, x)| *x);
                let size = m.rem * CHECK_SIZE;
                presentation.line_ornaments.push((
                    marker.line,
                    Ornament::Check {
                        x: gpui_kit::px(x - m.task_column()),
                        size: gpui_kit::px(size),
                        radius: gpui_kit::px(CHECK_RADIUS),
                        border: gpui_kit::px(CHECK_BORDER),
                        border_color: if *checked {
                            palette.link
                        } else {
                            palette.muted
                        },
                        fill: checked.then_some(palette.link),
                        check: checked.then_some(palette.check),
                        // The preview centres it in a box the window's (rounded)
                        // line height tall.
                        row: Some(gpui_kit::px((m.body * typography::LINE_HEIGHT).round())),
                    },
                ));
            }
            MarkerKind::Hidden { .. }
            | MarkerKind::Quote { .. }
            | MarkerKind::Indent
            | MarkerKind::Rule
            | MarkerKind::Underline => presentation.concealed.push(range),
        }
    }
    presentation
        .concealed
        .extend(layout.conceal.iter().cloned());
    presentation.concealed.sort_by_key(|r| r.start);

    // Inline code off the caret, in the code face at its size.
    let concealed_ends: Vec<usize> = layout.conceal.iter().map(|r| r.end).collect();
    for (range, style) in &layout.styles {
        if *style != LiveStyle::Code || !concealed_ends.contains(&range.start) {
            continue;
        }
        if line_of(range.start) != line_of(range.end) || range.is_empty() {
            continue;
        }
        presentation.replacements.push(Replacement {
            style: HighlightStyle {
                color: palette.code_text,
                ..Default::default()
            },
            family: Some(palette.mono.clone()),
            size: CODE_SIZE,
            chip: Some(ReplacementChip {
                padding: gpui_kit::px(CODE_PADDING),
                background: palette.code_background,
                radius: palette.code_radius,
                height: ChipHeight::CapHeight,
                icon: None,
                dashed_bottom: None,
                whole_width: false,
                text_baseline: None,
            }),
            ..Replacement::new(range.clone(), text[range.clone()].to_owned())
        });
    }
    // How much taller each line's inline elements make it, above and below.
    let mut grown: HashMap<usize, (f32, f32)> = HashMap::new();
    // Inline formulas off the caret, typeset; a tall one makes its line
    // taller below, as the preview's line box grows.
    for (range, formula) in math {
        if line_of(range.start) != line_of(range.end) || range.is_empty() {
            continue;
        }
        presentation.replacements.push(Replacement {
            image: Some(ReplacementImage {
                image: formula.image.clone(),
                // Its size is authored, so laid out on whole device pixels.
                size: gpui_kit::size(
                    gpui_kit::px(m.snap(f32::from(formula.width))),
                    gpui_kit::px(m.snap(f32::from(formula.height))),
                ),
                baseline: formula.baseline,
            }),
            ..Replacement::new(range.clone(), "")
        });
        let (above, below) = m.grows(
            f32::from(formula.baseline),
            m.snap(f32::from(formula.height)),
        );
        let entry = grown.entry(line_of(range.start)).or_default();
        entry.0 = entry.0.max(above);
        entry.1 = entry.1.max(below);
    }
    // Wiki-links to no note, off the caret, with a dashed line under them
    // as the preview draws them (`wikilinks::UnresolvedLink`).
    for link in &layout.links {
        let LinkTarget::Wiki { title, .. } = &link.target else {
            continue;
        };
        let concealed = concealed_ends.contains(&link.text.start);
        if resolved(title) || !concealed || line_of(link.text.start) != line_of(link.text.end) {
            continue;
        }
        // The preview's element is a line tall with a border below it.
        let (baseline, height) = m.element_metrics((m.body * typography::LINE_HEIGHT).round() + 1.);
        let (above, below) = m.grows(baseline, height);
        let entry = grown.entry(line_of(link.text.start)).or_default();
        entry.0 = entry.0.max(above);
        entry.1 = entry.1.max(below);
        presentation.replacements.push(Replacement {
            style: HighlightStyle::color(palette.over_page(palette.link, UNRESOLVED_TEXT)),
            chip: Some(ReplacementChip {
                padding: gpui_kit::px(0.),
                background: gpui_kit::transparent_black(),
                radius: gpui_kit::px(0.),
                // The preview's element, its border at its bottom.
                height: ChipHeight::Baseline {
                    ascent: gpui_kit::px(baseline),
                    descent: gpui_kit::px(height - baseline),
                },
                icon: None,
                dashed_bottom: Some(palette.over_page(palette.link, UNRESOLVED_UNDERLINE)),
                whole_width: true,
                text_baseline: Some(gpui_kit::px(m.element_text_baseline())),
            }),
            ..Replacement::new(link.text.clone(), text[link.text.clone()].to_owned())
        });
    }
    // Due dates off the caret's lines, as chips.
    let revealed_line = |line: usize| {
        layout
            .blocks
            .iter()
            .any(|b| b.revealed && b.lines.contains(&line))
    };
    for chip in due {
        let line = line_of(chip.range.start);
        if revealed_line(line) || on_widget(line) || line != line_of(chip.range.end) {
            continue;
        }
        // The chip's size, room, height and corners as `due::DueChip`
        // sets them, in ems of its own size.
        let size = m.body * DUE_SIZE;
        presentation.replacements.push(Replacement {
            style: HighlightStyle {
                color: Some(chip.text),
                font_weight: Some(FontWeight::MEDIUM),
                ..Default::default()
            },
            size: DUE_SIZE,
            chip: Some(ReplacementChip {
                padding: gpui_kit::px(m.snap(size * 0.5)),
                background: chip.fill,
                radius: gpui_kit::px(size * 0.45),
                height: ChipHeight::Fixed(gpui_kit::px(m.snap(size * 1.55))),
                icon: Some(ChipIcon {
                    path: palette.calendar.clone(),
                    size: gpui_kit::px(m.snap(size * 0.95)),
                    gap: gpui_kit::px(m.snap(DUE_ICON_GAP * m.zoom)),
                }),
                dashed_bottom: None,
                whole_width: true,
                text_baseline: Some(gpui_kit::px(m.due_text_baseline(size))),
            }),
            ..Replacement::new(chip.range.clone(), chip.label.clone())
        });
    }
    for (&line, &(above, below)) in &grown {
        let entry = space.entry(line).or_default();
        entry.0 += above;
        entry.1 += below;
    }
    // Each block of text is a box of the preview's, rounded up to whole device
    // pixels: its rows and the room its inline elements add.
    for block in layout.blocks.iter().filter(|b| shown(b)) {
        let leaf = matches!(block.kind, BlockKind::Paragraph | BlockKind::Heading(_));
        if !leaf
            || layout
                .widgets
                .iter()
                .any(|w| w.lines.start == block.lines.start)
        {
            continue;
        }
        // The room inside it (between its lines) is counted as it is; the
        // room above its first line and below its last is a gap's, but for
        // what its elements add.
        let first = grown.get(&block.lines.start).map_or(0., |g| g.0);
        let last = grown.get(&(block.lines.end - 1)).map_or(0., |g| g.1);
        let extra = first + last;
        presentation
            .leaves
            .push((block.lines.clone(), gpui_kit::px(extra)));
    }
    presentation.replacements.sort_by_key(|r| r.range.start);

    presentation.line_space = {
        let mut items: Vec<(usize, Pixels, Pixels)> = space
            .into_iter()
            .map(|(line, (above, below))| (line, gpui_kit::px(above), gpui_kit::px(below)))
            .collect();
        items.sort_by_key(|(line, _, _)| *line);
        items
    };
    insets.sort_by_key(|(line, _)| *line);
    insets.dedup_by_key(|(line, _)| *line);
    presentation.line_insets = insets
        .into_iter()
        .filter(|(_, x)| *x > 0.)
        .map(|(line, x)| (line, gpui_kit::px(x)))
        .collect();
    presentation.line_ornaments.sort_by_key(|(line, _)| *line);
    // Rows as tall as the preview's, not rounded to whole pixels.
    presentation.line_height = Some(gpui_kit::px(m.line()));
    presentation.line_scales.sort_by_key(|(line, _)| *line);
    presentation.line_heights.sort_by_key(|(line, _)| *line);

    LivePlan {
        decorations: decorations(layout, palette, resolved),
        presentation,
        widgets,
    }
}

/// The room between blocks `a` and `b`, the next drawn after it.
fn gap_between(a: &LiveBlock, b: &LiveBlock, m: &LiveMetrics) -> f32 {
    if a.top != b.top || a.top.is_none() {
        flow_gap(Flow::top(a), Flow::top(b), m)
    } else {
        inner_gap(a, b, m)
    }
}

/// How far a block's quotes inset its text.
fn quote_x(block: &LiveBlock, m: &LiveMetrics) -> f32 {
    block.quote_depth as f32 * m.quote_step()
}

fn is_list_marker(kind: &MarkerKind) -> bool {
    matches!(
        kind,
        MarkerKind::Bullet { .. }
            | MarkerKind::Number { .. }
            | MarkerKind::Task { .. }
            | MarkerKind::Hidden { .. }
    )
}

/// A list marker drawn as `text`, hanging in its indent column like the
/// Preview's, right-aligned, in the muted colour.
fn marker_column(
    range: Range<usize>,
    text: String,
    m: &LiveMetrics,
    palette: &LivePalette,
) -> Replacement {
    Replacement {
        style: HighlightStyle::color(palette.muted),
        column: Some(ReplacementColumn {
            width: gpui_kit::px(m.indent()),
            gap: gpui_kit::px(m.marker_gap()),
            hanging: true,
        }),
        ..Replacement::new(range, text)
    }
}

/// Where each line of `text` starts.
fn line_starts(text: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(text.match_indices('\n').map(|(at, _)| at + 1))
        .collect()
}

/// The Markdown of a widget's whole `lines`, without the quote marks of
/// `quotes` quotes and the indent its first line has.
fn widget_source(text: &str, starts: &[usize], lines: &Range<usize>, quotes: usize) -> String {
    let start = starts[lines.start];
    let end = starts
        .get(lines.end)
        .map_or(text.len(), |next| next.saturating_sub(1));
    let mut source = text[start..end.max(start)]
        .trim_end_matches('\r')
        .to_owned();
    for _ in 0..quotes {
        source = crate::preview::unquote(&source);
    }
    let indent = source.lines().next().map_or(0, |first| {
        first.len() - first.trim_start_matches([' ', '\t']).len()
    });
    if indent == 0 {
        return source;
    }
    source
        .lines()
        .map(|line| {
            let strip = line.len() - line.trim_start_matches([' ', '\t']).len();
            &line[strip.min(indent)..]
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// About how tall a widget is, until it is measured.
fn estimate(kind: &BlockKind, lines: usize, m: &LiveMetrics) -> f32 {
    let rows = lines.max(1) as f32;
    match kind {
        BlockKind::Code { .. } => {
            typography::CODE_TOP
                + typography::CODE_PADDING_BOTTOM
                + rows.max(2.) * typography::CODE_SIZE * typography::CODE_LINE_HEIGHT
        }
        BlockKind::FrontMatter => NO_GAP,
        _ => rows * m.line(),
    }
}

/// The text decorations for `layout`'s styles and spans: sorted, disjoint
/// and never empty, each part of the text in the styles of every span over
/// it, an inner (shorter) span's winning where two set the same property.
pub fn decorations(
    layout: &LiveLayout,
    palette: &LivePalette,
    resolved: &dyn Fn(&str) -> bool,
) -> Vec<TextDecoration> {
    let link_style = |range: &Range<usize>| {
        let wiki = layout.links.iter().find(|l| l.text == *range);
        match wiki.map(|l| &l.target) {
            Some(LinkTarget::Wiki { title, .. }) => palette.link(resolved(title)),
            _ => palette.link(true),
        }
    };
    let spans: Vec<(Range<usize>, HighlightStyle)> = layout
        .styles
        .iter()
        .map(|(range, style)| {
            let highlight = match style {
                LiveStyle::Link => link_style(range),
                other => palette.style(*other),
            };
            (range.clone(), highlight)
        })
        .chain(
            layout
                .spans
                .iter()
                .filter_map(|(range, span)| Some((range.clone(), palette.span(*span)?))),
        )
        .filter(|(range, _)| range.start < range.end)
        .collect();
    merge(spans)
}

/// `spans` as sorted, disjoint decorations, an inner (shorter) span's style
/// winning where two set the same property.
fn merge(spans: Vec<(Range<usize>, HighlightStyle)>) -> Vec<TextDecoration> {
    let mut bounds: Vec<usize> = spans
        .iter()
        .flat_map(|(range, _)| [range.start, range.end])
        .collect();
    bounds.sort_unstable();
    bounds.dedup();
    let mut order: Vec<usize> = (0..spans.len()).collect();
    order.sort_by_key(|&ix| spans[ix].0.start);
    let mut next = 0;
    // The spans over the current part, outermost first.
    let mut open: Vec<usize> = Vec::new();
    let mut out: Vec<TextDecoration> = Vec::new();
    for pair in bounds.windows(2) {
        let (start, end) = (pair[0], pair[1]);
        open.retain(|&ix| spans[ix].0.end > start);
        while next < order.len() && spans[order[next]].0.start <= start {
            open.push(order[next]);
            next += 1;
        }
        if open.is_empty() {
            continue;
        }
        open.sort_by_key(|&ix| std::cmp::Reverse(spans[ix].0.len()));
        let style = open.iter().fold(HighlightStyle::default(), |style, &ix| {
            style.highlight(spans[ix].1)
        });
        match out.last_mut() {
            Some(last) if last.range.end == start && last.style == style => last.range.end = end,
            _ => out.push(TextDecoration::new(start..end, style)),
        }
    }
    out
}

/// The caret's and the active selection's ranges, as the engine reveals
/// them.
fn reveal_ranges(state: &EditorState) -> Vec<Range<usize>> {
    let cursor = state.cursor();
    vec![state.selected_range(), cursor..cursor]
}

/// Typesets an inline formula (without its `$`s) for text at a size, as the
/// preview does; `None` until it is drawn.
pub type MathTypesetter = Rc<dyn Fn(&str, Pixels, &App) -> Option<TypesetMath>>;

/// The formulas of `layout` off the caret, as `$…$` the preview typesets,
/// with their typesetting where it is ready.
fn typeset_math(
    text: &str,
    layout: &LiveLayout,
    typeset: &MathTypesetter,
    m: &LiveMetrics,
    cx: &App,
) -> Vec<(Range<usize>, TypesetMath)> {
    layout
        .spans
        .iter()
        .filter(|(_, span)| *span == LiveSpan::InlineMath)
        .filter_map(|(inner, _)| {
            // Its `$`s are hidden off the caret.
            let open = layout.conceal.iter().find(|c| c.end == inner.start)?;
            let marks = inner.start - open.start;
            let span = text.get(open.start..inner.end + marks)?;
            let next = text
                .get(inner.end + marks..)
                .and_then(|rest| rest.chars().next());
            if !kanso_render::is_inline_math(span, next) {
                return None;
            }
            let formula = typeset(text.get(inner.clone())?, gpui_kit::px(m.body), cx)?;
            Some((inner.clone(), formula))
        })
        .collect()
}

/// Draws a widget's Markdown: the Preview's renderer, with its extensions.
pub type MarkdownRenderer = Rc<dyn Fn(&Entity<TextViewState>, &mut Window, &mut App) -> AnyElement>;

/// What Live needs from the note's editor to draw: the Preview's renderer
/// for widgets, what the Preview is handed for a widget's Markdown (its
/// wiki-links and due dates rewritten), whether a wiki-link resolves, and
/// the due dates as chips.
pub struct LiveContext<'a> {
    pub render: MarkdownRenderer,
    pub preview_source: &'a dyn Fn(&str) -> String,
    pub resolved: &'a dyn Fn(&str) -> bool,
    pub due: Vec<DueLabel>,
    pub metrics: LiveMetrics,
    pub math: MathTypesetter,
}

/// What a layout was worked out for: the text's version and length, the
/// caret and selection, the metrics and the theme's colours.
type Shown = (u64, usize, Vec<Range<usize>>, LiveMetrics, LivePalette);

/// The editor's Live presentation and the decorations that go with it.
#[derive(Default)]
pub struct LiveMarks {
    collection: Option<TextDecorationCollection>,
    /// What the presentation in the editor was worked out for, and the
    /// presentation the editor kept.
    shown: Option<(Shown, EditorPresentation)>,
    /// The widgets' text views, by what they show.
    views: HashMap<u64, (Entity<TextViewState>, Subscription)>,
}

impl LiveMarks {
    /// Draws `body` in Live mode, working the layout out again if the text
    /// (`version`), the caret or selection, the sizes or the theme changed
    /// since, or the editor dropped the presentation (a new text).
    pub fn refresh(
        &mut self,
        body: &Entity<EditorState>,
        version: u64,
        live: LiveContext<'_>,
        cx: &mut App,
    ) {
        let palette = LivePalette::from_theme(cx);
        let state = body.read(cx);
        let reveal = reveal_ranges(state);
        let key: Shown = (
            version,
            state.text().len(),
            reveal.clone(),
            live.metrics,
            palette.clone(),
        );
        if let Some((shown, kept)) = &self.shown
            && *shown == key
            && *kept == state.editor_presentation()
        {
            return;
        }
        let text = state.value();
        let layout = live_layout(&text, &reveal);
        let math = typeset_math(&text, &layout, &live.math, &live.metrics, cx);
        let LivePlan {
            mut presentation,
            widgets,
            decorations,
        } = plan(
            &text,
            &layout,
            &live.metrics,
            &palette,
            live.resolved,
            &live.due,
            &math,
        );
        let mut views = std::mem::take(&mut self.views);
        for widget in &widgets {
            let key = widget.key();
            let renderer: BlockWidgetRenderer = match &widget.kind {
                WidgetKind::Gap(height) => {
                    let (height, quotes, m) = (*height, widget.quotes, live.metrics);
                    let bar = palette.quote.opacity(QUOTE_BAR_OPACITY);
                    Rc::new(move |_, _| {
                        quoted(div().h(gpui_kit::px(height)), quotes, bar, &m).into_any_element()
                    })
                }
                WidgetKind::Markdown { source, .. } => {
                    let view = match views.remove(&key) {
                        Some(view) => view,
                        None => {
                            let markdown = (live.preview_source)(source);
                            let view = cx.new(|cx| TextViewState::markdown(&markdown, cx));
                            // It parses and lays out on its own: the editor
                            // draws it again.
                            let editor = body.downgrade();
                            let watch = cx.observe(&view, move |_, cx| {
                                editor.update(cx, |_, cx| cx.notify()).ok();
                            });
                            (view, watch)
                        }
                    };
                    let state = view.0.clone();
                    self.views.insert(key, view);
                    let render = live.render.clone();
                    let (inset, quotes, m) = (widget.inset, widget.quotes, live.metrics);
                    let bar = palette.quote.opacity(QUOTE_BAR_OPACITY);
                    Rc::new(move |window, cx| {
                        let inner = div()
                            .w_full()
                            .pl(gpui_kit::px(inset))
                            .child(render(&state, window, cx));
                        quoted(inner, quotes, bar, &m).into_any_element()
                    })
                }
            };
            presentation.block_widgets.push(BlockWidget {
                lines: widget.lines.clone(),
                height: gpui_kit::px(widget.height().max(NO_GAP)),
                key,
                render: renderer,
            });
        }
        let kept = body.update(cx, |state, cx| {
            state.set_presentation(Some(presentation), cx);
            state.editor_presentation()
        });
        self.shown = Some((key, kept));
        match &self.collection {
            Some(collection) => collection.set(decorations, cx),
            None if decorations.is_empty() => {}
            None => {
                self.collection = Some(body.update(cx, |state, cx| {
                    state.create_decorations_collection(decorations, cx)
                }));
            }
        }
    }

    /// Math and diagrams finished drawing: the widgets showing them lay out
    /// again, and the formulas are worked out again at the next refresh.
    pub fn drawings_ready(&mut self, body: &Entity<EditorState>, cx: &mut App) {
        self.shown = None;
        if self.views.is_empty() {
            return;
        }
        for (view, _) in self.views.values() {
            view.update(cx, |state, cx| state.invalidate_inline_layout(cx));
        }
        body.update(cx, |_, cx| cx.notify());
    }

    /// Draws `body` as plain source again.
    pub fn clear(&mut self, body: &Entity<EditorState>, cx: &mut App) {
        self.views.clear();
        if self.shown.take().is_none() {
            return;
        }
        body.update(cx, |state, cx| state.set_presentation(None, cx));
        if let Some(collection) = &self.collection {
            collection.clear(cx);
        }
    }

    /// Whether a Live presentation is drawn.
    #[cfg(test)]
    pub fn is_shown(&self) -> bool {
        self.shown.is_some()
    }
}

/// `inner` beside the bars of `quotes` quotes, as the Preview draws a quote:
/// a bar, then its text past the room beside it.
fn quoted(inner: Div, quotes: usize, bar: Hsla, m: &LiveMetrics) -> Div {
    let mut el = inner;
    for _ in 0..quotes {
        el = div()
            .w_full()
            .min_w_0()
            .border_l(gpui_kit::px(QUOTE_BAR * m.zoom))
            .border_color(bar)
            .pl(gpui_kit::px(QUOTE_PAD * m.zoom))
            .child(el);
    }
    el
}

/// The source range a click on a task's box hits names the task by the byte
/// its item starts at: its bullet or number, past the indent.
pub fn task_item_start(text: &str, range: &Range<usize>) -> Option<usize> {
    let bytes = text.as_bytes().get(range.clone())?;
    let skip = bytes
        .iter()
        .take_while(|b| matches!(b, b' ' | b'\t'))
        .count();
    (skip < bytes.len()).then_some(range.start + skip)
}

/// The due dates of `text` as chips, at `now`, in `colors`.
pub fn due_labels(
    text: &str,
    now: kanso_core::dates::DateTime,
    colors: &crate::theme::Colors,
) -> Vec<DueLabel> {
    crate::due::marks(text, now)
        .into_iter()
        .filter_map(|(range, tone)| {
            let token = text.get(range.clone())?;
            let due = kanso_core::tasks::Due::parse(token.strip_prefix('@')?)?;
            let checked = tone == crate::due::Tone::Done;
            let (text_color, fill) = crate::due::chip_colors(tone, colors);
            // A repeat after the date goes into its chip.
            let line_end = text[range.end..]
                .find('\n')
                .map_or(text.len(), |at| range.end + at);
            let repeat = kanso_core::repeat::repeat_token(&text[range.start..line_end]);
            let end = repeat
                .as_ref()
                .map_or(range.end, |(words, _)| range.start + words.end);
            Some(DueLabel {
                range: range.start..end,
                label: crate::due::repeat_label(
                    crate::due::chip_label(due, checked, now),
                    repeat.map(|(_, repeat)| repeat),
                ),
                text: text_color,
                fill,
            })
        })
        .collect()
}

#[cfg(test)]
#[path = "live_marks_tests.rs"]
mod tests;
