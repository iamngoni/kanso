//! A note as one self-contained HTML page, in a preview theme's colors:
//! math, diagrams and drawings inline as SVG, attached images as data URIs,
//! no scripts and nothing fetched (a Content Security Policy says so). It
//! opens anywhere, prints, and becomes a PDF.
//!
//! Raw HTML in a note is shown as text, except an attached image's `<img>`
//! tag; links into the app keep their text but go nowhere, and wiki-links
//! (`[[Title]]`, `[[Title#Heading]]`, `[[Title|alias]]`) show as their label.
//!
//! A page in a published notebook ([`Site`]) is the same page, except that
//! links to the notebook's other published notes stay links, to
//! `kanso-page:<slug>` ([`PAGE_LINK_PREFIX`]), which the share page turns
//! into the site's own addresses; [`site_index`] is the notebook's first page.

use std::fmt::Write as _;
use std::hash::{Hash as _, Hasher as _};

use base64::Engine as _;
use markdown::mdast::Node;

use crate::{Failure, Job, Kind, Outcome, Palette, Rgb, Svg, escape, render_blocking};

pub mod app;
pub use app::{
    AppMode, DueChip, DueTone, HeadingId, QueryResult, QueryRow, QueryTable, QueryViews, TaskBox,
    TaskSlot, heading_slug, headings, is_allowed_link, is_allowed_source, sanitize, task_boxes,
};

/// An attachment's content type and bytes, by ID (`file:…`).
pub type Attachments<'a> = &'a dyn Fn(&str) -> Option<(String, Vec<u8>)>;

/// How a page sets its text: the faces and size the preview uses, and the
/// highlighter that colors fenced code as it does. The defaults are the
/// preview's system faces at 16 points, and uncolored code.
#[derive(Clone, Copy)]
pub struct PageOptions<'a> {
    /// The body face, when it isn't the system's.
    pub font: Option<&'a str>,
    /// The monospaced face, when it isn't the system's.
    pub mono: Option<&'a str>,
    /// The body text size, in points.
    pub size: f32,
    /// Colors a fenced code block by its language; `None` leaves code as
    /// plain text.
    pub highlight: Option<&'a dyn Highlighter>,
    /// Makes the page one for the phone's app: its links keep the addresses
    /// the app understands and nothing else a page could do (see [`app`]).
    /// `None` is the page that is published and exported, unchanged.
    pub app: Option<&'a AppMode<'a>>,
}

impl Default for PageOptions<'_> {
    fn default() -> Self {
        PageOptions {
            font: None,
            mono: None,
            size: 16.,
            highlight: None,
            app: None,
        }
    }
}

/// A run of a code block in one style, as a byte range of the code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeSpan {
    pub start: usize,
    pub end: usize,
    /// `0xRRGGBB`.
    pub color: Option<u32>,
    pub bold: bool,
    pub italic: bool,
}

/// Colors fenced code the way the preview does: the grammar the fence names
/// and the syntax theme's colors.
pub trait Highlighter: Send + Sync {
    /// The styles for `code` in `lang` (the fence's info string), or `None`
    /// when Kanso has no grammar for it.
    fn highlight(&self, lang: &str, code: &str) -> Option<Vec<CodeSpan>>;
}

/// The parse options the preview uses, with math.
fn parse_options() -> markdown::ParseOptions {
    let mut options = markdown::ParseOptions::gfm();
    options.constructs.frontmatter = false;
    options.constructs.math_text = true;
    options.constructs.math_flow = true;
    options
}

/// What a piece of the note becomes.
enum Piece {
    /// Stands alone, in place of a paragraph, in a figure.
    Block(String),
    /// Stands alone, in place of a paragraph, without a figure.
    Bare(String),
    /// Sits in a line.
    Inline(String),
}

/// A link from one page of a published notebook to another: this, then the
/// page's slug (empty for the notebook's index).
pub const PAGE_LINK_PREFIX: &str = "kanso-page:";
/// The prefix of an ID link to a note inside a note body, then the note ID
/// without its `note:` prefix; the same as `kanso_core::links::NOTE_LINK_PREFIX`.
pub const NOTE_LINK_PREFIX: &str = "inkdrop://note/";

/// What a link to another note names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoteRef<'a> {
    /// A note by ID (`note:…`), from an ID link.
    Id(&'a str),
    /// A note by title as written, from a wiki-link.
    Title(&'a str),
}

/// A page among a published notebook's pages.
pub struct Site<'a> {
    /// The slug of the published page a link names; `None` for a note that
    /// isn't published, whose links show as plain text.
    pub resolve: &'a dyn Fn(NoteRef<'_>) -> Option<String>,
    /// The notebook's name, for the link back to its index.
    pub home: &'a str,
}

/// The note an ID link ([`NOTE_LINK_PREFIX`] then the ID) points at, as `note:<id>`.
/// Pure.
pub fn linked_note_id(url: &str) -> Option<String> {
    let id = url.strip_prefix(NOTE_LINK_PREFIX)?;
    let id = id.split(['?', '#', '/', ')', ' ']).next()?;
    (!id.is_empty()).then(|| format!("note:{id}"))
}

/// The prefix of an attachment link inside a note body, then the file ID.
/// It is the scheme the app that first wrote these notes uses, and the same
/// as `kanso_core::links::FILE_LINK_PREFIX` (this crate doesn't depend on
/// `kanso-core`); it stays exactly as written.
pub const FILE_LINK_PREFIX: &str = "inkdrop://file:";

/// The attachment ID an attachment link ([`FILE_LINK_PREFIX`]) points at.
/// Pure.
pub fn attachment_id(url: &str) -> Option<String> {
    let id = url.strip_prefix(FILE_LINK_PREFIX)?;
    let id = id.split(['?', '#', '/', ')', ' ']).next()?;
    (!id.is_empty()).then(|| format!("file:{id}"))
}

/// An `<img>` tag's `src`, `alt` and `width`. Pure.
pub fn img_tag(html: &str) -> Option<(String, String, Option<u32>)> {
    let html = html.trim();
    let rest = html
        .strip_prefix("<img")
        .or_else(|| html.strip_prefix("<IMG"))?;
    if !rest.starts_with(|c: char| c.is_whitespace()) {
        return None;
    }
    let rest = rest.trim_end_matches('>').trim_end_matches('/');
    let (mut src, mut alt, mut width) = (None, String::new(), None);
    let mut chars = rest;
    loop {
        chars = chars.trim_start();
        if chars.is_empty() {
            break;
        }
        let name_end = chars
            .find(|c: char| c == '=' || c.is_whitespace())
            .unwrap_or(chars.len());
        let name = chars[..name_end].to_ascii_lowercase();
        chars = chars[name_end..].trim_start();
        let Some(after) = chars.strip_prefix('=') else {
            continue;
        };
        let after = after.trim_start();
        let (value, tail) = match after.chars().next() {
            Some(q @ ('"' | '\'')) => {
                let end = after[1..].find(q)? + 1;
                (&after[1..end], &after[end + 1..])
            }
            _ => {
                let end = after.find(char::is_whitespace).unwrap_or(after.len());
                (&after[..end], &after[end..])
            }
        };
        match name.as_str() {
            "src" => src = Some(value.to_owned()),
            "alt" => alt = value.to_owned(),
            "width" => width = value.trim_end_matches("px").parse().ok(),
            _ => {}
        }
        chars = tail;
    }
    Some((src?, alt, width))
}

/// An image's alt text without an Obsidian-style width after a `|`
/// (`Cat|300`, or `Cat|300x200`, whose height is left to the image), and
/// that width. Pure.
pub fn split_image_width(alt: &str) -> (&str, Option<u32>) {
    let Some((text, size)) = alt.rsplit_once('|') else {
        return (alt, None);
    };
    let size = size.trim();
    let width = size.split_once(['x', 'X']).map_or(size, |(w, _)| w);
    if width.is_empty() || width.len() > 5 || !width.bytes().all(|b| b.is_ascii_digit()) {
        return (alt, None);
    }
    match width.parse::<u32>() {
        Ok(width) if width > 0 => (text.trim_end(), Some(width)),
        _ => (alt, None),
    }
}

/// The label a wiki-link's inside (`Title`, `Title#Heading` or
/// `Title|alias`, as compiled HTML) shows: the alias, else the title and
/// heading. `None` when it has no words. Pure.
fn wiki_label(inner: &str) -> Option<String> {
    let label = match inner.split_once('|') {
        Some((_, alias)) if !alias.trim().is_empty() => alias.trim().to_owned(),
        Some((target, _)) => heading_label(target),
        None => heading_label(inner),
    };
    let words = label.replace(['<', '>'], "");
    (!words.trim().is_empty()).then_some(label)
}

/// `Title#Heading` as "Title › Heading", or the heading alone.
fn heading_label(target: &str) -> String {
    match target.split_once('#') {
        Some((title, heading)) if title.trim().is_empty() => heading.trim().to_owned(),
        Some((title, heading)) if heading.trim().is_empty() => title.trim().to_owned(),
        Some((title, heading)) => format!("{} › {}", title.trim(), heading.trim()),
        None => target.trim().to_owned(),
    }
}

/// Inline tags a wiki-link's text may hold, as Markdown compiles it.
const WIKI_INLINE_TAGS: [&str; 6] = ["<em>", "</em>", "<strong>", "</strong>", "<del>", "</del>"];

/// The title a wiki-link's inside names, as plain text: without its
/// heading, alias, inline tags and character references. Pure.
fn wiki_title(inner: &str) -> String {
    let target = inner.split_once('|').map_or(inner, |(target, _)| target);
    let title = target.split_once('#').map_or(target, |(title, _)| title);
    let mut text = String::with_capacity(title.len());
    let mut rest = title;
    while let Some(at) = rest.find('<') {
        text.push_str(&rest[..at]);
        rest = rest[at..].find('>').map_or("", |end| &rest[at + end + 1..]);
    }
    text.push_str(rest);
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
        .trim()
        .to_owned()
}

/// The wiki-link at the start of `html` (which starts `[[`): its label,
/// the title it names and how long it is. Pure.
fn wiki_link_at(html: &str) -> Option<(String, String, usize)> {
    let close = html.get(2..)?.find("]]")? + 2;
    let inner = &html[2..close];
    if inner.contains('\n') || inner.contains("[[") {
        return None;
    }
    // Only inline emphasis inside: no code, links or block tags.
    let mut rest = inner;
    while let Some(at) = rest.find('<') {
        let tag_end = rest[at..].find('>')? + at + 1;
        if !WIKI_INLINE_TAGS.contains(&&rest[at..tag_end]) {
            return None;
        }
        rest = &rest[tag_end..];
    }
    Some((wiki_label(inner)?, wiki_title(inner), close + 2))
}

/// `[[Title]]`, `[[Title#Heading]]` and `[[Title|alias]]` in compiled
/// HTML, outside code, as their label: a standalone page has no note to
/// link to. Pure.
pub fn wiki_links(html: &str) -> String {
    wiki_links_in(html, None)
}

/// [`wiki_links`], with links to a site's published notes kept as links
/// to their pages. Pure.
pub fn wiki_links_in(html: &str, site: Option<&Site>) -> String {
    if !html.contains("[[") {
        return html.to_owned();
    }
    let mut out = String::with_capacity(html.len());
    let mut in_code = 0usize;
    let mut rest = html;
    while let Some(c) = rest.chars().next() {
        if c == '<' {
            let end = rest.find('>').map_or(rest.len(), |e| e + 1);
            let tag = rest[..end].to_ascii_lowercase();
            if tag.starts_with("<code") || tag.starts_with("<pre") {
                in_code += 1;
            } else if tag.starts_with("</code") || tag.starts_with("</pre") {
                in_code = in_code.saturating_sub(1);
            }
            out.push_str(&rest[..end]);
            rest = &rest[end..];
            continue;
        }
        if in_code == 0
            && rest.starts_with("[[")
            && let Some((label, title, len)) = wiki_link_at(rest)
        {
            match site.and_then(|site| (site.resolve)(NoteRef::Title(&title))) {
                Some(slug) => {
                    let _ = write!(
                        out,
                        r#"<a class="kanso-wikilink" href="{PAGE_LINK_PREFIX}{}">{label}</a>"#,
                        escape(&slug)
                    );
                }
                None => {
                    let _ = write!(out, r#"<span class="kanso-wikilink">{label}</span>"#);
                }
            }
            rest = &rest[len..];
            continue;
        }
        out.push(c);
        rest = &rest[c.len_utf8()..];
    }
    out
}

/// Whether a fence's info string names `want`.
fn fence_is(lang: &Option<String>, want: &str) -> bool {
    lang.as_deref()
        .is_some_and(|l| l.trim().eq_ignore_ascii_case(want))
}

/// The attached Excalidraw scene a paragraph is, if it is only a link (or
/// image) to an attachment named `….excalidraw`. Pure.
pub fn attached_scene(node: &Node) -> Option<String> {
    let Node::Paragraph(paragraph) = node else {
        return None;
    };
    let (url, label) = match paragraph.children.as_slice() {
        [Node::Link(link)] => (link.url.as_str(), node_text(&link.children)),
        [Node::Image(image)] => (image.url.as_str(), image.alt.clone()),
        _ => return None,
    };
    let label = label.trim().to_ascii_lowercase();
    (label.ends_with(".excalidraw") || label.ends_with(".excalidraw.json"))
        .then(|| attachment_id(url))
        .flatten()
}

/// The formula a paragraph is, if it is only one `$$…$$`: display math as
/// KaTeX writes it on one line. `markdown` is the paragraph's source. Pure.
pub fn display_math_paragraph<'a>(node: &'a Node, markdown: &str) -> Option<&'a str> {
    let Node::Paragraph(paragraph) = node else {
        return None;
    };
    let [Node::InlineMath(math)] = paragraph.children.as_slice() else {
        return None;
    };
    let markdown = markdown.trim();
    (markdown.len() > 4 && markdown.starts_with("$$") && markdown.ends_with("$$"))
        .then_some(math.value.as_str())
}

fn node_text(nodes: &[Node]) -> String {
    nodes.iter().map(Node::to_string).collect()
}

fn svg_markup(svg: &Svg, class: &str, scale: f32) -> String {
    let style = match svg.baseline {
        Some(baseline) if class == "kanso-math" => format!(
            " style=\"vertical-align:{}px\"",
            crate::svg::number(-(svg.height - baseline) * scale)
        ),
        _ => String::new(),
    };
    format!(r#"<span class="{class}"{style}>{}</span>"#, svg.markup)
}

/// `data:<type>;base64,<bytes>`, as pages carry their images. Pure.
pub fn data_uri(content_type: &str, bytes: &[u8]) -> String {
    format!(
        "data:{};base64,{}",
        content_type,
        base64::engine::general_purpose::STANDARD.encode(bytes)
    )
}

/// The image an attachment is, as an `<img>`; its alt text if it is gone.
fn attached_image(id: &str, alt: &str, width: Option<u32>, attachments: Attachments<'_>) -> String {
    let safe_type = |t: &str| t.starts_with("image/") && !t.contains(['"', '<', '>']);
    match attachments(id) {
        Some((content_type, bytes)) if safe_type(&content_type) => format!(
            r#"<img src="{}" alt="{}"{}>"#,
            data_uri(&content_type, &bytes),
            escape(alt),
            width
                .map(|w| format!(r#" width="{w}""#))
                .unwrap_or_default()
        ),
        _ => format!(r#"<span class="kanso-missing">{}</span>"#, escape(alt)),
    }
}

/// Finds what the note's Markdown can't say in HTML itself, depth first.
/// `codes` are the start offsets of the note's code blocks, in order; only
/// the page in the app uses them, to number its copy links.
#[allow(clippy::too_many_arguments)]
fn collect(
    node: &Node,
    source: &str,
    palette: &Palette,
    attachments: Attachments<'_>,
    site: Option<&Site>,
    options: &PageOptions<'_>,
    codes: &[usize],
    out: &mut Vec<(std::ops::Range<usize>, Piece)>,
) {
    let range = node.position().map(|p| p.start.offset..p.end.offset);
    // The node as written.
    let written = range
        .clone()
        .and_then(|r| source.get(r))
        .unwrap_or_default();
    let in_app = options.app.is_some();
    let draw = |kind: Kind, code: &str| -> Outcome {
        render_blocking(&Job::new(kind, code, *palette))
            .as_ref()
            .clone()
    };
    let drawn = |kind: Kind, code: &str| -> Option<Svg> { draw(kind, code).ok() };
    // A drawing as a block; in the app, one that failed shows its code with
    // a badge saying why, as the desktop's preview does.
    let block = |outcome: Outcome, class: &str, fallback: &dyn Fn(&Failure) -> String| match outcome
    {
        Ok(svg) => Some(Piece::Block(svg_markup(&svg, class, 1.))),
        Err(failure) if in_app => Some(Piece::Bare(fallback(&failure))),
        Err(_) => None,
    };
    let copy_index = |node: &Node| {
        let start = node.position()?.start.offset;
        codes.iter().position(|&at| at == start)
    };
    let piece = match node {
        Node::Math(math) => block(
            draw(Kind::DisplayMath, &math.value),
            "kanso-math-display",
            &|failure| app::failed_html(failure, None, written, None),
        ),
        Node::InlineMath(math) => {
            let range = range.clone().unwrap_or_default();
            let span = source.get(range.clone()).unwrap_or_default();
            let next = source.get(range.end..).and_then(|rest| rest.chars().next());
            if crate::is_inline_math(span, next) {
                drawn(Kind::InlineMath, &math.value)
                    .map(|svg| Piece::Inline(svg_markup(&svg, "kanso-math", 1.)))
            } else {
                // Dollar amounts, not math: the text as written.
                Some(Piece::Inline(escape(span)))
            }
        }
        Node::Code(code) if fence_is(&code.lang, "math") => block(
            draw(Kind::DisplayMath, &code.value),
            "kanso-math-display",
            &|failure| {
                app::failed_html(failure, code.lang.as_deref(), &code.value, copy_index(node))
            },
        ),
        Node::Code(code) if fence_is(&code.lang, "mermaid") => block(
            draw(Kind::Mermaid, &code.value),
            "kanso-diagram",
            &|failure| {
                app::failed_html(failure, code.lang.as_deref(), &code.value, copy_index(node))
            },
        ),
        Node::Code(code)
            if in_app && fence_is(&code.lang, "excalidraw") && app::is_empty_scene(&code.value) =>
        {
            Some(Piece::Block(app::EMPTY_DRAWING.to_owned()))
        }
        Node::Code(code) if fence_is(&code.lang, "excalidraw") => block(
            draw(Kind::Excalidraw, &code.value),
            "kanso-drawing",
            &|failure| {
                app::failed_html(failure, code.lang.as_deref(), &code.value, copy_index(node))
            },
        ),
        Node::Code(code) if code.lang.as_deref().is_some_and(crate::is_plugin_language) => {
            let language = code.lang.as_deref().unwrap_or_default().trim();
            block(
                render_blocking(&Job::plugin(language, code.value.as_str(), *palette))
                    .as_ref()
                    .clone(),
                "kanso-drawing",
                &|failure| {
                    app::failed_html(failure, code.lang.as_deref(), &code.value, copy_index(node))
                },
            )
        }
        Node::Code(code)
            if options.app.is_some_and(|mode| mode.query.is_some())
                && app::is_query_fence(code.lang.as_deref()) =>
        {
            options
                .app
                .and_then(|mode| mode.query)
                .map(|query| Piece::Bare(app::query_html(&code.value, &query.run(&code.value))))
        }
        // In the app every code block is drawn here, with its copy link.
        Node::Code(code) if in_app => {
            let spans = options
                .highlight
                .and_then(|highlight| {
                    highlight.highlight(code.lang.as_deref().unwrap_or_default(), &code.value)
                })
                .unwrap_or_default();
            Some(Piece::Bare(code_html(
                code.lang.as_deref(),
                &code.value,
                &spans,
                copy_index(node),
            )))
        }
        Node::Code(code) => options.highlight.and_then(|highlight| {
            let lang = code.lang.as_deref().unwrap_or_default();
            highlight.highlight(lang, &code.value).map(|spans| {
                Piece::Bare(code_html(code.lang.as_deref(), &code.value, &spans, None))
            })
        }),
        Node::Paragraph(_) if display_math_paragraph(node, written).is_some() => {
            display_math_paragraph(node, written).and_then(|formula| {
                block(
                    draw(Kind::DisplayMath, formula),
                    "kanso-math-display",
                    &|failure| app::failed_html(failure, None, written, None),
                )
            })
        }
        Node::Paragraph(_) if attached_scene(node).is_some() => {
            let scene = attached_scene(node)
                .and_then(|id| attachments(&id))
                .and_then(|(_, bytes)| String::from_utf8(bytes).ok());
            match scene {
                Some(scene) if in_app && app::is_empty_scene(&scene) => {
                    Some(Piece::Block(app::EMPTY_DRAWING.to_owned()))
                }
                Some(scene) => block(
                    draw(Kind::Excalidraw, &scene),
                    "kanso-drawing",
                    &|failure| app::failed_html(failure, None, written, None),
                ),
                None if in_app => Some(Piece::Bare(app::failed_html(
                    &Failure::Invalid("The drawing's file is missing.".into()),
                    None,
                    written,
                    None,
                ))),
                None => None,
            }
        }
        Node::Image(image) => {
            let (alt, width) = split_image_width(&image.alt);
            match attachment_id(&image.url) {
                Some(id) => Some(Piece::Inline(attached_image(&id, alt, width, attachments))),
                // A web image given a width: the same tag Markdown would
                // write, with the width and without the `|300`.
                None => width
                    .filter(|_| {
                        image.url.starts_with("https://") || image.url.starts_with("http://")
                    })
                    .map(|width| {
                        Piece::Inline(format!(
                            r#"<img src="{}" alt="{}" width="{width}">"#,
                            escape(&image.url),
                            escape(alt)
                        ))
                    }),
            }
        }
        // In a site, an ID link to a published note links to its page.
        Node::Link(link) if site.is_some() => linked_note_id(&link.url)
            .and_then(|id| site.and_then(|site| (site.resolve)(NoteRef::Id(&id))))
            .map(|slug| {
                Piece::Inline(format!(
                    r#"<a href="{PAGE_LINK_PREFIX}{}">{}</a>"#,
                    escape(&slug),
                    escape(&node_text(&link.children))
                ))
            }),
        Node::Html(html) => img_tag(&html.value).and_then(|(src, alt, width)| {
            attachment_id(&src)
                .map(|id| Piece::Inline(attached_image(&id, &alt, width, attachments)))
        }),
        _ => None,
    };
    match (piece, range) {
        (Some(piece), Some(range)) if source.get(range.clone()).is_some() => {
            out.push((range, piece))
        }
        _ => {
            for child in node.children().into_iter().flatten() {
                collect(
                    child,
                    source,
                    palette,
                    attachments,
                    site,
                    options,
                    codes,
                    out,
                );
            }
        }
    }
}

/// `markdown` as plain paragraphs, for text markdown-rs can't parse: its
/// lines as written, a blank line between paragraphs.
fn plain_html(markdown: &str) -> String {
    markdown
        .split("\n\n")
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(|p| format!("<p>{}</p>\n", escape(p).replace('\n', "<br>\n")))
        .collect()
}

/// The note's body as HTML, without the page around it.
pub fn body(markdown: &str, palette: &Palette, attachments: Attachments<'_>) -> String {
    body_with(
        markdown,
        palette,
        attachments,
        None,
        &PageOptions::default(),
    )
}

/// [`body`], as a page of `site` when given.
pub fn body_in(
    markdown: &str,
    palette: &Palette,
    attachments: Attachments<'_>,
    site: Option<&Site>,
) -> String {
    body_with(
        markdown,
        palette,
        attachments,
        site,
        &PageOptions::default(),
    )
}

/// [`body_in`], set the way `options` say.
pub fn body_with(
    markdown: &str,
    palette: &Palette,
    attachments: Attachments<'_>,
    site: Option<&Site>,
    options: &PageOptions<'_>,
) -> String {
    let parse = parse_options();
    let Ok(root) = crate::markdown_safe::to_mdast(markdown, &parse) else {
        return plain_html(markdown);
    };
    let facts = options.app.map(|_| app::facts(&root));
    let codes: Vec<usize> = facts
        .as_ref()
        .map(|facts| facts.code_offsets())
        .unwrap_or_default();
    let mut pieces = Vec::new();
    collect(
        &root,
        markdown,
        palette,
        attachments,
        site,
        options,
        &codes,
        &mut pieces,
    );
    pieces.sort_by_key(|(range, _)| range.start);
    // A placeholder no note will contain: it survives Markdown as a word.
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    markdown.hash(&mut hasher);
    let nonce = format!("{:x}", hasher.finish());
    let token = |n: usize| format!("kansopiece{nonce}n{n}x");
    let mut text = String::with_capacity(markdown.len());
    let mut at = 0;
    let mut kept = Vec::new();
    for (range, piece) in pieces {
        if range.start < at {
            continue;
        }
        text.push_str(&markdown[at..range.start]);
        text.push_str(&token(kept.len()));
        at = range.end;
        kept.push(piece);
    }
    text.push_str(&markdown[at..]);
    let compile = markdown::CompileOptions {
        gfm_tagfilter: true,
        // In the app every address is let through to be checked once the page
        // is whole (`app::sanitize`), not dropped here, so the app's own
        // addresses survive.
        allow_dangerous_protocol: options.app.is_some(),
        ..markdown::CompileOptions::default()
    };
    let Ok(mut html) =
        crate::markdown_safe::to_html_with_options(&text, &markdown::Options { parse, compile })
    else {
        return plain_html(markdown);
    };
    // Links into the app (and other schemes a browser can't follow) lose
    // their address, so they aren't links that go nowhere.
    html = html.replace("<a href=\"\">", "<a>");
    html = wiki_links_in(&html, site);
    for (n, piece) in kept.into_iter().enumerate() {
        let token = token(n);
        html = match piece {
            Piece::Block(markup) => {
                let paragraph = format!("<p>{token}</p>");
                if html.contains(&paragraph) {
                    html.replacen(&paragraph, &format!("<figure>{markup}</figure>"), 1)
                } else {
                    html.replacen(&token, &markup, 1)
                }
            }
            Piece::Bare(markup) => {
                let paragraph = format!("<p>{token}</p>");
                if html.contains(&paragraph) {
                    html.replacen(&paragraph, &markup, 1)
                } else {
                    html.replacen(&token, &markup, 1)
                }
            }
            Piece::Inline(markup) => html.replacen(&token, &markup, 1),
        };
    }
    match options.app {
        Some(mode) => app::finish(html, facts.as_ref(), mode, markdown),
        None => html,
    }
}

/// Escapes a code block as spans colored like the preview's highlighting.
fn code_html(lang: Option<&str>, code: &str, spans: &[CodeSpan], copy: Option<usize>) -> String {
    let mut out = String::with_capacity(code.len() + 96);
    out.push_str("<pre class=\"kanso-code\">");
    if let Some(lang) = lang.map(str::trim).filter(|lang| !lang.is_empty()) {
        let _ = write!(out, "<span class=\"kanso-lang\">{}</span>", escape(lang));
    }
    if let Some(n) = copy {
        out.push_str(&app::copy_link(n));
    }
    out.push_str("<code>");
    let mut at = 0;
    for span in spans {
        let start = boundary(code, span.start.max(at).min(code.len()));
        let end = boundary(code, span.end.min(code.len()).max(start));
        if end <= at {
            continue;
        }
        if start > at {
            out.push_str(&escape(&code[at..start]));
        }
        let mut style = String::new();
        if let Some(color) = span.color {
            let _ = write!(style, "color:#{:06x};", color & 0xFF_FFFF);
        }
        if span.italic {
            style.push_str("font-style:italic;");
        }
        if span.bold {
            style.push_str("font-weight:bold;");
        }
        let text = escape(&code[start..end]);
        if style.is_empty() {
            out.push_str(&text);
        } else {
            let _ = write!(out, "<span style=\"{style}\">{text}</span>");
        }
        at = end;
    }
    out.push_str(&escape(&code[at..]));
    out.push_str("</code></pre>");
    out
}

/// `at`, moved left to the nearest character boundary.
fn boundary(text: &str, mut at: usize) -> usize {
    while at > 0 && !text.is_char_boundary(at) {
        at -= 1;
    }
    at
}

/// The stack a page falls back to when it names no face.
const SANS_FALLBACK: &str =
    "-apple-system,BlinkMacSystemFont,\"Segoe UI\",Inter,\"Helvetica Neue\",Arial,sans-serif";
const MONO_FALLBACK: &str =
    "ui-monospace,\"SF Mono\",Menlo,Consolas,\"DejaVu Sans Mono\",monospace";

/// A family name a stylesheet can carry, followed by `fallback`. Names
/// with quotes, semicolons, braces or other syntax are left out, and so are
/// Core Text's private `.Name` families, which a browser doesn't know.
fn family(name: Option<&str>, fallback: &str) -> String {
    let safe = |c: char| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.' | '+');
    match name.map(str::trim) {
        Some(name) if !name.is_empty() && !name.starts_with('.') && name.chars().all(safe) => {
            format!("\"{name}\",{fallback}")
        }
        _ => fallback.to_owned(),
    }
}

/// A size a page can set: the preview's when sane, a readable default
/// otherwise.
fn text_size(size: f32) -> f32 {
    if size.is_finite() && (8.0..=72.0).contains(&size) {
        size
    } else {
        16.
    }
}

/// A page's stylesheet: the preview's typography and colors.
pub fn stylesheet(p: &Palette, options: &PageOptions<'_>) -> String {
    let c = |rgb: Rgb| rgb.hex();
    let size = text_size(options.size);
    let width = crate::typography::readable_width(size, false);
    let quiet = c(p.border.over(p.surface, 0.6));
    let link_under = c(p.accent.over(p.background, 0.4));
    let quote_bar = c(p.quote.over(p.background, 0.45));
    let code_text = p.code_text.map_or_else(|| c(p.text), c);
    let inline_text = p
        .inline_code_text
        .map(|color| format!("color:{};", c(color)))
        .unwrap_or_default();
    let table_head = p.table_header_background.map_or_else(|| c(p.surface), c);
    let heading = p
        .heading
        .map(|color| format!("h1,h2,h3,h4,h5,h6{{color:{}}}", c(color)))
        .unwrap_or_default();
    let h6_color = c(p.task_done);
    let check_stroke = if p.dark { "black" } else { "white" };
    let check = format!(
        "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16'%3E%3Cpath d='m3.25 8.25 3 3 6.5-7' fill='none' stroke='{check_stroke}' stroke-width='1.8' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E\")"
    );
    let h1 = crate::typography::heading(1);
    let h2 = crate::typography::heading(2);
    let h3 = crate::typography::heading(3);
    let h4 = crate::typography::heading(4);
    let h5 = crate::typography::heading(5);
    let h6 = crate::typography::heading(6);
    let bottom = crate::typography::HEADING_BOTTOM_GAP;
    let gap = crate::typography::PARAGRAPH_GAP;
    format!(
        r#"
:root{{color-scheme:{scheme}}}
*{{box-sizing:border-box}}
html{{background:{bg};-webkit-print-color-adjust:exact;print-color-adjust:exact}}
body{{margin:0 auto;max-width:calc({width}px + 4rem);padding:3rem 2rem 4rem;background:{bg};color:{text};font:{size}px/{line} {sans};-webkit-font-smoothing:antialiased}}
header h1{{font-size:2rem;line-height:1.25;margin:0 0 1.5rem;font-weight:700}}
h1,h2,h3,h4,h5,h6{{font-weight:600;line-height:1.35}}
h1{{font-size:{h1s}em;line-height:1.25;margin:{h1g}em 0 {bottom}em;font-weight:700}}
h2{{font-size:{h2s}em;margin:{h2g}em 0 {bottom}em}}
h3{{font-size:{h3s}em;margin:{h3g}em 0 {bottom}em}}
h4{{font-size:{h4s}em;margin:{h4g}em 0 {bottom}em}}
h5{{font-size:{h5s}em;margin:{h5g}em 0 {bottom}em}}
h6{{font-size:{h6s}em;margin:{h6g}em 0 {bottom}em;color:{h6color}}}
{heading}
:is(h1,h2,h3,h4,h5,h6)+:is(h1,h2,h3,h4,h5,h6){{margin-top:.25em}}
p,ul,ol,pre,table,blockquote,figure{{margin:0 0 {gap}em}}
a{{color:{accent};text-decoration:underline;text-underline-offset:2px;text-decoration-color:{linkunder}}}
a:not([href]){{color:inherit;text-decoration-style:dotted}}
code,pre{{font-family:{mono}}}
code{{font-size:1em;background:{inlinebg};border-radius:4px;padding:.1em .3em;{inlinetext}}}
pre{{position:relative;background:{surface};border:1px solid {quiet};border-radius:{radius}px;padding:{codetop}px {codepadx}px {codepadb}px;overflow-x:auto;line-height:{codeline};font-size:{codesize}px;color:{codetext}}}
pre code{{font-size:inherit;background:none;padding:0;color:inherit}}
.kanso-lang{{position:absolute;top:7px;right:12px;font-family:{sans};font-size:12px;line-height:16px;color:{muted}}}
blockquote{{border-left:3px solid {quotebar};margin-left:0;padding-left:14px;color:{quote}}}
table{{border-collapse:collapse;display:block;overflow-x:auto;font-size:.9333em}}
th,td{{border:1px solid {border};padding:7px 12px;text-align:left}}
thead th{{background:{tablehead};font-weight:600}}
hr{{border:0;border-top:1px solid {border};margin:1.6em 0}}
img{{max-width:100%;height:auto}}
ul,ol{{padding-left:1.4em}}
li+li{{margin-top:.25em}}
li::marker{{color:{taskdone}}}
ul:has(>li>input[type=checkbox]){{padding-left:0;list-style:none}}
li:has(>input[type=checkbox]){{list-style:none}}
li:has(>input[type=checkbox])>input[type=checkbox]{{appearance:none;-webkit-appearance:none;width:15px;height:15px;margin:0 8px 0 0;vertical-align:-2px;border:1.5px solid {taskdone};border-radius:4px;background:transparent}}
li:has(>input[type=checkbox])>input[type=checkbox]:checked{{background:{link} {check} no-repeat center/12px 12px;border-color:{link}}}
li:has(>input[type=checkbox]:checked){{color:{taskdone}}}
figure{{margin-left:0;margin-right:0;overflow-x:auto}}
figure svg{{max-width:100%;height:auto}}
.kanso-math svg{{display:inline-block}}
.kanso-math-display{{display:block;text-align:center;margin:.4em 0}}
.kanso-diagram,.kanso-drawing{{display:block}}
.kanso-missing{{color:{muted};font-style:italic}}
.kanso-wikilink{{border-bottom:1px dotted {muted}}}
a.kanso-wikilink{{border-bottom:0}}
.kanso-site-nav{{margin:0 0 1.2rem;font-size:.9rem}}
.kanso-site-nav a{{color:{muted};text-decoration:none}}
ul.kanso-site-index{{list-style:none;padding:0}}
.kanso-site-index li{{margin:0 0 1.1em}}
.kanso-site-index a{{font-weight:600;text-decoration:none}}
.kanso-site-index p{{margin:.2em 0 0;color:{muted}}}
{app_css}@media print{{html,body{{background:none}}body{{max-width:none;padding:0}}pre,figure,table,img{{break-inside:avoid}}h1,h2,h3{{break-after:avoid}}}}
"#,
        scheme = if p.dark { "dark" } else { "light" },
        bg = c(p.background),
        text = c(p.text),
        muted = c(p.muted),
        accent = c(p.accent),
        border = c(p.border),
        surface = c(p.surface),
        quiet = quiet,
        width = width,
        size = size,
        line = crate::typography::LINE_HEIGHT,
        sans = family(options.font, SANS_FALLBACK),
        mono = family(options.mono, MONO_FALLBACK),
        heading = heading,
        h6color = h6_color,
        h1s = h1.scale,
        h1g = h1.gap,
        h2s = h2.scale,
        h2g = h2.gap,
        h3s = h3.scale,
        h3g = h3.gap,
        h4s = h4.scale,
        h4g = h4.gap,
        h5s = h5.scale,
        h5g = h5.gap,
        h6s = h6.scale,
        h6g = h6.gap,
        bottom = bottom,
        gap = gap,
        radius = crate::typography::CODE_RADIUS,
        codetop = crate::typography::CODE_TOP,
        codepadx = crate::typography::CODE_PADDING_X,
        codepadb = crate::typography::CODE_PADDING_BOTTOM,
        codeline = crate::typography::CODE_LINE_HEIGHT,
        codesize = crate::typography::CODE_SIZE,
        linkunder = link_under,
        quotebar = quote_bar,
        quote = c(p.quote),
        taskdone = c(p.task_done),
        inlinebg = c(p.inline_code_background),
        inlinetext = inline_text,
        tablehead = table_head,
        codetext = code_text,
        link = c(p.accent),
        app_css = if options.app.is_some() {
            app::stylesheet(p, &check)
        } else {
            String::new()
        },
        check = check,
    )
}

/// A whole page: the note's title and body, styled, standalone.
pub fn page(
    title: &str,
    markdown: &str,
    palette: &Palette,
    attachments: Attachments<'_>,
) -> String {
    page_with(
        title,
        markdown,
        palette,
        attachments,
        &PageOptions::default(),
    )
}

/// [`page`], set the way `options` say.
pub fn page_with(
    title: &str,
    markdown: &str,
    palette: &Palette,
    attachments: Attachments<'_>,
    options: &PageOptions<'_>,
) -> String {
    page_in_with(title, markdown, palette, attachments, None, options)
}

/// [`page`], as a page of `site` when given: links to the site's other
/// pages stay links, and a link above the title leads back to its index.
pub fn page_in(
    title: &str,
    markdown: &str,
    palette: &Palette,
    attachments: Attachments<'_>,
    site: Option<&Site>,
) -> String {
    page_in_with(
        title,
        markdown,
        palette,
        attachments,
        site,
        &PageOptions::default(),
    )
}

/// [`page_in`], set the way `options` say.
pub fn page_in_with(
    title: &str,
    markdown: &str,
    palette: &Palette,
    attachments: Attachments<'_>,
    site: Option<&Site>,
    options: &PageOptions<'_>,
) -> String {
    let body = body_with(markdown, palette, attachments, site, options);
    let title = title.trim();
    let shown_title = if title.is_empty() { "Untitled" } else { title };
    let nav = site.map(|site| {
        format!(
            r#"<nav class="kanso-site-nav"><a href="{PAGE_LINK_PREFIX}">← {}</a></nav>"#,
            escape(site.home.trim())
        )
    });
    document(shown_title, nav.as_deref(), &body, palette, options)
}

/// A note on a published notebook's index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexEntry {
    pub slug: String,
    pub title: String,
    /// A line or two of the note's text.
    pub snippet: String,
    /// The notebook inside the published one it is in ("Plans › 2026"),
    /// or `None` for the published notebook's own notes.
    pub group: Option<String>,
}

/// A published notebook's first page: its name, then its notes, those of
/// the notebooks inside it under their names, each linking to its page.
pub fn site_index(name: &str, entries: &[IndexEntry], palette: &Palette) -> String {
    site_index_with(name, entries, palette, &PageOptions::default())
}

/// [`site_index`], set the way `options` say.
pub fn site_index_with(
    name: &str,
    entries: &[IndexEntry],
    palette: &Palette,
    options: &PageOptions<'_>,
) -> String {
    let name = name.trim();
    let shown = if name.is_empty() { "Notebook" } else { name };
    let mut main = String::new();
    let mut group: Option<Option<&str>> = None;
    for entry in entries {
        let this = entry.group.as_deref();
        if group != Some(this) {
            if group.is_some() {
                main.push_str("</ul>\n");
            }
            if let Some(name) = this {
                let _ = writeln!(main, "<h2>{}</h2>", escape(name));
            }
            main.push_str("<ul class=\"kanso-site-index\">\n");
            group = Some(this);
        }
        let title = entry.title.trim();
        let _ = write!(
            main,
            r#"<li><a href="{PAGE_LINK_PREFIX}{}">{}</a>"#,
            escape(&entry.slug),
            escape(if title.is_empty() { "Untitled" } else { title })
        );
        if !entry.snippet.trim().is_empty() {
            let _ = write!(main, "<p>{}</p>", escape(entry.snippet.trim()));
        }
        main.push_str("</li>\n");
    }
    if group.is_some() {
        main.push_str("</ul>\n");
    } else {
        main.push_str("<p class=\"kanso-missing\">Nothing is published here yet.</p>\n");
    }
    document(shown, None, &main, palette, options)
}

/// The page around a title and its HTML: styled, standalone.
fn document(
    title: &str,
    nav: Option<&str>,
    main: &str,
    palette: &Palette,
    options: &PageOptions<'_>,
) -> String {
    let mut html = String::with_capacity(main.len() + 4096);
    let _ = write!(
        html,
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src data:; style-src 'unsafe-inline'">
<meta name="generator" content="Kanso">
<title>{title}</title>
<style>{css}</style>
</head>
<body>
<header>{nav}<h1>{title}</h1></header>
<main>
{main}</main>
</body>
</html>
"#,
        title = escape(title),
        nav = nav.unwrap_or_default(),
        css = stylesheet(palette, options),
    );
    html
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn none(_: &str) -> Option<(String, Vec<u8>)> {
        None
    }

    #[test]
    fn a_note_becomes_a_standalone_page() {
        let note = "Intro with $x^2$ inline.\n\n$$\n\\int_0^1 x\\,dx\n$$\n\n```mermaid\ngraph TD\n  A --> B\n```\n\n- [x] done\n- [ ] open\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n<script>alert(1)</script>\n\n[app link](inkdrop://note/abc) and [web](https://example.com)\n";
        let html = page("My <note>", note, &Palette::LIGHT, &none);
        assert!(html.starts_with("<!doctype html>"));
        assert!(html.contains("<title>My &lt;note&gt;</title>"));
        assert!(html.contains("class=\"kanso-math\""));
        assert!(html.contains("class=\"kanso-math-display\""));
        assert!(html.contains("class=\"kanso-diagram\""));
        assert!(html.contains("<table>"));
        assert!(html.contains("type=\"checkbox\""));
        // No scripts, and the policy forbids them anyway.
        assert!(!html.contains("<script"));
        assert!(html.contains("default-src 'none'"));
        assert!(html.contains("<a>app link</a>"));
        assert!(html.contains("href=\"https://example.com\""));
        assert!(!html.contains("kansopiece"));
    }

    /// The site of the tests: "Plans" and note:abc are published.
    fn plans(target: NoteRef<'_>) -> Option<String> {
        match target {
            NoteRef::Title(title) if title.eq_ignore_ascii_case("plans") => Some("plans".into()),
            NoteRef::Id("note:abc") => Some("abc-page".into()),
            _ => None,
        }
    }

    #[test]
    fn a_site_page_links_to_its_published_notes_only() {
        let site = Site {
            resolve: &plans,
            home: "Work & play",
        };
        let note = format!(
            "See [[Plans#Next]], [[Secret]] and [[plans|the plan]].\n\n[by id]({NOTE_LINK_PREFIX}abc) and [hidden]({NOTE_LINK_PREFIX}zzz) and `[[Plans]]`\n"
        );
        let html = page_in("Note", &note, &Palette::LIGHT, &none, Some(&site));
        assert!(
            html.contains(r#"<a class="kanso-wikilink" href="kanso-page:plans">Plans › Next</a>"#)
        );
        assert!(html.contains(r#"<a class="kanso-wikilink" href="kanso-page:plans">the plan</a>"#));
        assert!(html.contains(r#"<span class="kanso-wikilink">Secret</span>"#));
        assert!(html.contains(r#"<a href="kanso-page:abc-page">by id</a>"#));
        assert!(html.contains("<a>hidden</a>"));
        assert!(html.contains("<code>[[Plans]]</code>"));
        assert!(html.contains(
            r#"<nav class="kanso-site-nav"><a href="kanso-page:">← Work &amp; play</a></nav>"#
        ));
        // Without a site, the same note links nowhere.
        let plain = page("Note", &note, &Palette::LIGHT, &none);
        assert!(!plain.contains("kanso-page:"));
        assert!(plain.contains("<a>by id</a>"));
    }

    #[test]
    fn a_site_index_lists_its_notes_by_notebook() {
        let entry = |slug: &str, title: &str, group: Option<&str>| IndexEntry {
            slug: slug.into(),
            title: title.into(),
            snippet: format!("About {title}"),
            group: group.map(str::to_owned),
        };
        let html = site_index(
            "Work",
            &[
                entry("a", "A <b>", None),
                entry("b", "", None),
                entry("c", "C", Some("Work › Old")),
            ],
            &Palette::DARK,
        );
        assert!(html.contains("<title>Work</title>"));
        assert!(html.contains(
            r#"<li><a href="kanso-page:a">A &lt;b&gt;</a><p>About A &lt;b&gt;</p></li>"#
        ));
        assert!(html.contains(r#"<a href="kanso-page:b">Untitled</a>"#));
        assert!(html.find("<h2>Work › Old</h2>").unwrap() > html.find("kanso-page:b").unwrap());
        assert_eq!(html.matches("<ul class=\"kanso-site-index\">").count(), 2);
        assert!(
            site_index("Empty", &[], &Palette::LIGHT).contains("Nothing is published here yet.")
        );
    }

    #[test]
    fn one_line_double_dollars_are_display_math() {
        let html = body(
            "$$\\sum_{k=1}^n k$$\n\nText $$x$$ inline.\n",
            &Palette::LIGHT,
            &none,
        );
        assert_eq!(html.matches("class=\"kanso-math-display\"").count(), 1);
        assert_eq!(html.matches("class=\"kanso-math\"").count(), 1);
        assert!(!html.contains("$$"));
    }

    /// Whether `[[` remains outside an inline or fenced code element.
    fn wikilink_markup_outside_code(html: &str) -> bool {
        html.match_indices("[[").any(|(at, _)| {
            let open = html[..at].rfind("<code");
            let close = html[..at].rfind("</code>");
            open.is_none_or(|open| close.is_some_and(|close| close > open))
        })
    }

    proptest! {
        /// Invariant: in a site page, every link to another page goes to a
        /// page the site said is published (or to the index), and no
        /// wiki-link is left as brackets outside code.
        #[test]
        fn prop_site_links_only_reach_published_pages(
            parts in prop::collection::vec(prop_oneof![
                "[a-z ]{0,8}",
                Just("[[Plans]]".to_owned()),
                Just("[[Secret|s]]".to_owned()),
                Just("[[plans#x]]".to_owned()),
                Just(format!("[t]({NOTE_LINK_PREFIX}abc)")),
                Just(format!("[u]({NOTE_LINK_PREFIX}q)")),
                Just("[w](https://example.com)".to_owned()),
            ], 0..10),
        ) {
            let site = Site { resolve: &plans, home: "Home" };
            let note = parts.join(" ");
            let html = page_in("T", &note, &Palette::LIGHT, &none, Some(&site));
            for (at, _) in html.match_indices("href=\"kanso-page:") {
                let rest = &html[at + "href=\"kanso-page:".len()..];
                let slug = &rest[..rest.find('"').unwrap()];
                prop_assert!(["", "plans", "abc-page"].contains(&slug), "{}", slug);
            }
            prop_assert!(!wikilink_markup_outside_code(&html));
        }
    }

    proptest! {
        /// Invariants: a paragraph is display math exactly when it is one
        /// `$$…$$` and nothing else, and the formula is what is inside.
        #[test]
        fn prop_display_math_paragraphs(formula in "[a-z0-9+=^]{1,12}", before in "[A-Za-z ]{0,6}") {
            // Four spaces start an indented code block, which is not a paragraph.
            prop_assume!(!before.starts_with("    "));
            let markdown = format!("{before}$${formula}$$");
            let tree = markdown::to_mdast(&markdown, &parse_options()).unwrap();
            let Some(paragraph) = tree.children().and_then(|c| c.first()) else {
                return Ok(());
            };
            let found = display_math_paragraph(paragraph, &markdown);
            if before.trim().is_empty() {
                prop_assert_eq!(found, Some(formula.as_str()));
            } else {
                prop_assert_eq!(found, None);
            }
        }
    }

    #[test]
    fn attachments_are_inlined() {
        let png = vec![0x89, b'P', b'N', b'G'];
        let attachments =
            |id: &str| (id == "file:abc").then(|| ("image/png".to_owned(), png.clone()));
        let html = body(
            "![Cat](inkdrop://file:abc)\n\n<img src=\"inkdrop://file:abc\" width=\"240\">\n\n![Gone](inkdrop://file:zzz)\n",
            &Palette::DARK,
            &attachments,
        );
        assert_eq!(
            html.matches("src=\"data:image/png;base64,iVBORw==\"")
                .count(),
            2,
            "{html}"
        );
        assert!(html.contains("width=\"240\""));
        assert!(html.contains("kanso-missing\">Gone<"));
    }

    #[test]
    fn diagrams_that_cannot_be_drawn_stay_code() {
        let html = body(
            "```mermaid\nsparkleDiagram\n  a\n```\n",
            &Palette::LIGHT,
            &none,
        );
        assert!(html.contains("<pre><code class=\"language-mermaid\">"));
        let html = body("Price $5 and $10.\n", &Palette::LIGHT, &none);
        assert!(html.contains("$5"), "{html}");
    }

    /// markdown 1.0.0 panics on this; the export shows it as plain text.
    #[test]
    fn text_markdown_rs_panics_on_exports_as_plain_paragraphs() {
        let html = body("Para\n---\n---\na\n===\n\nNext <b>", &Palette::LIGHT, &none);
        assert_eq!(
            html,
            "<p>Para<br>\n---<br>\n---<br>\na<br>\n===</p>\n<p>Next &lt;b&gt;</p>\n"
        );
    }

    #[test]
    fn wiki_links_export_as_their_labels() {
        let html = body(
            "See [[Plans]], [[Plans#Next steps]], [[Plans|the plan]] and [[#Intro]].\n\n`[[kept]]` stays code.\n",
            &Palette::LIGHT,
            &none,
        );
        assert!(
            html.contains(r#"<span class="kanso-wikilink">Plans</span>,"#),
            "{html}"
        );
        assert!(html.contains(r#"<span class="kanso-wikilink">Plans › Next steps</span>"#));
        assert!(html.contains(r#"<span class="kanso-wikilink">the plan</span>"#));
        assert!(html.contains(r#"<span class="kanso-wikilink">Intro</span>"#));
        assert!(html.contains("<code>[[kept]]</code>"), "{html}");
        let html = body(
            "A [[*big* idea]] here, and [[]] stays.\n",
            &Palette::LIGHT,
            &none,
        );
        assert!(
            html.contains(r#"<span class="kanso-wikilink"><em>big</em> idea</span>"#),
            "{html}"
        );
        assert!(html.contains("[[]]"));
    }

    #[test]
    fn image_widths_after_a_bar() {
        assert_eq!(split_image_width("Cat|300"), ("Cat", Some(300)));
        assert_eq!(split_image_width("Cat |300x200"), ("Cat", Some(300)));
        assert_eq!(split_image_width("|120"), ("", Some(120)));
        assert_eq!(split_image_width("a|b"), ("a|b", None));
        assert_eq!(split_image_width("Cat|0"), ("Cat|0", None));
        assert_eq!(split_image_width("Cat|9999999"), ("Cat|9999999", None));
        assert_eq!(split_image_width("Cat"), ("Cat", None));
        let png = vec![0x89, b'P', b'N', b'G'];
        let attachments =
            |id: &str| (id == "file:abc").then(|| ("image/png".to_owned(), png.clone()));
        let html = body(
            "![Cat|300](inkdrop://file:abc)\n\n![Web|200](https://example.com/a.png)\n",
            &Palette::LIGHT,
            &attachments,
        );
        assert!(html.contains(r#"alt="Cat" width="300""#), "{html}");
        assert!(
            html.contains(r#"<img src="https://example.com/a.png" alt="Web" width="200">"#),
            "{html}"
        );
        assert!(!html.contains("|300") && !html.contains("|200"));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        /// Invariant: no wiki-link survives export as literal brackets,
        /// whatever its title, heading or alias.
        #[test]
        fn prop_wiki_links_leave_no_brackets(
            title in "[A-Za-z0-9][A-Za-z0-9 ,.'&-]{0,20}",
            heading in prop::option::of("[A-Za-z0-9][A-Za-z0-9 ]{0,10}"),
            alias in prop::option::of("[A-Za-z0-9][A-Za-z0-9 !?]{0,10}"),
            before in "([A-Za-z][A-Za-z ]{0,9})?",
            after in "[A-Za-z .]{0,10}",
            heading_line in any::<bool>(),
        ) {
            let mut link = title.clone();
            if let Some(heading) = &heading {
                link = format!("{link}#{heading}");
            }
            if let Some(alias) = &alias {
                link = format!("{link}|{alias}");
            }
            let line = format!("{before}[[{link}]]{after}");
            let markdown = if heading_line { format!("## {line}\n") } else { format!("{line}\n\n- {line}\n") };
            let html = page("t", &markdown, &Palette::LIGHT, &none);
            prop_assert!(!html.contains("[["), "{}", html);
            prop_assert!(html.contains("kanso-wikilink"));
        }
    }

    #[test]
    fn img_tags_read() {
        assert_eq!(
            img_tag(r#"<img src="inkdrop://file:a" alt='A b' width="240px">"#),
            Some(("inkdrop://file:a".into(), "A b".into(), Some(240)))
        );
        assert_eq!(img_tag("<imgx src=a>"), None);
        assert_eq!(img_tag("<img alt=a>"), None);
        assert_eq!(
            img_tag("<img src=a/>"),
            Some(("a".into(), String::new(), None))
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        /// Invariants: any note exports to a page with no script element
        /// and no placeholder left behind.
        #[test]
        fn prop_pages_have_no_scripts(text in "(\\PC|\n|<script>|\\$|`|\\[|\\]|\\(|\\))*") {
            let html = page("t", &text, &Palette::LIGHT, &none);
            let lower = html.to_ascii_lowercase();
            prop_assert!(!lower.contains("<script"));
            prop_assert!(!html.contains("kansopiece"));
        }

        /// Round trip: an attachment ID inside a URL is found again.
        #[test]
        fn prop_attachment_ids_round_trip(id in "[A-Za-z0-9_-]{1,20}", tail in prop::option::of("[?#][a-z=]{0,6}")) {
            let url = format!("inkdrop://file:{id}{}", tail.unwrap_or_default());
            prop_assert_eq!(attachment_id(&url), Some(format!("file:{id}")));
        }
    }
    /// A highlighter for the tests: the first byte red and bold.
    struct RedFirst;

    impl Highlighter for RedFirst {
        fn highlight(&self, _: &str, code: &str) -> Option<Vec<CodeSpan>> {
            (!code.is_empty()).then(|| {
                vec![CodeSpan {
                    start: 0,
                    end: 1,
                    color: Some(0xFF0000),
                    bold: true,
                    italic: false,
                }]
            })
        }
    }

    #[test]
    fn highlighted_code_becomes_spans_with_its_language() {
        let highlight = RedFirst;
        let options = PageOptions {
            highlight: Some(&highlight),
            ..PageOptions::default()
        };
        let html = page_with(
            "Code",
            "```rust\n<fn> & main\n```\n",
            &Palette::LIGHT,
            &none,
            &options,
        );
        assert!(html.contains(r#"<pre class="kanso-code">"#), "{html}");
        assert!(
            html.contains(r#"<span class="kanso-lang">rust</span>"#),
            "{html}"
        );
        assert!(
            html.contains(r#"<span style="color:#ff0000;font-weight:bold;">&lt;</span>"#),
            "{html}"
        );
        assert!(html.contains("fn&gt; &amp; main"), "{html}");
        assert!(!html.contains("<figure>"), "{html}");
        assert!(!html.contains("language-rust"), "{html}");
    }

    #[test]
    fn a_site_and_its_index_use_the_page_style() {
        let options = PageOptions {
            font: Some("Alegreya"),
            size: 18.,
            ..PageOptions::default()
        };
        let site = Site {
            resolve: &plans,
            home: "Work",
        };
        let page = page_in_with(
            "Note",
            "See [[Plans]].\n",
            &Palette::LIGHT,
            &none,
            Some(&site),
            &options,
        );
        assert!(page.contains("font:18px/"), "{page}");
        assert!(page.contains("\"Alegreya\""), "{page}");
        assert!(page.contains("class=\"kanso-site-nav\""), "{page}");
        let index = site_index_with(
            "Work",
            &[IndexEntry {
                slug: "a".into(),
                title: "A".into(),
                snippet: String::new(),
                group: None,
            }],
            &Palette::LIGHT,
            &options,
        );
        assert!(index.contains("font:18px/"), "{index}");
        assert!(index.contains("\"Alegreya\""), "{index}");
        assert!(index.contains("kanso-site-index"), "{index}");
    }

    #[test]
    fn a_family_that_could_break_the_style_falls_back() {
        let options = PageOptions {
            font: Some("Bad; }</style><script>x"),
            mono: Some("Bad\"{}\""),
            ..PageOptions::default()
        };
        let css = stylesheet(&Palette::LIGHT, &options);
        assert!(!css.contains("Bad"), "{css}");
        assert!(!css.to_ascii_lowercase().contains("</style"));
        assert!(!css.to_ascii_lowercase().contains("<script"));
        assert!(css.contains("-apple-system"), "{css}");
        assert!(css.contains("ui-monospace"), "{css}");
    }

    #[test]
    fn task_boxes_are_drawn_by_the_page() {
        let css = stylesheet(&Palette::LIGHT, &PageOptions::default());
        assert!(css.contains("appearance:none"), "{css}");
        assert!(css.contains("li:has(>input[type=checkbox])"), "{css}");
        assert!(css.contains("data:image/svg+xml"), "{css}");
    }

    #[test]
    fn the_stylesheet_uses_the_previews_numbers() {
        let options = PageOptions {
            font: Some("Alegreya"),
            mono: Some("Iosevka"),
            size: 18.,
            highlight: None,
            app: None,
        };
        let css = stylesheet(&Palette::LIGHT, &options);
        let width = crate::typography::readable_width(18., false);
        assert!(
            css.contains(&format!("max-width:calc({width}px + 4rem)")),
            "{css}"
        );
        assert!(css.contains("font:18px/1.65"), "{css}");
        assert!(css.contains("\"Alegreya\""), "{css}");
        assert!(css.contains("\"Iosevka\""), "{css}");
        let h1 = crate::typography::heading(1);
        assert!(
            css.contains(&format!(
                "font-size:{}em;line-height:1.25;margin:{}em 0 {}em",
                h1.scale,
                h1.gap,
                crate::typography::HEADING_BOTTOM_GAP
            )),
            "{css}"
        );
        assert!(
            css.contains(&format!(
                "border-radius:{}px",
                crate::typography::CODE_RADIUS
            )),
            "{css}"
        );
        assert!(css.contains("margin:0 0 0.9em"), "{css}");
        assert!(!css.contains("46rem"), "{css}");
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        /// Invariants: any palette, size and family name make a stylesheet
        /// with balanced braces and no way to close the style element.
        #[test]
        fn prop_stylesheets_are_well_formed(
            bg in any::<u32>(),
            text in any::<u32>(),
            family in "[^\0]{0,32}",
            mono in "[^\0]{0,32}",
            size in any::<f32>(),
        ) {
            let palette = Palette {
                background: Rgb(bg & 0xFF_FFFF),
                text: Rgb(text & 0xFF_FFFF),
                ..Palette::LIGHT
            };
            let options = PageOptions {
                font: Some(&family),
                mono: Some(&mono),
                size,
                highlight: None,
                app: None,
            };
            let css = stylesheet(&palette, &options);
            prop_assert_eq!(css.matches('{').count(), css.matches('}').count());
            prop_assert!(!css.to_ascii_lowercase().contains("</style"));
            prop_assert!(!css.to_ascii_lowercase().contains("<script"));
            prop_assert_eq!(css.matches('"').count() % 2, 0);
            prop_assert!(css.contains("max-width:calc("));
        }

        /// Invariants: a highlighted page escapes its code and never emits
        /// a span outside the code.
        #[test]
        fn prop_highlighted_code_is_escaped(code in "[<>&\"'a-z0-9 ]{0,60}") {
            prop_assume!(!code.is_empty());
            let highlight = RedFirst;
            let options = PageOptions {
                highlight: Some(&highlight),
                ..PageOptions::default()
            };
            let html = page_with(
                "t",
                &format!("```rust\n{code}\n```\n"),
                &Palette::LIGHT,
                &none,
                &options,
            );
            prop_assert!(!html.contains("<script"));
            prop_assert!(html.contains("<pre class=\"kanso-code\">"));
            prop_assert!(html.contains("color:#ff0000"));
        }
    }
}
