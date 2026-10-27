//! The page "in the app" (#48 step 7): the same page publishing makes, with
//! the addresses the phone's app understands kept on its links, and nothing
//! else a page could do.
//!
//! In this mode ([`AppMode`], on [`PageOptions`](super::PageOptions)):
//!
//! - links into the app keep their address: a note (`inkdrop://note/<id>`,
//!   with `#Heading` for a wiki-link to one), an attachment
//!   (`inkdrop://file:<id>`), a note to make (`kanso-new:`), a tag
//!   (`kanso-tag:`), a due date (`kanso-due:`), a task box (`kanso-task:<n>`)
//!   and a place in the same page (`#slug`); web and mail links keep theirs;
//!   every other address is dropped, and so is every other `href` or `src`
//!   ([`is_allowed_link`], applied to the whole page by [`sanitize`]);
//! - headings carry stable `id`s ([`heading_slug`]) so `#slug` links scroll
//!   without a script;
//! - each task box is a link carrying its place in the note's task list, the
//!   n-th box being `task_items(body)[n]` of `kanso-core`;
//! - a `query` fence is a table of the notes it lists;
//! - a due date is a chip.
//!
//! The page has no scripts, no event handlers and fetches nothing: the
//! scrubber removes what could, whatever the note holds.

use std::fmt::Write as _;

use markdown::mdast::Node;

use crate::{Palette, Rgb, escape};

/// A task of the note's task list (`kanso_core::tasks::task_items`): the line
/// (0-based) its box is on and whether it is ticked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskSlot {
    pub line: u32,
    pub checked: bool,
}

/// The table a `query` fence shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QueryResult {
    /// The notes it lists.
    Table(QueryTable),
    /// The query can't be read; the message says why.
    Invalid { message: String },
}

/// A query view's table: column labels (the first is the title) and rows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryTable {
    pub columns: Vec<String>,
    pub rows: Vec<QueryRow>,
}

/// A row of a query view: the note it links to (`note:…`), its title and a
/// cell for each column after the title.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryRow {
    pub id: String,
    pub title: String,
    pub cells: Vec<String>,
}

/// Runs a `query` fence for the page. The page never reads the store.
pub trait QueryViews: Send + Sync {
    /// The table for the fence whose text is `source`.
    fn run(&self, source: &str) -> QueryResult;
}

/// How near a due date feels, which colors its chip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DueTone {
    Plain,
    Today,
    Overdue,
    Done,
}

/// A due date as a chip: its words, its longer words and its tone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DueChip {
    pub label: String,
    pub title: String,
    pub tone: DueTone,
}

/// Turns a `kanso-due:` address into a chip, if it is one.
pub type DueChips<'a> = &'a dyn Fn(&str) -> Option<DueChip>;

/// What the page in the app needs from its surroundings.
#[derive(Clone, Copy, Default)]
pub struct AppMode<'a> {
    /// The note's task list, to number the boxes by. A box that no entry
    /// names is drawn as it is on a published page, without a link.
    pub tasks: &'a [TaskSlot],
    /// Fills `query` fences; without it they stay code.
    pub query: Option<&'a dyn QueryViews>,
    /// Turns a `kanso-due:` address into a chip; without it the link stays.
    pub due: Option<DueChips<'a>>,
}

/// The address of a task box: this, then its place in the task list.
pub const TASK_LINK_PREFIX: &str = "kanso-task:";
/// The address of a tag: this, then the name, percent-encoded.
pub const TAG_LINK_PREFIX: &str = "kanso-tag:";
/// The address of a due date.
pub const DUE_LINK_PREFIX: &str = "kanso-due:";
/// The address of a note to make.
pub const NEW_NOTE_PREFIX: &str = "kanso-new:";
/// The address of a code block's copy link: this, then the block's place in
/// [`code_blocks`].
pub const COPY_LINK_PREFIX: &str = "kanso-copy:";

/// A code block's copy link: a button in its corner that the app answers by
/// copying the block (the page has no script to do it).
pub(super) fn copy_link(n: usize) -> String {
    format!(
        "<a class=\"kanso-copy\" href=\"{COPY_LINK_PREFIX}{n}\" aria-label=\"Copy code\"><svg viewBox=\"0 0 24 24\" width=\"15\" height=\"15\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><rect x=\"9\" y=\"9\" width=\"13\" height=\"13\" rx=\"2\"/><path d=\"M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1\"/></svg></a>"
    )
}

/// A source that couldn't be drawn: its code as written, with a badge
/// saying why (and the details as the badge's title), as the desktop's
/// preview shows it.
pub(super) fn failed_html(
    failure: &crate::Failure,
    lang: Option<&str>,
    code: &str,
    copy: Option<usize>,
) -> String {
    let detail = failure.detail();
    let title = if detail.is_empty() {
        String::new()
    } else {
        format!(" title=\"{}\"", escape(detail))
    };
    format!(
        "<div class=\"kanso-failed\"><span class=\"kanso-query-badge\"{title}>{}</span>{}</div>",
        escape(&failure.badge()),
        super::code_html(lang, code, &[], copy)
    )
}

/// What an Excalidraw drawing with nothing in it shows: a quiet dashed
/// canvas that says so.
pub(super) const EMPTY_DRAWING: &str =
    "<div class=\"kanso-empty-drawing\"><span>An empty drawing</span></div>";

/// Whether a drawing's JSON is a scene with nothing in it yet: what "New
/// drawing" writes (the desktop's `is_empty_scene`). Pure.
pub fn is_empty_scene(source: &str) -> bool {
    match serde_json::from_str::<serde_json::Value>(source) {
        Ok(serde_json::Value::Object(scene)) => scene
            .get("elements")
            .and_then(|e| e.as_array())
            .is_some_and(|elements| elements.is_empty()),
        Ok(serde_json::Value::Array(elements)) => elements.is_empty(),
        _ => false,
    }
}

// MARK: Headings

/// A heading's words as plain text and the `id` the page gives it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeadingId {
    pub level: u8,
    pub text: String,
    pub id: String,
}

/// A title as wiki-links match it (`kanso_core::links::normalize_title`):
/// trimmed, inner whitespace collapsed, lowercased.
fn normalized(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// The `id` a heading's words make: the words as `[[Title#Heading]]` matches
/// them (case and spacing ignored), letters and digits kept, other
/// punctuation dropped, spaces and hyphens one hyphen. "section" for words
/// with none. Pure.
pub fn heading_slug(text: &str) -> String {
    let mut out = String::new();
    let mut gap = false;
    for c in normalized(text).chars() {
        if c.is_alphanumeric() || c == '_' {
            if gap && !out.is_empty() {
                out.push('-');
            }
            gap = false;
            out.push(c);
        } else if c.is_whitespace() || c == '-' {
            gap = true;
        }
    }
    if out.is_empty() {
        out.push_str("section");
    }
    out
}

/// The `id`s of headings with these words, in order: a slug taken before
/// gets `-1`, `-2`, … after it, so every `id` in a page is its own. Pure.
fn unique_slugs(texts: &[String]) -> Vec<String> {
    let mut taken = std::collections::HashSet::new();
    texts
        .iter()
        .map(|text| {
            let base = heading_slug(text);
            let mut id = base.clone();
            let mut n = 0;
            while !taken.insert(id.clone()) {
                n += 1;
                id = format!("{base}-{n}");
            }
            id
        })
        .collect()
}

/// What the Markdown syntax tree says the page needs: headings and task
/// boxes, in document order.
pub(super) struct Facts {
    headings: Vec<(u8, String)>,
    /// Each task item.
    tasks: Vec<TaskNode>,
    /// The start offset and text of each code block (fenced or indented).
    codes: Vec<(usize, String)>,
}

/// A task item of the Markdown syntax tree.
#[derive(Clone, Copy, Debug)]
struct TaskNode {
    /// Where the item starts (its list marker), in bytes.
    offset: usize,
    /// The line it is on, 0-based.
    line: u32,
    checked: bool,
}

impl Facts {
    /// Where each code block starts, in order: its place in the list is its
    /// copy link's number.
    pub(super) fn code_offsets(&self) -> Vec<usize> {
        self.codes.iter().map(|(at, _)| *at).collect()
    }
}

pub(super) fn facts(root: &Node) -> Facts {
    fn walk(node: &Node, facts: &mut Facts) {
        match node {
            Node::Heading(heading) => {
                let text = heading.children.iter().map(Node::to_string).collect();
                facts.headings.push((heading.depth, text));
            }
            Node::Code(code) => {
                if let Some(position) = &node.position() {
                    facts
                        .codes
                        .push((position.start.offset, code.value.clone()));
                }
            }
            Node::ListItem(item) => {
                if let (Some(checked), Some(position)) = (item.checked, &item.position) {
                    facts.tasks.push(TaskNode {
                        offset: position.start.offset,
                        line: position.start.line.saturating_sub(1) as u32,
                        checked,
                    });
                }
            }
            _ => {}
        }
        for child in node.children().into_iter().flatten() {
            walk(child, facts);
        }
    }
    let mut facts = Facts {
        headings: Vec::new(),
        tasks: Vec::new(),
        codes: Vec::new(),
    };
    walk(root, &mut facts);
    facts
}

/// The text of each code block of a note, in order (fenced and indented,
/// drawings' sources included): the n-th is what `kanso-copy:n` copies. Pure.
pub fn code_blocks(markdown: &str) -> Vec<String> {
    match crate::markdown_safe::to_mdast(markdown, &super::parse_options()) {
        Ok(root) => facts(&root)
            .codes
            .into_iter()
            .map(|(_, code)| code)
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// The headings of a note as the page in the app numbers them, in order.
/// Pure.
pub fn headings(markdown: &str) -> Vec<HeadingId> {
    let Ok(root) = crate::markdown_safe::to_mdast(markdown, &super::parse_options()) else {
        return Vec::new();
    };
    let found = facts(&root).headings;
    let ids = unique_slugs(
        &found
            .iter()
            .map(|(_, text)| text.clone())
            .collect::<Vec<_>>(),
    );
    found
        .into_iter()
        .zip(ids)
        .map(|((level, text), id)| HeadingId { level, text, id })
        .collect()
}

/// `<h2>` becomes `<h2 id="…">` for each heading of `facts`, as long as the
/// page's headings are the tree's.
fn give_heading_ids(html: &str, found: &[HeadingId]) -> String {
    let mut out = String::with_capacity(html.len() + found.len() * 24);
    let mut rest = html;
    let mut next = found.iter();
    'tags: while let Some(at) = rest.find("<h") {
        let tag = rest.as_bytes();
        let level = tag.get(at + 2).copied().unwrap_or(0);
        let plain = (b'1'..=b'6').contains(&level) && tag.get(at + 3) == Some(&b'>');
        out.push_str(&rest[..at]);
        if plain {
            match next.next() {
                Some(heading) if heading.level == level - b'0' => {
                    let _ = write!(out, "<h{} id=\"{}\">", heading.level, escape(&heading.id));
                    rest = &rest[at + 4..];
                    continue 'tags;
                }
                // The page's headings aren't the tree's: leave them alone.
                _ => {
                    out.push_str(&rest[at..]);
                    return out;
                }
            }
        }
        out.push_str("<h");
        rest = &rest[at + 2..];
    }
    out.push_str(rest);
    out
}

/// Percent-decoding of a fragment (`%C3%A9` and so on); bytes that don't
/// decode stay as they are. Pure.
pub fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && let Some(hex) = text.get(i + 1..i + 3)
            && let Ok(byte) = u8::from_str_radix(hex, 16)
        {
            out.push(byte);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `&amp;`, `&quot;`, `&lt;`, `&gt;` and `&#39;` as the characters; any other
/// reference is left as it is. Pure.
fn unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

/// Links to a place in the page (`<a href="#…">`) point at a heading's `id`:
/// one that is already an `id` stays, one that names a heading's words
/// (`[[…]]`-style, any case) becomes its `id`, and any other loses its address.
fn resolve_fragments(html: &str, found: &[HeadingId]) -> String {
    const OPEN: &str = "<a href=\"#";
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(at) = rest.find(OPEN) {
        out.push_str(&rest[..at]);
        let after = &rest[at + OPEN.len()..];
        let Some(end) = after.find('"') else {
            out.push_str(&rest[at..]);
            return out;
        };
        let wanted = percent_decode(&unescape(&after[..end]));
        let id = found
            .iter()
            .find(|heading| heading.id == wanted)
            .or_else(|| {
                let words = normalized(&wanted);
                found.iter().find(|h| normalized(&h.text) == words)
            })
            .map(|heading| heading.id.as_str());
        match id {
            Some(id) => {
                let _ = write!(out, "{OPEN}{}\"", escape(id));
            }
            None => out.push_str("<a"),
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}

// MARK: Tasks

/// Whether the text has a carriage return that isn't half of a line break:
/// Markdown counts it as a line end and Rust's `lines` does not, so the
/// lines of the tree and of the task list could differ.
pub(super) fn has_lone_cr(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes
        .iter()
        .enumerate()
        .any(|(i, &b)| b == b'\r' && bytes.get(i + 1) != Some(&b'\n'))
}

/// The slot of `slots` each task of the tree is, in the tree's order, or
/// `None` where the tree and the slots don't agree on it: no slot is on its
/// line and ticked the same way, or another task of the tree shares its line
/// (`- [ ] - [ ] x`), which the slots, one to a line, can't tell apart.
fn match_slots(tasks: &[TaskNode], slots: &[TaskSlot]) -> Vec<Option<usize>> {
    tasks
        .iter()
        .map(|task| {
            let alone = tasks.iter().filter(|other| other.line == task.line).count() == 1;
            alone
                .then(|| {
                    slots
                        .iter()
                        .position(|slot| slot.line == task.line && slot.checked == task.checked)
                })
                .flatten()
        })
        .collect()
}

/// A task box of a note, as its syntax tree has it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskBox {
    /// Where its list item starts (the list marker), in bytes of the note.
    pub offset: usize,
    /// The line it is on, 0-based.
    pub line: u32,
    pub checked: bool,
    /// Its place among the note's task slots (`kanso_core::tasks::task_items`,
    /// so the index `toggle_task` takes), or `None` when that can't be known
    /// with certainty. A box without one must not be ticked.
    pub slot: Option<usize>,
}

/// The task boxes of the note whose syntax tree is `root`, in document
/// order, each with its place in `slots` where the tree and the slots agree
/// on it (the rule the page's linked boxes follow). A note with a lone
/// carriage return, which Markdown counts as a line end and the slots do not,
/// has no slots to give. Pure.
pub fn task_boxes(root: &Node, source: &str, slots: &[TaskSlot]) -> Vec<TaskBox> {
    let tasks = facts(root).tasks;
    let matched = if has_lone_cr(source) {
        vec![None; tasks.len()]
    } else {
        match_slots(&tasks, slots)
    };
    tasks
        .iter()
        .zip(matched)
        .map(|(task, slot)| TaskBox {
            offset: task.offset,
            line: task.line,
            checked: task.checked,
            slot,
        })
        .collect()
}

/// Each task box of the page as a link carrying the box's place in `slots`
/// (the n-th is `kanso-task:n`). A box the tree and the slots don't agree on
/// stays a plain box.
fn link_task_boxes(html: &str, facts: &Facts, slots: &[TaskSlot]) -> String {
    const BOX: &str = "<input type=\"checkbox\"";
    if html.matches(BOX).count() != facts.tasks.len() {
        return html.to_owned();
    }
    let mut out = String::with_capacity(html.len() + facts.tasks.len() * 96);
    let mut rest = html;
    let matched = match_slots(&facts.tasks, slots);
    for (task, slot) in facts.tasks.iter().zip(matched) {
        let checked = task.checked;
        let Some(at) = rest.find(BOX) else { break };
        let Some(len) = rest[at..].find("/>") else {
            break;
        };
        let input = &rest[at..at + len + 2];
        out.push_str(&rest[..at]);
        match slot {
            Some(n) if input.contains("checked=\"\"") == checked => {
                let (class, state) = if checked {
                    ("kanso-task kanso-task-done", "true")
                } else {
                    ("kanso-task", "false")
                };
                let _ = write!(
                    out,
                    "<a class=\"{class}\" href=\"{TASK_LINK_PREFIX}{n}\" role=\"checkbox\" aria-checked=\"{state}\"><span class=\"kanso-box\"></span></a>"
                );
            }
            _ => out.push_str(input),
        }
        rest = &rest[at + len + 2..];
    }
    out.push_str(rest);
    out
}

// MARK: Due dates

/// Each `kanso-due:` link as its chip, where `chip` makes one.
fn draw_due_chips(html: &str, chip: &dyn Fn(&str) -> Option<DueChip>) -> String {
    const OPEN: &str = "<a href=\"kanso-due:";
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(at) = rest.find(OPEN) {
        out.push_str(&rest[..at]);
        let after = &rest[at + OPEN.len()..];
        let (Some(quote), Some(close)) = (after.find('"'), after.find("</a>")) else {
            out.push_str(&rest[at..]);
            return out;
        };
        let url = format!("{DUE_LINK_PREFIX}{}", unescape(&after[..quote]));
        let end = at + OPEN.len() + close + 4;
        match chip(&url).filter(|_| quote < close) {
            Some(chip) => {
                let tone = match chip.tone {
                    DueTone::Plain => "",
                    DueTone::Today => " kanso-due-today",
                    DueTone::Overdue => " kanso-due-overdue",
                    DueTone::Done => " kanso-due-done",
                };
                let _ = write!(
                    out,
                    "<a class=\"kanso-due{tone}\" href=\"{}\" title=\"{}\">{}</a>",
                    escape(&url),
                    escape(&chip.title),
                    escape(&chip.label)
                );
            }
            None => out.push_str(&rest[at..end]),
        }
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

// MARK: Query views

/// Whether a fence's language names a query view: the first word is `query`
/// in any case (`kanso_core::query_view::is_query_fence`).
pub(super) fn is_query_fence(lang: Option<&str>) -> bool {
    lang.and_then(|lang| lang.split_whitespace().next())
        .is_some_and(|lang| lang.eq_ignore_ascii_case("query"))
}

/// The address of the note `id` (`note:…`).
pub fn note_address(id: &str) -> String {
    format!(
        "{}{}",
        super::NOTE_LINK_PREFIX,
        id.strip_prefix("note:").unwrap_or(id)
    )
}

/// A query fence as the page draws it: a table with a link to each note, or
/// the query's text with a badge and the reason when it can't be read.
pub(super) fn query_html(source: &str, result: &QueryResult) -> String {
    let mut out = String::from("<div class=\"kanso-query\">");
    match result {
        QueryResult::Invalid { message } => {
            let _ = write!(
                out,
                "<span class=\"kanso-query-badge\">Query</span><pre class=\"kanso-code\"><code>{}</code></pre><p class=\"kanso-query-why\">{}</p>",
                escape(source),
                escape(message)
            );
        }
        QueryResult::Table(table) => {
            out.push_str("<table><thead><tr>");
            for label in &table.columns {
                let _ = write!(out, "<th>{}</th>", escape(label));
            }
            out.push_str("</tr></thead>");
            if !table.rows.is_empty() {
                out.push_str("<tbody>");
                for row in &table.rows {
                    let title = row.title.trim();
                    let _ = write!(
                        out,
                        "<tr><td><a href=\"{}\">{}</a></td>",
                        escape(&note_address(&row.id)),
                        escape(if title.is_empty() { "Untitled" } else { title })
                    );
                    for cell in &row.cells {
                        let _ = write!(out, "<td>{}</td>", escape(cell));
                    }
                    out.push_str("</tr>");
                }
                out.push_str("</tbody>");
            }
            out.push_str("</table>");
            if table.rows.is_empty() {
                out.push_str("<p class=\"kanso-query-empty\">No notes match.</p>");
            }
        }
    }
    out.push_str("</div>");
    out
}

// MARK: The page

/// The page's body in the app: the compiled HTML with headings numbered,
/// task boxes linked, due dates drawn as chips and everything a page must
/// not carry taken out.
pub(super) fn finish(
    html: String,
    facts: Option<&Facts>,
    mode: &AppMode<'_>,
    source: &str,
) -> String {
    let mut html = html;
    if let Some(facts) = facts {
        let found: Vec<HeadingId> = {
            let ids = unique_slugs(
                &facts
                    .headings
                    .iter()
                    .map(|(_, text)| text.clone())
                    .collect::<Vec<_>>(),
            );
            facts
                .headings
                .iter()
                .zip(ids)
                .map(|((level, text), id)| HeadingId {
                    level: *level,
                    text: text.clone(),
                    id,
                })
                .collect()
        };
        html = give_heading_ids(&html, &found);
        html = resolve_fragments(&html, &found);
        if !has_lone_cr(source) {
            html = link_task_boxes(&html, facts, mode.tasks);
        }
    }
    if let Some(chip) = mode.due {
        html = draw_due_chips(&html, chip);
    }
    sanitize(&html)
}

// MARK: What a page may carry

/// Whether a link's address may stay in the page in the app: web
/// (`http://`, `https://`), mail (`mailto:`), the app's own addresses
/// (`inkdrop://note/`, `inkdrop://file:`, `kanso-new:`, `kanso-tag:`,
/// `kanso-due:`, `kanso-task:`) and a place in the page (`#…`). The scheme
/// is read as written, in any case, so an address that hides its scheme
/// behind a character reference or a leading space is not one of these. Pure.
pub fn is_allowed_link(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    let after = |prefix: &str| lower.strip_prefix(prefix);
    after("http://").is_some_and(|rest| !rest.is_empty())
        || after("https://").is_some_and(|rest| !rest.is_empty())
        || after("mailto:").is_some_and(|rest| !rest.is_empty())
        || lower.starts_with("inkdrop://note/")
        || lower.starts_with("inkdrop://file:")
        || lower.starts_with(NEW_NOTE_PREFIX)
        || lower.starts_with(TAG_LINK_PREFIX)
        || lower.starts_with(DUE_LINK_PREFIX)
        || lower.starts_with(TASK_LINK_PREFIX)
        || lower.starts_with(COPY_LINK_PREFIX)
        || lower.starts_with('#')
}

/// Whether a loaded resource's address may stay: an inline image
/// (`data:image/…`). Nothing is fetched. Pure.
pub fn is_allowed_source(url: &str) -> bool {
    url.to_ascii_lowercase().starts_with("data:image/")
}

/// Tags the page never carries, with whatever is inside the ones that hold
/// script or markup of their own.
const BANNED: [&str; 24] = [
    "script",
    "iframe",
    "frame",
    "frameset",
    "object",
    "embed",
    "applet",
    "meta",
    "link",
    "base",
    "form",
    "button",
    "textarea",
    "select",
    "audio",
    "video",
    "source",
    "track",
    "foreignobject",
    "animate",
    "animatemotion",
    "animatetransform",
    "set",
    "noscript",
];
/// The banned tags whose content is not markup: it goes with the tag.
const BANNED_WITH_CONTENT: [&str; 7] = [
    "script", "iframe", "frameset", "object", "applet", "noscript", "textarea",
];

/// Attributes that fetch or act, and so never stay.
const BANNED_ATTRIBUTES: [&str; 12] = [
    "srcset",
    "action",
    "formaction",
    "poster",
    "background",
    "ping",
    "data",
    "longdesc",
    "lowsrc",
    "dynsrc",
    "codebase",
    "manifest",
];

/// Whether CSS holds nothing that fetches: every `url()` names an inline
/// image or a place in the page, and there is no `@import`, `image-set()`,
/// `src()`, escape or old script hook. Pure.
fn css_is_inert(css: &str) -> bool {
    let lower = css.to_ascii_lowercase();
    if lower.contains('\\')
        || [
            "@import",
            "image-set(",
            "src(",
            "cross-fade(",
            "element(",
            "expression(",
            "binding",
            "behavior",
            "@font-face",
            "javascript:",
        ]
        .iter()
        .any(|bad| lower.contains(bad))
    {
        return false;
    }
    let mut rest = lower.as_str();
    while let Some(at) = rest.find("url(") {
        let after = &rest[at + 4..];
        let Some(close) = after.find(')') else {
            return false;
        };
        let arg = after[..close].trim().trim_matches(['"', '\'']).trim();
        if !(arg.starts_with('#') || arg.starts_with("data:image/")) {
            return false;
        }
        rest = &after[close + 1..];
    }
    true
}

/// One attribute of a tag, name as written, value as written (character
/// references and all) without its quotes.
struct Attribute {
    name: String,
    value: Option<String>,
}

/// The attributes of the tag text between its name and its `>`, and whether
/// it ends `/`.
fn attributes(text: &str) -> (Vec<Attribute>, bool) {
    let text = text.trim_end();
    let (text, closes) = match text.strip_suffix('/') {
        Some(rest) => (rest, true),
        None => (text, false),
    };
    let mut found = Vec::new();
    let mut rest = text;
    loop {
        rest = rest.trim_start_matches(|c: char| c.is_whitespace() || c == '/');
        if rest.is_empty() {
            break;
        }
        let end = rest
            .find(|c: char| c == '=' || c.is_whitespace() || c == '/')
            .unwrap_or(rest.len());
        let name = rest[..end].to_owned();
        rest = rest[end..].trim_start();
        let Some(after) = rest.strip_prefix('=') else {
            found.push(Attribute { name, value: None });
            continue;
        };
        let after = after.trim_start();
        let (value, tail) = match after.chars().next() {
            Some(quote @ ('"' | '\'')) => match after[1..].find(quote) {
                Some(end) => (&after[1..1 + end], &after[end + 2..]),
                None => (&after[1..], ""),
            },
            _ => {
                let end = after.find(char::is_whitespace).unwrap_or(after.len());
                (&after[..end], &after[end..])
            }
        };
        found.push(Attribute {
            name,
            value: Some(value.replace('"', "&quot;")),
        });
        rest = tail;
    }
    (found, closes)
}

/// Where the tag that starts at `from` (a `<`) ends: after its `>`, with
/// quotes honored. `None` for a tag that never closes.
fn tag_end(html: &str, from: usize) -> Option<usize> {
    let mut quote: Option<u8> = None;
    for (i, &b) in html.as_bytes().iter().enumerate().skip(from + 1) {
        match (quote, b) {
            (Some(q), b) if b == q => quote = None,
            (Some(_), _) => {}
            (None, b'"' | b'\'') => quote = Some(b),
            (None, b'>') => return Some(i + 1),
            _ => {}
        }
    }
    None
}

/// The tag `name` with its attributes, kept to what a page may carry.
fn clean_tag(name: &str, text: &str) -> Option<String> {
    let (attrs, closes) = attributes(text);
    let mut out = format!("<{name}");
    let mut dropped_source = false;
    let mut alt = None;
    for attr in attrs {
        if attr.name.is_empty()
            || !attr
                .name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '_' | '-' | '.'))
        {
            continue;
        }
        let value = attr.value.as_deref();
        let decoded = value.map(unescape);
        let keep = match attr.name.to_ascii_lowercase().as_str() {
            name if name.starts_with("on") => false,
            name if BANNED_ATTRIBUTES.contains(&name) => false,
            "href" | "xlink:href" => match (&decoded, name) {
                (Some(url), "a") => is_allowed_link(url),
                (Some(url), _) => url.starts_with('#') || is_allowed_source(url),
                (None, _) => false,
            },
            "src" => {
                let ok = decoded.as_deref().is_some_and(is_allowed_source);
                dropped_source |= !ok;
                ok
            }
            "style" => decoded.as_deref().is_none_or(css_is_inert),
            "alt" => {
                alt = value.map(str::to_owned);
                true
            }
            _ => true,
        };
        if keep {
            match value {
                Some(value) => {
                    let _ = write!(out, " {}=\"{value}\"", attr.name);
                }
                None => {
                    let _ = write!(out, " {}", attr.name);
                }
            }
        }
    }
    if name.eq_ignore_ascii_case("img") && dropped_source {
        // An image that would be fetched shows its words instead.
        return Some(format!(
            "<span class=\"kanso-missing\">{}</span>",
            alt.unwrap_or_default()
        ));
    }
    out.push_str(if closes { " />" } else { ">" });
    Some(out)
}

/// The page's HTML with what could act or fetch taken out: scripts, frames,
/// forms, `<meta>`, `<link>`, `<base>` and their kin; event handlers
/// (`on…=`); every `href` that [`is_allowed_link`] doesn't allow; every `src`
/// but an inline image; styles that fetch. Everything else is kept as it is.
/// Pure.
pub fn sanitize(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut at = 0;
    while at < html.len() {
        let Some(lt) = html[at..].find('<').map(|i| i + at) else {
            out.push_str(&html[at..]);
            break;
        };
        out.push_str(&html[at..lt]);
        let rest = &html[lt..];
        if rest.starts_with("<!--") {
            at = rest.find("-->").map_or(html.len(), |end| lt + end + 3);
            continue;
        }
        if rest.starts_with("<![CDATA[") {
            let end = rest.find("]]>").map_or(html.len(), |end| lt + end + 3);
            // Character data is not markup; only a plain style's text could
            // be CSS, and that is cleaned with the style.
            out.push_str(&html[lt..end]);
            at = end;
            continue;
        }
        let next = rest.as_bytes().get(1).copied().unwrap_or(0);
        if !(next.is_ascii_alphabetic() || next == b'/') {
            // Not a tag: a bare `<` is text.
            out.push_str("&lt;");
            at = lt + 1;
            continue;
        }
        let Some(end) = tag_end(html, lt) else {
            out.push_str("&lt;");
            at = lt + 1;
            continue;
        };
        let inside = &html[lt + 1..end - 1];
        let closing = inside.starts_with('/');
        let body = inside.trim_start_matches('/');
        let name_len = body
            .find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, ':' | '_' | '-')))
            .unwrap_or(body.len());
        if name_len == 0 || body[name_len..].starts_with(|c: char| !(c.is_whitespace() || c == '/'))
        {
            // Not a tag's name: the `<` is text.
            out.push_str("&lt;");
            at = lt + 1;
            continue;
        }
        let name = &body[..name_len];
        let lowered = name.to_ascii_lowercase();
        let local = lowered.rsplit(':').next().unwrap_or(&lowered);
        at = end;
        if BANNED.contains(&local) {
            if !closing && BANNED_WITH_CONTENT.contains(&local) {
                let close = format!("</{local}");
                at = html[end..]
                    .to_ascii_lowercase()
                    .find(&close)
                    .and_then(|i| tag_end(html, end + i))
                    .unwrap_or(html.len());
            }
            continue;
        }
        if closing {
            let _ = write!(out, "</{name}>");
            continue;
        }
        if lowered == "style" {
            let close = html[end..].to_ascii_lowercase().find("</style");
            let stop = close.map_or(html.len(), |i| end + i);
            out.push_str("<style>");
            if css_is_inert(&html[end..stop]) {
                out.push_str(&html[end..stop]);
            }
            out.push_str("</style>");
            at = close
                .and_then(|i| tag_end(html, end + i))
                .unwrap_or(html.len());
            continue;
        }
        if let Some(tag) = clean_tag(name, &body[name_len..]) {
            out.push_str(&tag);
        }
    }
    out
}

// MARK: Style

/// The danger color of a chip: a warm red that reads on the page.
fn danger(dark: bool) -> Rgb {
    if dark { Rgb(0xF2766B) } else { Rgb(0xB3261E) }
}

/// The stylesheet's rules for the page in the app: task boxes as links with
/// a tap target of 45 points, due chips and query tables, in the page's
/// colors.
pub(super) fn stylesheet(p: &Palette, check: &str) -> String {
    let c = |rgb: Rgb| rgb.hex();
    let red = danger(p.dark);
    let tint = |rgb: Rgb| c(rgb.over(p.background, 0.16));
    let table_head = p.table_header_background.map_or_else(|| c(p.surface), c);
    format!(
        r#"header{{display:none}}
body{{max-width:none;margin:0;padding:0 0 1rem}}
ul:has(>li>a.kanso-task){{padding-left:0;list-style:none}}
li:has(>a.kanso-task),li:has(>p:first-child>a.kanso-task){{list-style:none}}
a.kanso-task{{display:inline-block;vertical-align:-2px;padding:15px;margin:-15px -7px -15px -15px;text-decoration:none;-webkit-tap-highlight-color:transparent}}
.kanso-box{{display:block;width:15px;height:15px;border:1.5px solid {taskdone};border-radius:4px;background:transparent}}
a.kanso-task-done .kanso-box{{background:{link} {check} no-repeat center/12px 12px;border-color:{link}}}
li:has(>a.kanso-task-done),li:has(>p:first-child>a.kanso-task-done){{color:{taskdone}}}
a.kanso-due{{display:inline-block;font-size:.8667em;line-height:1.5;padding:0 .55em;border-radius:999px;text-decoration:none;color:{muted};background:{surface}}}
a.kanso-due-today{{color:{link};background:{todaybg}}}
a.kanso-due-overdue{{color:{red};background:{redbg}}}
a.kanso-due-done{{color:{muted};background:{donebg}}}
.kanso-query{{position:relative;margin:0 0 1em}}
.kanso-query table{{display:table;width:100%;table-layout:fixed;font-size:.8em;margin:0}}
.kanso-query th,.kanso-query td{{border:0;padding:5px 8px;text-align:left;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:{muted}}}
.kanso-query thead th{{background:none;font-weight:400;font-size:.92em;padding-bottom:4px;border-bottom:1px solid {border}}}
.kanso-query th:first-child,.kanso-query td:first-child{{width:34%}}
.kanso-query td:first-child a{{display:block;padding:8px 0;margin:-8px 0;text-decoration:none;color:{link};font-size:1.04em}}
.kanso-query-empty{{margin:0;padding:8px;font-size:.8em;color:{muted}}}
.kanso-query-badge{{position:absolute;top:7px;left:10px;z-index:1;padding:1px 7px;border-radius:5px;background:{tablehead};border:1px solid {border};font-size:.72em;color:{muted}}}
.kanso-failed{{position:relative}}
.kanso-failed pre{{padding-top:{codetop}px}}
.kanso-copy{{position:absolute;top:0;right:0;width:44px;height:44px;display:flex;align-items:center;justify-content:center;color:{muted};text-decoration:none;-webkit-tap-highlight-color:transparent}}
pre:has(>.kanso-copy) .kanso-lang{{right:44px}}
.kanso-empty-drawing{{display:flex;align-items:center;justify-content:center;max-width:380px;height:120px;border:1px dashed {border};border-radius:8px;background:{surface};font-size:.8667em;color:{muted}}}
.kanso-query-why{{margin:.4em 0 0;font-size:.8em;color:{muted}}}
"#,
        taskdone = c(p.task_done),
        link = c(p.accent),
        muted = c(p.muted),
        surface = c(p.surface),
        border = c(p.border),
        tablehead = table_head,
        red = c(red),
        redbg = tint(red),
        todaybg = tint(p.accent),
        donebg = c(p.surface.over(p.background, 0.6)),
        codetop = crate::typography::CODE_TOP,
    )
}

#[cfg(test)]
mod tests;
