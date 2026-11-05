//! A note as typed blocks, for a preview the app draws itself (#48): the
//! same note the page in the app ([`html::app`](crate::html::app)) shows,
//! read from the same syntax tree, with the same addresses on its links.
//!
//! The rules are the page's, so a tap means the same thing on both:
//!
//! - the text is parsed with [`html::parse_options`];
//! - headings carry the `id`s the page gives them ([`html::headings`]);
//! - a task box carries its place in the note's task list where the page
//!   links it ([`html::task_boxes`]), and none where it doesn't;
//! - a code block's copy number is its `kanso-copy:` number;
//! - only the links the page keeps are links ([`html::is_allowed_link`]);
//!   `#…` links point at a heading's `id` or are plain text;
//! - math, diagrams and drawings are drawn by the same jobs, and one that
//!   fails is its code with the page's badge;
//! - query views and due chips come from the same [`AppMode`];
//! - raw HTML is shown as text, except an attached image's `<img>`.
//!
//! Pure: what the note needs from outside comes in as arguments.

use std::collections::HashMap;

use markdown::mdast::{AlignKind, Node};

use crate::html::app::{self, AppMode, DUE_LINK_PREFIX, DueTone, HeadingId, QueryResult};
use crate::html::{self, Attachments, Highlighter};
use crate::{Failure, Job, Kind, Outcome, Palette, Svg, escape, render_blocking};

/// A block of the note, in document order.
#[derive(Clone, Debug, PartialEq)]
pub enum Block {
    /// `level` 1 to 6; `id` is the page's, for `#id` links.
    Heading {
        level: u8,
        id: String,
        inlines: Vec<Inline>,
    },
    Paragraph {
        inlines: Vec<Inline>,
    },
    /// `start` is the first number of an ordered list (1 for a bullet list);
    /// a tight list has no gaps between its items.
    List {
        ordered: bool,
        start: u32,
        tight: bool,
        items: Vec<ListItem>,
    },
    Quote {
        blocks: Vec<Block>,
    },
    /// Code, in runs covering all of it in order; `copy` is its
    /// `kanso-copy:` number.
    Code {
        language: Option<String>,
        runs: Vec<CodeRun>,
        copy: u32,
    },
    Rule,
    /// A table: the header row's cells and each row's cells.
    Table {
        aligns: Vec<Align>,
        head: Vec<Vec<Inline>>,
        rows: Vec<Vec<Vec<Inline>>>,
    },
    /// Math, a diagram, a drawing or a plugin's drawing, set on its own.
    Figure {
        kind: FigureKind,
        svg: Svg,
    },
    /// An Excalidraw scene with nothing in it yet.
    EmptyDrawing,
    /// A `canvas` fence: the canvas note it names (and the group, if any),
    /// and what it shows. A locked canvas is a placeholder, never drawn.
    Canvas {
        note: String,
        group: Option<String>,
        shown: crate::canvas::embed::Shown,
    },
    /// A source that couldn't be drawn: its code, with a badge saying why
    /// and the details. `copy` is its `kanso-copy:` number when it is a fence.
    Failed {
        badge: String,
        detail: String,
        language: Option<String>,
        code: String,
        copy: Option<u32>,
    },
    /// A `query` fence's table; no rows is "No notes match."
    Query {
        columns: Vec<String>,
        rows: Vec<QueryRow>,
    },
    /// A `query` fence that can't be read: its text and why.
    QueryInvalid {
        code: String,
        message: String,
    },
    /// Raw HTML, shown as the text it is.
    Html {
        text: String,
    },
}

/// Whether raw HTML is nothing but whole `<!-- … -->` comments and
/// whitespace. Pure.
fn only_comments(raw: &str) -> bool {
    let mut rest = raw.trim();
    if rest.is_empty() {
        return false;
    }
    while !rest.is_empty() {
        let Some(after) = rest.strip_prefix("<!--") else {
            return false;
        };
        let Some(end) = after.find("-->") else {
            return false;
        };
        rest = after[end + 3..].trim_start();
    }
    true
}

/// An item of a list: its task box, if it is a task, and its blocks.
#[derive(Clone, Debug, PartialEq)]
pub struct ListItem {
    pub task: Option<TaskBox>,
    pub blocks: Vec<Block>,
}

/// A task box: whether it is ticked, and its place in the note's task list
/// (`kanso-task:<index>`), `None` for a box that must not be ticked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskBox {
    pub checked: bool,
    pub index: Option<u32>,
}

/// A run of a code block in one style. `color` is `0xRRGGBB`, `None` for
/// the code's own color.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeRun {
    pub text: String,
    pub color: Option<u32>,
    pub bold: bool,
    pub italic: bool,
}

/// How a table column is aligned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    None,
    Left,
    Center,
    Right,
}

/// What a figure is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FigureKind {
    Math,
    Diagram,
    Drawing,
    Plugin,
}

/// A row of a query view: the note's address, its title ("Untitled" when
/// it has none) and a cell for each column after the title.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryRow {
    pub url: String,
    pub title: String,
    pub cells: Vec<String>,
}

/// A piece of a line of text.
#[derive(Clone, Debug, PartialEq)]
pub enum Inline {
    /// Text in the styles around it; `code` is inline code.
    Text {
        text: String,
        bold: bool,
        italic: bool,
        strike: bool,
        code: bool,
    },
    /// A link the page keeps, by the address it carries.
    Link { url: String, inlines: Vec<Inline> },
    /// A due date as a chip.
    Due {
        url: String,
        label: String,
        title: String,
        tone: DueTone,
    },
    /// Inline math, drawn; `svg.baseline` sets it on the line.
    Math { svg: Svg },
    /// An attached image; `width` in points when the note gives one.
    Image {
        content_type: String,
        bytes: Vec<u8>,
        alt: String,
        width: Option<u32>,
    },
    /// What can't be shown: an image that isn't attached (its alt text) or a
    /// wiki-link to nothing (its label).
    Missing { text: String },
    /// A hard line break.
    Break,
}

/// The styles text inherits.
#[derive(Clone, Copy, Default)]
struct Style {
    bold: bool,
    italic: bool,
    strike: bool,
}

/// `markdown` as blocks, the way the page in the app shows it: `mode` gives
/// the task slots, query views and due chips; `attachments` the attached
/// files; `highlight` colors code. Pure (drawings are cached).
pub fn blocks(
    markdown: &str,
    attachments: Attachments<'_>,
    palette: &Palette,
    mode: &AppMode<'_>,
    highlight: Option<&dyn Highlighter>,
) -> Vec<Block> {
    blocks_with_sources(markdown, attachments, palette, mode, highlight)
        .into_iter()
        .map(|(block, _)| block)
        .collect()
}

/// [`blocks`], each with the bytes of `markdown` it is drawn from: the
/// top-level node (a heading, a list, a fence) as the parser spans it. The
/// footnotes listed after the note, and the paragraphs of text Markdown
/// can't parse, come from nowhere in particular: `None`. Pure.
pub fn blocks_with_sources(
    markdown: &str,
    attachments: Attachments<'_>,
    palette: &Palette,
    mode: &AppMode<'_>,
    highlight: Option<&dyn Highlighter>,
) -> Vec<(Block, Option<std::ops::Range<usize>>)> {
    let Ok(root) = crate::markdown_safe::to_mdast(markdown, &html::parse_options()) else {
        return plain(markdown)
            .into_iter()
            .map(|block| (block, None))
            .collect();
    };
    let facts = app::facts(&root);
    let tasks = html::task_boxes(&root, markdown, mode.tasks)
        .into_iter()
        .map(|task| (task.offset, task.slot))
        .collect();
    let mut walk = Walk {
        source: markdown,
        attachments,
        palette: *palette,
        mode,
        highlight,
        codes: facts.code_offsets(),
        headings: app::heading_ids(&facts),
        next_heading: 0,
        tasks,
        definitions: HashMap::new(),
        footnotes: HashMap::new(),
        cited: Vec::new(),
        depth: 0,
    };
    walk.gather(&root);
    let mut out = Vec::new();
    let mut sourced = Vec::new();
    for node in root.children().into_iter().flatten() {
        walk.block(node, &mut out);
        let source = node
            .position()
            .map(|p| p.start.offset..p.end.offset)
            .filter(|range| markdown.get(range.clone()).is_some());
        sourced.extend(out.drain(..).map(|block| (block, source.clone())));
    }
    walk.footnote_list(&mut out);
    sourced.extend(out.into_iter().map(|block| (block, None)));
    sourced
}

/// What the page shows for text Markdown can't parse: its paragraphs, line
/// by line.
fn plain(markdown: &str) -> Vec<Block> {
    markdown
        .split("\n\n")
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut inlines = Vec::new();
            for (n, line) in p.split('\n').enumerate() {
                if n > 0 {
                    inlines.push(Inline::Break);
                }
                inlines.push(text(line, Style::default(), false));
            }
            Block::Paragraph { inlines }
        })
        .collect()
}

fn text(text: &str, style: Style, code: bool) -> Inline {
    Inline::Text {
        text: text.to_owned(),
        bold: style.bold,
        italic: style.italic,
        strike: style.strike,
        code,
    }
}

/// A URL as the page's link carries it: markdown-rs's own sanitizing
/// (unsafe characters percent-encoded, sequences already encoded kept,
/// HTML characters escaped), read back the way the page reads its `href`.
/// Pure.
pub(crate) fn page_url(value: &str) -> String {
    app::unescape(&markdown::sanitize(value))
}

/// How many quotes and lists deep the blocks go; anything deeper is shown
/// in the deepest one, flattened, so a pathological note can't exhaust the
/// stack.
const MAX_DEPTH: usize = 64;

/// A fence's language as the page labels it.
fn language(lang: &Option<String>) -> Option<String> {
    lang.as_deref()
        .map(str::trim)
        .filter(|lang| !lang.is_empty())
        .map(str::to_owned)
}

fn failed(failure: &Failure, language: Option<String>, code: &str, copy: Option<u32>) -> Block {
    Block::Failed {
        badge: failure.badge(),
        detail: failure.detail().to_owned(),
        language,
        code: code.to_owned(),
        copy,
    }
}

struct Walk<'a> {
    source: &'a str,
    attachments: Attachments<'a>,
    palette: Palette,
    mode: &'a AppMode<'a>,
    highlight: Option<&'a dyn Highlighter>,
    /// Where each code block starts: its place is its copy number.
    codes: Vec<usize>,
    headings: Vec<HeadingId>,
    next_heading: usize,
    /// Each task item's slot, by where it starts.
    tasks: HashMap<usize, Option<usize>>,
    /// Link reference definitions: the first URL for each label.
    definitions: HashMap<String, String>,
    /// Footnote definitions, by label.
    footnotes: HashMap<String, Node>,
    /// The footnotes cited, in the order first cited.
    cited: Vec<String>,
    /// How many quotes and lists the walk is in.
    depth: usize,
}

impl<'a> Walk<'a> {
    /// Finds the note's link reference and footnote definitions.
    fn gather(&mut self, node: &Node) {
        match node {
            Node::Definition(definition) => {
                // markdown-rs gives the label normalized (case and spacing).
                self.definitions
                    .entry(definition.identifier.clone())
                    .or_insert_with(|| definition.url.clone());
            }
            Node::FootnoteDefinition(definition) => {
                self.footnotes
                    .entry(definition.identifier.clone())
                    .or_insert_with(|| node.clone());
            }
            _ => {}
        }
        for child in node.children().into_iter().flatten() {
            self.gather(child);
        }
    }

    /// The cited footnotes, numbered, after the note.
    fn footnote_list(&mut self, out: &mut Vec<Block>) {
        let mut items = Vec::new();
        let mut n = 0;
        // Each is listed once: once cited it is looked up by its number.
        while n < self.cited.len() {
            if let Some(Node::FootnoteDefinition(definition)) =
                self.footnotes.remove(&self.cited[n])
            {
                let blocks = self.blocks_of(&definition.children);
                items.push(ListItem { task: None, blocks });
            }
            n += 1;
        }
        if !items.is_empty() {
            out.push(Block::Rule);
            out.push(Block::List {
                ordered: true,
                start: 1,
                tight: false,
                items,
            });
        }
    }

    fn written(&self, node: &Node) -> &'a str {
        node.position()
            .and_then(|p| self.source.get(p.start.offset..p.end.offset))
            .unwrap_or_default()
    }

    fn copy_index(&self, node: &Node) -> Option<u32> {
        let start = node.position()?.start.offset;
        self.codes
            .iter()
            .position(|&at| at == start)
            .map(|n| n as u32)
    }

    fn draw(&self, kind: Kind, code: &str) -> Outcome {
        render_blocking(&Job::new(kind, code, self.palette))
            .as_ref()
            .clone()
    }

    fn children(&mut self, node: &Node) -> Vec<Block> {
        match node.children() {
            Some(children) => self.blocks_of(children),
            None => Vec::new(),
        }
    }

    fn blocks_of(&mut self, nodes: &[Node]) -> Vec<Block> {
        let mut out = Vec::new();
        for node in nodes {
            self.block(node, &mut out);
        }
        out
    }

    /// A drawing as a figure, or its code and why when it failed.
    fn figure(
        &self,
        outcome: Outcome,
        kind: FigureKind,
        language: Option<String>,
        code: &str,
        copy: Option<u32>,
    ) -> Block {
        match outcome {
            Ok(svg) => Block::Figure { kind, svg },
            Err(failure) => failed(&failure, language, code, copy),
        }
    }

    fn block(&mut self, node: &Node, out: &mut Vec<Block>) {
        let written = self.written(node);
        match node {
            Node::Blockquote(_) | Node::List(_) if self.depth >= MAX_DEPTH => {
                self.flattened(node, out);
            }
            Node::Heading(heading) => {
                let id = self
                    .headings
                    .get(self.next_heading)
                    .map(|found| found.id.clone())
                    .unwrap_or_default();
                self.next_heading += 1;
                let inlines = self.inlines(&heading.children, Style::default());
                out.push(Block::Heading {
                    level: heading.depth,
                    id,
                    inlines,
                });
            }
            Node::Paragraph(_) if html::display_math_paragraph(node, written).is_some() => {
                let formula = html::display_math_paragraph(node, written).unwrap_or_default();
                let outcome = self.draw(Kind::DisplayMath, formula);
                out.push(self.figure(outcome, FigureKind::Math, None, written, None));
            }
            Node::Paragraph(_) if html::attached_scene(node).is_some() => {
                let scene = html::attached_scene(node)
                    .and_then(|id| (self.attachments)(&id))
                    .and_then(|(_, bytes)| String::from_utf8(bytes).ok());
                out.push(match scene {
                    Some(scene) if app::is_empty_scene(&scene) => Block::EmptyDrawing,
                    Some(scene) => {
                        let outcome = self.draw(Kind::Excalidraw, &scene);
                        self.figure(outcome, FigureKind::Drawing, None, written, None)
                    }
                    None => failed(
                        &Failure::Invalid("The drawing's file is missing.".into()),
                        None,
                        written,
                        None,
                    ),
                });
            }
            Node::Paragraph(paragraph) => {
                let inlines = self.inlines(&paragraph.children, Style::default());
                out.push(Block::Paragraph { inlines });
            }
            Node::Math(math) => {
                let outcome = self.draw(Kind::DisplayMath, &math.value);
                out.push(self.figure(outcome, FigureKind::Math, None, written, None));
            }
            Node::Code(code) => out.push(self.code(node, code)),
            Node::List(list) => {
                let mut tight = !list.spread;
                let mut items = Vec::new();
                for child in &list.children {
                    let Node::ListItem(item) = child else {
                        continue;
                    };
                    tight &= !item.spread;
                    let task = item.checked.map(|checked| TaskBox {
                        checked,
                        index: item
                            .position
                            .as_ref()
                            .and_then(|p| self.tasks.get(&p.start.offset).copied().flatten())
                            .map(|slot| slot as u32),
                    });
                    self.depth += 1;
                    let blocks = self.blocks_of(&item.children);
                    self.depth -= 1;
                    items.push(ListItem { task, blocks });
                }
                out.push(Block::List {
                    ordered: list.ordered,
                    start: if list.ordered {
                        list.start.unwrap_or(1)
                    } else {
                        1
                    },
                    tight,
                    items,
                });
            }
            Node::Blockquote(_) => {
                self.depth += 1;
                let blocks = self.children(node);
                self.depth -= 1;
                out.push(Block::Quote { blocks });
            }
            Node::ThematicBreak(_) => out.push(Block::Rule),
            Node::Table(table) => {
                // Every row has a cell for each column, as in the page: short
                // rows are filled with empty cells, extra cells dropped.
                let columns = table.align.len();
                let mut rows = table.children.iter().filter_map(|row| match row {
                    Node::TableRow(row) => Some(row),
                    _ => None,
                });
                let mut cells = |row: &markdown::mdast::TableRow| -> Vec<Vec<Inline>> {
                    let mut cells: Vec<Vec<Inline>> = row
                        .children
                        .iter()
                        .take(columns)
                        .map(|cell| match cell.children() {
                            Some(children) => self.inlines(children, Style::default()),
                            None => Vec::new(),
                        })
                        .collect();
                    cells.resize_with(columns, Vec::new);
                    cells
                };
                let head = rows
                    .next()
                    .map(&mut cells)
                    .unwrap_or_else(|| vec![Vec::new(); columns]);
                let body = rows.map(&mut cells).collect();
                out.push(Block::Table {
                    aligns: table
                        .align
                        .iter()
                        .map(|align| match align {
                            AlignKind::Left => Align::Left,
                            AlignKind::Center => Align::Center,
                            AlignKind::Right => Align::Right,
                            AlignKind::None => Align::None,
                        })
                        .collect(),
                    head,
                    rows: body,
                });
            }
            // A comment (such as a recording's transcript markers) is
            // hidden, as in a browser.
            Node::Html(raw) if only_comments(&raw.value) => {}
            Node::Html(raw) => match self.html_image(&raw.value) {
                Some(image) => out.push(Block::Paragraph {
                    inlines: vec![image],
                }),
                None => out.push(Block::Html {
                    text: raw.value.clone(),
                }),
            },
            // Shown after the note, numbered, where cited; links say nothing.
            Node::FootnoteDefinition(_) | Node::Definition(_) => {}
            Node::Yaml(_) | Node::Toml(_) => {}
            _ => {
                let blocks = self.children(node);
                out.extend(blocks);
            }
        }
    }

    /// The blocks in `node`, a quote or list past [`MAX_DEPTH`], at this
    /// depth: its quotes, lists and items opened without recursing, each
    /// other block in order.
    fn flattened(&mut self, node: &Node, out: &mut Vec<Block>) {
        let mut stack = vec![node];
        while let Some(next) = stack.pop() {
            match next {
                Node::Blockquote(_) | Node::List(_) | Node::ListItem(_) => {
                    stack.extend(next.children().into_iter().flatten().rev());
                }
                other => self.block(other, out),
            }
        }
    }

    /// A code node: a drawing, a query view or code, as the page has it.
    fn code(&self, node: &Node, code: &markdown::mdast::Code) -> Block {
        let lang = language(&code.lang);
        let copy = self.copy_index(node);
        let is = |want: &str| html::fence_is(&code.lang, want);
        if is("math") {
            let outcome = self.draw(Kind::DisplayMath, &code.value);
            return self.figure(outcome, FigureKind::Math, lang, &code.value, copy);
        }
        if is("mermaid") {
            let outcome = self.draw(Kind::Mermaid, &code.value);
            return self.figure(outcome, FigureKind::Diagram, lang, &code.value, copy);
        }
        if is("excalidraw") {
            if app::is_empty_scene(&code.value) {
                return Block::EmptyDrawing;
            }
            let outcome = self.draw(Kind::Excalidraw, &code.value);
            return self.figure(outcome, FigureKind::Drawing, lang, &code.value, copy);
        }
        if is("canvas")
            && let Some(notes) = self.mode.notes
            && let Some(embed) = kanso_core::canvas::embed::parse(&code.value)
        {
            let shown = crate::canvas::embed::show(&embed, notes, &self.palette);
            return Block::Canvas {
                note: embed.note,
                group: embed.group,
                shown,
            };
        }
        if let Some(plugin) = code
            .lang
            .as_deref()
            .filter(|lang| crate::is_plugin_language(lang))
        {
            let outcome = render_blocking(&Job::plugin(
                plugin.trim(),
                code.value.as_str(),
                self.palette,
            ))
            .as_ref()
            .clone();
            return self.figure(outcome, FigureKind::Plugin, lang, &code.value, copy);
        }
        if let Some(query) = self.mode.query
            && app::is_query_fence(code.lang.as_deref())
        {
            return match query.run(&code.value) {
                QueryResult::Invalid { message } => Block::QueryInvalid {
                    code: code.value.clone(),
                    message,
                },
                QueryResult::Table(table) => Block::Query {
                    columns: table.columns,
                    rows: table
                        .rows
                        .into_iter()
                        .map(|row| {
                            let title = row.title.trim();
                            QueryRow {
                                url: app::note_address(&row.id),
                                title: if title.is_empty() { "Untitled" } else { title }.to_owned(),
                                cells: row.cells,
                            }
                        })
                        .collect(),
                },
            };
        }
        let spans = self
            .highlight
            .and_then(|highlight| {
                highlight.highlight(code.lang.as_deref().unwrap_or_default(), &code.value)
            })
            .unwrap_or_default();
        // Every code node is in the list it is numbered by: markdown-rs
        // gives each node its position, and `facts` lists each code node's.
        debug_assert!(copy.is_some(), "a code block without a copy number");
        Block::Code {
            language: lang,
            runs: code_runs(&code.value, &spans),
            copy: copy.unwrap_or_default(),
        }
    }

    /// An `<img>` tag naming an attachment, as the image (or its alt text
    /// when the file is gone); `None` for any other HTML.
    fn html_image(&self, raw: &str) -> Option<Inline> {
        let (src, alt, width) = html::img_tag(raw)?;
        let id = html::attachment_id(&src)?;
        Some(self.attached_image(&id, &alt, width))
    }

    fn attached_image(&self, id: &str, alt: &str, width: Option<u32>) -> Inline {
        let safe_type = |t: &str| t.starts_with("image/") && !t.contains(['"', '<', '>']);
        match (self.attachments)(id) {
            Some((content_type, bytes)) if safe_type(&content_type) => Inline::Image {
                content_type,
                bytes,
                alt: alt.to_owned(),
                width,
            },
            _ => Inline::Missing {
                text: alt.to_owned(),
            },
        }
    }

    fn inlines(&mut self, nodes: &[Node], style: Style) -> Vec<Inline> {
        let mut out = Vec::new();
        for node in nodes {
            self.inline(node, style, &mut out);
        }
        out
    }

    fn inline(&mut self, node: &Node, style: Style, out: &mut Vec<Inline>) {
        match node {
            Node::Text(t) => {
                // Line endings are spaces, as in the page.
                let value = t.value.replace("\r\n", " ").replace(['\n', '\r'], " ");
                push_text(&value, style, out)
            }
            Node::Emphasis(e) => out.extend(self.inlines(
                &e.children,
                Style {
                    italic: true,
                    ..style
                },
            )),
            Node::Strong(s) => out.extend(self.inlines(
                &s.children,
                Style {
                    bold: true,
                    ..style
                },
            )),
            Node::Delete(d) => out.extend(self.inlines(
                &d.children,
                Style {
                    strike: true,
                    ..style
                },
            )),
            Node::InlineCode(code) => out.push(text(&code.value, style, true)),
            Node::Break(_) => out.push(Inline::Break),
            Node::InlineMath(math) => {
                let range = node
                    .position()
                    .map(|p| p.start.offset..p.end.offset)
                    .unwrap_or_default();
                let span = self.source.get(range.clone()).unwrap_or_default();
                let next = self
                    .source
                    .get(range.end..)
                    .and_then(|rest| rest.chars().next());
                if crate::is_inline_math(span, next) {
                    match self.draw(Kind::InlineMath, &math.value) {
                        Ok(svg) => out.push(Inline::Math { svg }),
                        // The page shows the formula as code.
                        Err(_) => out.push(text(&math.value, style, true)),
                    }
                } else {
                    // Dollar amounts, not math: the text as written.
                    push_text(span, style, out);
                }
            }
            Node::Link(link) => {
                let inlines = self.inlines(&link.children, style);
                self.link(&link.url, inlines, out);
            }
            Node::LinkReference(reference) => {
                let inlines = self.inlines(&reference.children, style);
                match self.definitions.get(&reference.identifier).cloned() {
                    Some(url) => self.link(&url, inlines, out),
                    None => out.extend(inlines),
                }
            }
            Node::Image(image) => {
                let (alt, width) = html::split_image_width(&image.alt);
                match html::attachment_id(&image.url) {
                    Some(id) => out.push(self.attached_image(&id, alt, width)),
                    None => {
                        // The page can't fetch it: its alt text, without a
                        // width where the page took one off.
                        let web =
                            image.url.starts_with("https://") || image.url.starts_with("http://");
                        let shown = if width.is_some() && web {
                            alt
                        } else {
                            &image.alt
                        };
                        out.push(Inline::Missing {
                            text: shown.to_owned(),
                        });
                    }
                }
            }
            Node::ImageReference(image) => out.push(Inline::Missing {
                text: image.alt.clone(),
            }),
            Node::Html(raw) if only_comments(&raw.value) => {}
            Node::Html(raw) => match self.html_image(&raw.value) {
                Some(image) => out.push(image),
                None => push_text(&raw.value, style, out),
            },
            Node::FootnoteReference(reference) => {
                let key = &reference.identifier;
                // Cited before (and so maybe listed already), else defined.
                let n = match self.cited.iter().position(|cited| cited == key) {
                    Some(n) => Some(n),
                    None if self.footnotes.contains_key(key) => {
                        self.cited.push(key.clone());
                        Some(self.cited.len() - 1)
                    }
                    None => None,
                };
                if let Some(n) = n {
                    out.push(text(&format!("[{}]", n + 1), style, false));
                }
            }
            _ => {
                if let Some(children) = node.children() {
                    out.extend(self.inlines(children, style));
                }
            }
        }
    }

    /// A link as the page keeps it: by its address as the page writes it, a
    /// `#…` pointing at a heading's `id`, a due date as its chip; a link the
    /// page drops is its words.
    fn link(&self, url: &str, inlines: Vec<Inline>, out: &mut Vec<Inline>) {
        // A link inside a link is its words, as in the page.
        let inlines: Vec<Inline> = inlines
            .into_iter()
            .flat_map(|inline| match inline {
                Inline::Link { inlines, .. } => inlines,
                other => vec![other],
            })
            .collect();
        let url = page_url(url);
        if let Some(fragment) = url.strip_prefix('#') {
            let wanted = app::percent_decode(fragment);
            match app::fragment_target(&wanted, &self.headings) {
                Some(id) => out.push(Inline::Link {
                    url: format!("#{id}"),
                    inlines,
                }),
                None => out.extend(inlines),
            }
            return;
        }
        // A moment in a recording stays a link: the app seeks its player. The
        // exported page, which has no player, still drops it.
        if !html::is_allowed_link(&url) && !is_audio_moment(&url) {
            out.extend(inlines);
            return;
        }
        if url.starts_with(DUE_LINK_PREFIX)
            && let Some(chip) = self.mode.due.and_then(|due| due(&url))
        {
            out.push(Inline::Due {
                url,
                label: chip.label,
                title: chip.title,
                tone: chip.tone,
            });
            return;
        }
        out.push(Inline::Link { url, inlines });
    }
}

/// The scheme of a link to a moment in a recording
/// (`kanso_engine::recording_note::AUDIO_SCHEME`).
pub const AUDIO_LINK_PREFIX: &str = "kanso-audio://file:";

/// Whether `url` is a link to a moment in a recording, as the app's player
/// follows. Pure.
pub fn is_audio_moment(url: &str) -> bool {
    url.to_ascii_lowercase()
        .strip_prefix(AUDIO_LINK_PREFIX)
        .is_some_and(|rest| !rest.is_empty())
}

/// Prose, with a wiki-link left over (`[[…]]`, naming no note) as its label.
fn push_text(value: &str, style: Style, out: &mut Vec<Inline>) {
    if !value.contains("[[") {
        out.push(text(value, style, false));
        return;
    }
    // Read the way the page reads its compiled HTML.
    let escaped = escape(value);
    let mut plain = String::new();
    let mut rest = escaped.as_str();
    while let Some(c) = rest.chars().next() {
        if rest.starts_with("[[")
            && let Some((label, _, len)) = html::wiki_link_at(rest)
        {
            if !plain.is_empty() {
                out.push(text(&app::unescape(&plain), style, false));
                plain.clear();
            }
            out.push(Inline::Missing {
                text: app::unescape(&label),
            });
            rest = &rest[len..];
            continue;
        }
        plain.push(c);
        rest = &rest[c.len_utf8()..];
    }
    if !plain.is_empty() {
        out.push(text(&app::unescape(&plain), style, false));
    }
}

/// `code` as runs in the styles `spans` give, covering all of it in order.
/// Pure.
pub fn code_runs(code: &str, spans: &[html::CodeSpan]) -> Vec<CodeRun> {
    html::code_pieces(code, spans)
        .into_iter()
        .filter(|(range, _)| !range.is_empty())
        .map(|(range, span)| CodeRun {
            text: code[range].to_owned(),
            color: span.and_then(|span| span.color.map(|color| color & 0xFF_FFFF)),
            bold: span.is_some_and(|span| span.bold),
            italic: span.is_some_and(|span| span.italic),
        })
        .collect()
}

#[cfg(test)]
mod tests;
