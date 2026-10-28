//! The note's preview in the app (#48 step 7): the page the WebView shows,
//! what a tap on it means, and the writes a tap makes.
//!
//! The page is `kanso-render`'s in-app mode ([`html::AppMode`]): the same page
//! that is published, with the addresses the app understands kept on its
//! links and nothing else a page could do. This module prepares the note's
//! text for it (wiki-links, `#tags` and due dates written as links), fills
//! its query views and due chips, colors its code, and reads the addresses
//! back ([`Renderer::link_target`]) so the shell never parses one.
//!
//! A locked note is a valid target of a link (it opens to its lock); its
//! text is only read while it is open, and only for the page, the heading a
//! link names, a code block's copy, or a tick.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::OnceLock;

use kanso_core::dates::DateTime;
use kanso_core::links::{FILE_LINK_PREFIX, NOTE_LINK_PREFIX, normalize_title};
use kanso_core::query_view::{ViewNote, cell_text};
use kanso_core::tasks::{linked_file_ids, task_items};
use kanso_core::{DocType, now_ms};
use kanso_engine::highlight::{self, Token};
use kanso_engine::links::{self, WikiIndex};
use kanso_engine::tasks::{self as due_chips, Tone};
use kanso_engine::theme::Rgb;
use kanso_engine::theme::file::TokenStyle;
use kanso_render::html::{
    self, AppMode, CodeSpan, DueChip, DueTone, Highlighter, PageOptions, QueryResult, QueryRow,
    QueryTable, QueryViews, TaskSlot,
};
use kanso_render::{MermaidEngine, mermaid_engine, set_mermaid_engine};

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::library::Library;
use crate::model::{
    AttachmentFile, DiagramRenderer, NewNote, PaletteDto, PreviewLink, PreviewOptions, Saved,
    SyntaxStyleRow, Ticked,
};
use crate::renderer::{Renderer, palette_of};

/// The scheme prefixes the page's own addresses carry.
const TASK: &str = html::app::TASK_LINK_PREFIX;
const COPY: &str = html::app::COPY_LINK_PREFIX;
const TAG: &str = html::app::TAG_LINK_PREFIX;
const DUE: &str = html::app::DUE_LINK_PREFIX;
const NEW: &str = html::app::NEW_NOTE_PREFIX;

impl Renderer {
    /// A note as a standalone page for the app's WebView, in `palette`'s
    /// colors and `options`' faces and size.
    ///
    /// - Its links keep the addresses [`Renderer::link_target`] reads: a
    ///   note (a wiki-link to a heading keeps it), a note to make, a tag, an
    ///   attachment, a task box, a code block's copy link, a due date, a
    ///   place in the page, and web and mail addresses. Anything else is
    ///   dropped, and the page has no scripts, handlers or fetches.
    /// - Math, diagrams and drawings are drawn, and one that fails shows its
    ///   code with the reason; images are inlined.
    /// - `query` fences are tables of the notes they list (never locked
    ///   ones); due dates are chips worded for the phone's day.
    /// - Code is colored when `options.code` has styles.
    ///
    /// [`KansoError::Locked`] while the note is locked and closed.
    pub fn preview_html_with(
        &self,
        id: &str,
        palette: PaletteDto,
        options: PreviewOptions,
    ) -> Result<String, KansoError> {
        guard(|| {
            let library = self.library();
            let (note, text) = library.readable_text(id)?.ok_or(KansoError::Locked)?;
            let store = &self.inner.store;
            let page_text = library.prepared(&text)?;
            let tasks: Vec<TaskSlot> = task_items(&text)
                .iter()
                .map(|item| TaskSlot {
                    line: item.line,
                    checked: item.checked,
                })
                .collect();
            let views = Views {
                library: &library,
                note_id: id,
                notes: OnceLock::new(),
            };
            let offset = i64::from(options.utc_offset_secs);
            let chip = move |url: &str| due_chip(url, now_ms() + offset * 1000);
            let colors = CodeColors::new(&options.code);
            let mode = AppMode {
                tasks: &tasks,
                query: Some(&views),
                due: Some(&chip),
            };
            let attachments = |file: &str| {
                store
                    .attachment(file)
                    .ok()
                    .flatten()
                    .map(|(content_type, blob)| (content_type, blob.data))
            };
            Ok(html::page_with(
                &note.title,
                &page_text,
                &palette_of(palette),
                &attachments,
                &PageOptions {
                    font: options.font.as_deref(),
                    mono: options.mono.as_deref(),
                    size: options.size,
                    highlight: colors.as_ref().map(|colors| colors as &dyn Highlighter),
                    app: Some(&mode),
                },
            ))
        })
    }

    /// What the address `url`, tapped in the page of note `note_id`, is.
    /// Anything the page doesn't emit and anything unsafe is
    /// [`PreviewLink::Nothing`]; the shell opens a [`PreviewLink::Web`]
    /// address with the system and nothing else.
    pub fn link_target(&self, note_id: &str, url: &str) -> Result<PreviewLink, KansoError> {
        guard(|| {
            let library = self.library();
            library.stored(note_id)?;
            if let Some(frag) = url.strip_prefix('#') {
                let id = links::percent_decode(frag).unwrap_or_else(|| frag.to_owned());
                return Ok(if id.is_empty() {
                    PreviewLink::Nothing
                } else {
                    PreviewLink::Heading { id }
                });
            }
            if let Some(index) = url.strip_prefix(TASK) {
                return Ok(index_target(index, |index| PreviewLink::Task { index }));
            }
            if let Some(index) = url.strip_prefix(COPY) {
                return Ok(index_target(index, |index| PreviewLink::CopyCode { index }));
            }
            if url.starts_with(DUE) {
                return Ok(PreviewLink::Due);
            }
            if url.starts_with(TAG) {
                return Ok(links::tags::tag_from_url(url)
                    .map_or(PreviewLink::Nothing, |name| PreviewLink::Tag { name }));
            }
            if url.starts_with(NEW) {
                return Ok(links::new_note_title(url)
                    .map_or(PreviewLink::Nothing, |title| PreviewLink::NewNote { title }));
            }
            if url.starts_with(NOTE_LINK_PREFIX) {
                return library.note_target(url);
            }
            if url.starts_with(FILE_LINK_PREFIX) {
                return library.file_target(url);
            }
            Ok(if is_web(url) {
                PreviewLink::Web {
                    url: url.to_owned(),
                }
            } else {
                PreviewLink::Nothing
            })
        })
    }

    /// The text of the `index`-th code block of note `note_id` (0-based, as
    /// the page numbers its copy links): what a copy link puts on the
    /// clipboard. [`KansoError::NotFound`] for a block the note doesn't have,
    /// [`KansoError::Locked`] while the note is closed.
    pub fn code_block(&self, note_id: &str, index: u32) -> Result<String, KansoError> {
        guard(|| {
            let library = self.library();
            let (_, text) = library.readable_text(note_id)?.ok_or(KansoError::Locked)?;
            let page_text = library.prepared(&text)?;
            html::app::code_blocks(&page_text)
                .into_iter()
                .nth(index as usize)
                .ok_or(KansoError::NotFound)
        })
    }

    /// Chooses what draws Mermaid diagrams from now on (the desktop's
    /// Settings › Appearance › Diagrams). It applies to the whole process.
    pub fn set_diagram_renderer(renderer: DiagramRenderer) -> Result<(), KansoError> {
        guard(|| {
            set_mermaid_engine(match renderer {
                DiagramRenderer::Pele => MermaidEngine::Pele,
                DiagramRenderer::Kanso => MermaidEngine::MermaidSvg,
            });
            Ok(())
        })
    }

    /// What draws Mermaid diagrams now: [`DiagramRenderer::Pele`] until
    /// [`Renderer::set_diagram_renderer`] says otherwise.
    pub fn diagram_renderer() -> Result<DiagramRenderer, KansoError> {
        guard(|| {
            Ok(match mermaid_engine() {
                MermaidEngine::Pele => DiagramRenderer::Pele,
                MermaidEngine::MermaidSvg => DiagramRenderer::Kanso,
            })
        })
    }

    /// The library this renderer draws for.
    fn library(&self) -> Library {
        Library {
            inner: self.inner.clone(),
        }
    }
}

impl Library {
    /// Ticks (or clears) the `index`-th task box of note `note_id` as the
    /// page numbers them (a [`PreviewLink::Task`]), when the note is still at
    /// `base_rev`, the revision the page was drawn from; otherwise
    /// ([`KansoError::Conflict`]) nothing is written. The result is
    /// [`Library::set_task`]'s, so its ticket takes the tick back;
    /// [`Ticked::Missing`] when the note has no such task any more.
    pub fn toggle_task_at(
        &self,
        note_id: &str,
        index: u32,
        base_rev: &str,
    ) -> Result<Ticked, KansoError> {
        guard(|| {
            let (note, text) = self.readable_text(note_id)?.ok_or(KansoError::Locked)?;
            if note.rev.as_deref() != Some(base_rev) {
                return Err(KansoError::Conflict {
                    current_rev: note.rev.unwrap_or_default(),
                });
            }
            let items = task_items(&text);
            let Some(item) = items.get(index as usize) else {
                return Ok(Ticked::Missing);
            };
            self.set_task(note_id, item.line, &item.text, !item.checked)
        })
    }

    /// The note a tapped unresolved wiki-link (a [`PreviewLink::NewNote`])
    /// asks for: a note titled `title` in the notebook of `from_note` (the
    /// default notebook when that one is the trash), empty. When a note with
    /// that title exists by now (made since the page was drawn), that one is
    /// returned and nothing is made. In a locked notebook the lock has to be
    /// open ([`KansoError::Locked`]).
    pub fn create_linked_note(&self, title: &str, from_note: &str) -> Result<Saved, KansoError> {
        guard(|| {
            let title = title.trim();
            if normalize_title(title).is_empty() {
                return Err(KansoError::invalid("The link has no title."));
            }
            let from = self.stored(from_note)?;
            if let Some((id, _)) = self.inner.store.resolve_title(title)? {
                let rev = self.stored(&id)?.rev.unwrap_or_default();
                return Ok(Saved { id, rev });
            }
            let notebook_id = (!from.is_trashed()).then_some(from.book_id);
            self.create(NewNote {
                notebook_id,
                title: title.to_owned(),
                body: String::new(),
                place: None,
                preferred_notebook_id: None,
                tags: Vec::new(),
                pinned: false,
            })
        })
    }

    /// An attachment's name, type and bytes, for the viewer or the share
    /// sheet. Only a file that a note readable now links is returned: the
    /// store can't say which closed locked notes link a file, so a file only
    /// they link is [`KansoError::NotFound`] until the lock opens.
    pub fn attachment(&self, id: &str) -> Result<AttachmentFile, KansoError> {
        guard(|| {
            if DocType::of_id(id) != Some(DocType::File) {
                return Err(KansoError::NotFound);
            }
            let store = &self.inner.store;
            let meta = store.file_meta(id)?.ok_or(KansoError::NotFound)?;
            if !self.linked_from_readable_note(id)? {
                return Err(KansoError::NotFound);
            }
            let (content_type, blob) = store.attachment(id)?.ok_or(KansoError::NotFound)?;
            Ok(AttachmentFile {
                id: id.to_owned(),
                name: meta.name,
                content_type,
                bytes: blob.data,
            })
        })
    }

    /// Whether a note whose text can be read now links file `id`.
    fn linked_from_readable_note(&self, id: &str) -> Result<bool, KansoError> {
        let store = &self.inner.store;
        let wanted = id.to_owned();
        if store
            .query_view_notes()?
            .iter()
            .any(|row| !row.body.is_empty() && linked_file_ids(&row.body).contains(&wanted))
        {
            return Ok(true);
        }
        for mut note in store.locked_notes()? {
            if !self.reveal(&mut note).is_closed() && linked_file_ids(&note.body).contains(&wanted)
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// The note's Markdown as the page is made from it: wiki-links, `#tags`
    /// and due dates written as the links the page understands.
    fn prepared(&self, text: &str) -> Result<String, KansoError> {
        let index = WikiIndex::new(self.inner.store.note_titles()?);
        let linked = links::rewrite_for_app(text, &index);
        let tagged = links::tags::rewrite_for_app(&linked);
        Ok(match due_chips::rewrite_for_preview(&tagged) {
            Cow::Borrowed(_) => tagged.into_owned(),
            Cow::Owned(dated) => dated,
        })
    }

    /// A note address (`inkdrop://note/<id>[#Heading]`) as a target.
    fn note_target(&self, url: &str) -> Result<PreviewLink, KansoError> {
        let Some(id) = links::linked_note(url) else {
            return Ok(PreviewLink::Nothing);
        };
        let Some(note) = self
            .stored(&id)
            .ok()
            .filter(|note| !note.is_trashed())
            .map(|note| note.id)
        else {
            return Ok(PreviewLink::Nothing);
        };
        let heading = url
            .split_once('#')
            .and_then(|(_, heading)| links::percent_decode(heading))
            .map(|heading| heading.trim().to_owned())
            .filter(|heading| !heading.is_empty());
        let anchor = match (&heading, self.readable_text(&note)?) {
            (Some(heading), Some((_, text))) => {
                let wanted = normalize_title(heading);
                html::headings(&self.prepared(&text)?)
                    .into_iter()
                    .find(|found| normalize_title(&found.text) == wanted)
                    .map(|found| found.id)
            }
            _ => None,
        };
        Ok(PreviewLink::Note {
            id: note,
            heading,
            anchor,
        })
    }

    /// An attachment address (`inkdrop://file:<id>`) as a target.
    fn file_target(&self, url: &str) -> Result<PreviewLink, KansoError> {
        let rest = &url[FILE_LINK_PREFIX.len()..];
        let rest = rest.split(['?', '#', '/', ')', ' ']).next().unwrap_or("");
        if rest.is_empty() {
            return Ok(PreviewLink::Nothing);
        }
        let id = format!("file:{rest}");
        Ok(match self.inner.store.file_meta(&id) {
            Ok(Some(meta)) => PreviewLink::Attachment {
                id,
                name: meta.name,
                content_type: meta.content_type,
                size: meta.content_length,
            },
            _ => PreviewLink::Nothing,
        })
    }
}

/// `make(n)` for a whole number, [`PreviewLink::Nothing`] for anything else.
fn index_target(text: &str, make: impl FnOnce(u32) -> PreviewLink) -> PreviewLink {
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return PreviewLink::Nothing;
    }
    text.parse().map_or(PreviewLink::Nothing, make)
}

/// Whether `url` is a web or mail address the system may open, as written
/// (no whitespace or control characters, scheme read in any case).
fn is_web(url: &str) -> bool {
    if url.chars().any(|c| c.is_whitespace() || c.is_control()) || !html::is_allowed_link(url) {
        return false;
    }
    let lower = url.to_ascii_lowercase();
    ["http://", "https://", "mailto:"]
        .iter()
        .any(|scheme| lower.starts_with(scheme))
}

/// A `kanso-due:` address as the chip the page shows at local time
/// `local_ms`.
fn due_chip(url: &str, local_ms: i64) -> Option<DueChip> {
    let (due, checked) = due_chips::parse_due_url(url)?;
    let now = DateTime::from_local_ms(local_ms);
    Some(DueChip {
        label: due_chips::chip_label(due, checked, now),
        title: due_chips::long_label(due),
        tone: match due_chips::tone(due, checked, now) {
            Tone::Plain => DueTone::Plain,
            Tone::Today => DueTone::Today,
            Tone::Overdue => DueTone::Overdue,
            Tone::Done => DueTone::Done,
        },
    })
}

// MARK: Query views

/// Runs the page's `query` fences against the library, reading the notes
/// once for the page.
struct Views<'a> {
    library: &'a Library,
    note_id: &'a str,
    notes: OnceLock<Result<Vec<ViewNote>, String>>,
}

impl QueryViews for Views<'_> {
    fn run(&self, source: &str) -> QueryResult {
        let spec = match kanso_core::query_view::parse(source) {
            Ok(spec) => spec,
            Err(err) => {
                return QueryResult::Invalid {
                    message: err.message,
                };
            }
        };
        let notes = self.notes.get_or_init(|| {
            self.library
                .inner
                .store
                .query_view_notes()
                .map(|rows| rows.into_iter().map(ViewNote::from).collect())
                .map_err(|err| err.to_string())
        });
        let notes = match notes {
            Ok(notes) => notes,
            Err(message) => {
                return QueryResult::Invalid {
                    message: format!("The notes couldn't be read: {message}"),
                };
            }
        };
        let rows = kanso_core::query_view::evaluate(&spec, notes, Some(self.note_id));
        QueryResult::Table(QueryTable {
            columns: spec.labels(),
            rows: rows
                .into_iter()
                .map(|row| QueryRow {
                    id: row.id,
                    title: row.title,
                    cells: row
                        .cells
                        .iter()
                        .map(|cell| cell.as_ref().map(cell_text).unwrap_or_default())
                        .collect(),
                })
                .collect(),
        })
    }
}

// MARK: Code colors

/// Colors fenced code with the theme's syntax styles, by the language the
/// fence names: the engine's tree-sitter highlighter, as the editor uses.
struct CodeColors {
    styles: BTreeMap<&'static str, TokenStyle>,
}

impl CodeColors {
    /// The colors of `rows`, or `None` when there are none to color with.
    fn new(rows: &[SyntaxStyleRow]) -> Option<CodeColors> {
        let styles: BTreeMap<&'static str, TokenStyle> = rows
            .iter()
            .filter_map(|row| {
                let token = Token::from_key(&row.token)?;
                Some((
                    token.key(),
                    TokenStyle {
                        color: row.color.map(Rgb),
                        italic: row.italic,
                        weight: row.weight,
                    },
                ))
            })
            .filter(|(_, style)| !style.is_empty())
            .collect();
        (!styles.is_empty()).then_some(CodeColors { styles })
    }
}

impl Highlighter for CodeColors {
    fn highlight(&self, lang: &str, code: &str) -> Option<Vec<CodeSpan>> {
        let grammar = highlight::grammar(lang)?;
        // The code as a fence of its own, so the highlighter injects the
        // grammar the way it does in a note; the fence is longer than any
        // run of backticks the code holds.
        let longest = code
            .split(|c| c != '`')
            .map(str::len)
            .max()
            .unwrap_or_default();
        let fence = "`".repeat(longest.max(2) + 1);
        let head = format!("{fence}{grammar}\n");
        let wrapped = format!("{head}{code}\n{fence}\n");
        let from = head.len();
        let to = from + code.len();
        let spans = highlight::spans(&wrapped, from..to)
            .into_iter()
            .filter_map(|span| {
                // The Markdown's own marks around the injected code are not
                // the code's colors.
                let tokens: Vec<Token> = span
                    .tokens
                    .into_iter()
                    .filter(|token| !matches!(token, Token::CodeBlock | Token::Embedded))
                    .collect();
                let style = kanso_engine::theme::style(&tokens, &self.styles);
                let bold = style.weight.is_some_and(|weight| weight >= 600);
                let italic = style.italic == Some(true);
                let color = style.color.map(|color| color.0);
                if color.is_none() && !bold && !italic {
                    return None;
                }
                let start = span.range.start.max(from) - from;
                let end = span.range.end.min(to) - from;
                (start < end).then_some(CodeSpan {
                    start,
                    end,
                    color,
                    bold,
                    italic,
                })
            })
            .collect();
        Some(spans)
    }
}
