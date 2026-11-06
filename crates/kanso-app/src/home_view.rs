//! A notebook's homepage (issue #201), in the editor's place: what the
//! notebook is for, its key notes, recent activity, open tasks, the people
//! and projects in it, decisions, a small map of how its notes link and the
//! notebooks inside it.
//!
//! The homepage is a generated view, not a note. Computed sections are built
//! from the derived tables on open; the two AI sections (About and
//! Decisions) are written by the person's model with a receipt on every
//! line (`kanso_engine::pages`), cached on the notebook and marked out of
//! date when the notes they came from change. The person's own words live in
//! a real note, the Home note, shown at the top and edited in place; Kanso
//! never writes into it except to append a section the person copied.

use gpui_component::scroll::ScrollableElement as _;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::{Icon, Sizable as _, StyledExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::now_ms;
use kanso_engine::ask::{Receipt, relocate};
use kanso_engine::pages::notebook::{NotebookPage, title_of};
use kanso_engine::pages::notebook_actions as actions;
use kanso_engine::pages::notebook_model::{ASK_BUTTON, FIRST_VISIT_BUTTON, MapView};
use kanso_engine::pages::{self, Item, Section, SectionKind, SectionState};
use kanso_store::{CheckOff, Origin, Store};

use crate::ai_feature::{self, Maker};
use crate::home_data::{self, Loaded};
use crate::library::{Library, LibraryEvent};
use crate::page_chrome::{ai_error_card, ai_trouble};
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

/// How long changes settle before the page reads the notebook again.
const RELOAD_DELAY: Duration = Duration::from_millis(500);
/// The pane width above which the sections sit in two columns.
pub const TWO_COLUMNS: f32 = 900.;
/// The reading column's widest, one column and two.
const NARROW_COLUMN: f32 = 720.;
const WIDE_COLUMN: f32 = 1080.;

/// What the page asks of the window around it.
pub enum HomeEvent {
    /// Open a note and mark these lines (0-based, inclusive).
    OpenLines { id: String, first: u32 },
    /// Open a note.
    OpenNote(String),
    /// Edit the Home note in the editor, then return here.
    EditHome(String),
    /// Open a notebook (and its homepage).
    OpenBook(String),
    /// Ask scoped to this notebook.
    Ask(String),
    /// Open the graph narrowed to this notebook.
    Graph(String),
    /// Something worth telling the person.
    Notice(String),
}

/// What a section's menu does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionAction {
    Refresh,
    Keep,
    CopyIntoHome,
    Hide,
}

/// The menu entries of a section, in order: Refresh only for an AI section
/// while AI is on; Keep only for an AI section; Copy only with lines. Pure,
/// so the choices can be tested.
pub fn section_actions(section: &Section, ai_on: bool) -> Vec<SectionAction> {
    let ai = section.kind == SectionKind::Ai;
    let mut out = Vec::new();
    if ai && ai_on {
        out.push(SectionAction::Refresh);
    }
    if ai {
        out.push(SectionAction::Keep);
    }
    if !section.items.is_empty() && section.key != "home_note" {
        out.push(SectionAction::CopyIntoHome);
    }
    if section.key != "home_note" {
        out.push(SectionAction::Hide);
    }
    out
}

/// The words for a section action.
pub fn action_label(action: SectionAction, pinned: bool) -> &'static str {
    match action {
        SectionAction::Refresh => "Refresh",
        SectionAction::Keep if pinned => "Stop keeping this version",
        SectionAction::Keep => "Keep this version",
        SectionAction::CopyIntoHome => "Copy into Home note",
        SectionAction::Hide => "Hide section",
    }
}

/// Which sections take the whole width in two columns.
pub fn full_width(key: &str) -> bool {
    matches!(key, "home_note" | "about")
}

/// The sections as rows of the page: a full-width section alone, the rest
/// two to a row in order in two columns, one to a row in one. Pure.
pub fn rows_of(keys: &[String], two_columns: bool) -> Vec<Vec<usize>> {
    let mut rows: Vec<Vec<usize>> = Vec::new();
    let mut pending: Option<usize> = None;
    for (ix, key) in keys.iter().enumerate() {
        if !two_columns || full_width(key) {
            if let Some(p) = pending.take() {
                rows.push(vec![p]);
            }
            rows.push(vec![ix]);
        } else if let Some(p) = pending.take() {
            rows.push(vec![p, ix]);
        } else {
            pending = Some(ix);
        }
    }
    if let Some(p) = pending {
        rows.push(vec![p]);
    }
    rows
}

pub struct NotebookHomeView {
    store: Arc<Store>,
    library: Entity<Library>,
    book: Option<String>,
    loaded: Option<Loaded>,
    map: Option<MapView>,
    /// Sections being written now.
    working: HashSet<String>,
    /// Sections that failed to write, with the model's own words.
    trouble: HashMap<String, String>,
    /// Sections folded to their title.
    folded: HashSet<String>,
    /// Computed sections shown in full.
    shown_all: HashSet<String>,
    pane_width: f32,
    stale: bool,
    reload_task: Option<Task<()>>,
    map_task: Option<Task<()>>,
    write_task: Option<Task<()>>,
    scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<HomeEvent> for NotebookHomeView {}

impl NotebookHomeView {
    pub fn new(library: Entity<Library>, cx: &mut Context<Self>) -> Self {
        let store = library.read(cx).store.clone();
        let subscriptions = vec![
            cx.subscribe(&library, |this, _, event, cx| {
                if let LibraryEvent::Changed(change) = event
                    && this.book.is_some()
                    && (change.id.starts_with("note:") || change.id.starts_with("book:"))
                {
                    this.stale = true;
                    this.reload_soon(cx);
                }
            }),
            // A locked notebook's homepage is built from the notes this
            // session opened: opening the notebook builds it, and locking
            // the notes again drops it at once.
            cx.observe_global::<crate::locked::LockKeys>(|this, cx| {
                if let Some(book) = this.book.clone()
                    && this.library.read(cx).is_book_locked(&book)
                {
                    this.reload(false, cx);
                }
            }),
        ];
        NotebookHomeView {
            store,
            library,
            book: None,
            loaded: None,
            map: None,
            working: HashSet::new(),
            trouble: HashMap::new(),
            folded: HashSet::new(),
            shown_all: HashSet::new(),
            pane_width: 1000.,
            stale: false,
            reload_task: None,
            map_task: None,
            write_task: None,
            scroll: ScrollHandle::new(),
            _subscriptions: subscriptions,
        }
    }

    /// The notebook whose homepage shows.
    pub fn book(&self) -> Option<&str> {
        self.book.as_deref()
    }

    /// Whether the page has loaded.
    #[cfg(test)]
    pub fn is_loaded(&self) -> bool {
        self.loaded.is_some()
    }

    /// The sections drawn now, in order, for tests and the parity check.
    #[cfg(test)]
    pub fn section_keys(&self) -> Vec<String> {
        self.loaded
            .iter()
            .flat_map(|l| &l.view.page.sections)
            .map(|s| s.key.clone())
            .collect()
    }

    /// The titles drawn now, in order.
    #[cfg(test)]
    pub fn section_titles(&self) -> Vec<String> {
        self.loaded
            .iter()
            .flat_map(|l| &l.view.page.sections)
            .map(|s| s.title.clone())
            .collect()
    }

    /// The muted line at the end, if any.
    #[cfg(test)]
    pub fn footer(&self) -> Option<String> {
        self.loaded.as_ref().and_then(|l| l.view.footer.clone())
    }

    pub fn set_pane_width(&mut self, width: f32, cx: &mut Context<Self>) {
        if (self.pane_width - width).abs() > 0.5 {
            self.pane_width = width;
            cx.notify();
        }
    }

    pub fn set_book(&mut self, id: Option<String>, cx: &mut Context<Self>) {
        if self.book == id {
            return;
        }
        self.book = id;
        self.loaded = None;
        self.map = None;
        self.working.clear();
        self.trouble.clear();
        self.shown_all.clear();
        self.write_task = None;
        self.map_task = None;
        self.reload(true, cx);
        cx.notify();
    }

    fn reload_soon(&mut self, cx: &mut Context<Self>) {
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(RELOAD_DELAY).await;
            this.update(cx, |this, cx| this.reload(false, cx)).ok();
        }));
    }

    /// The notes a locked notebook has open this session, revealed. `None`
    /// when the notebook is not locked, or is locked and not open.
    fn opened_notes(
        &self,
        id: &str,
        cx: &mut Context<Self>,
    ) -> Option<Option<Vec<kanso_engine::pages::notebook_model::OpenedNote>>> {
        if !self.library.read(cx).is_book_locked(id) {
            return Some(None);
        }
        if !crate::locked::LockKeys::is_unlocked(cx) {
            return None;
        }
        let books = self.store.book_subtree(id, true).ok()?;
        let notes = self.store.notes_in_books(&books).ok()?;
        let mut out = Vec::new();
        for mut note in notes {
            if note.is_trashed() {
                continue;
            }
            match crate::locked::reveal(&mut note, cx) {
                crate::locked::LockView::Closed(_) => continue,
                _ => out.push(kanso_engine::pages::notebook_model::OpenedNote {
                    id: note.id,
                    title: note.title,
                    body: note.body,
                    book_id: note.book_id,
                    created_at: note.created_at,
                    updated_at: note.updated_at,
                    pinned: note.pinned,
                }),
            }
        }
        if out.is_empty() {
            None
        } else {
            Some(Some(out))
        }
    }

    /// Reads the page off the main thread. `opened` is a page just shown:
    /// sections its refresh setting wants written are written then.
    fn reload(&mut self, opened: bool, cx: &mut Context<Self>) {
        self.stale = false;
        let Some(id) = self.book.clone() else {
            self.loaded = None;
            return;
        };
        let Some(notes) = self.opened_notes(&id, cx) else {
            // Locked and closed: the lock screen asks; nothing is drawn.
            self.loaded = None;
            self.map = None;
            cx.notify();
            return;
        };
        let store = self.store.clone();
        let config = cx
            .try_global::<AppState>()
            .map(|state| state.config.clone())
            .unwrap_or_default();
        let shown: Vec<String> = self.shown_all.iter().cloned().collect();
        let task = cx.background_executor().spawn({
            let (id, config) = (id.clone(), config.clone());
            async move {
                let all: Vec<&str> = shown.iter().map(String::as_str).collect();
                home_data::load(&store, &id, &config, &all, notes)
            }
        });
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            let loaded = task.await;
            this.update(cx, |this, cx| {
                if this.book.as_deref() != Some(id.as_str()) {
                    return;
                }
                match loaded {
                    Ok(loaded) => {
                        let due = loaded
                            .as_ref()
                            .filter(|_| opened)
                            .map(|l| l.due.clone())
                            .unwrap_or_default();
                        let map_wanted = loaded.as_ref().is_some_and(|l| {
                            !l.locked_open && l.view.page.sections.iter().any(|s| s.key == "map")
                        });
                        this.loaded = loaded;
                        if map_wanted {
                            this.load_map(cx);
                        } else {
                            this.map = None;
                        }
                        if !due.is_empty() {
                            this.write(due, cx);
                        }
                    }
                    Err(err) => eprintln!("kanso: notebook home: {err}"),
                }
                cx.notify();
            })
            .ok();
        }));
    }

    /// Lays the map out off the main thread; the sections are already on
    /// screen.
    fn load_map(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.book.clone() else { return };
        let store = self.store.clone();
        let task = cx.background_executor().spawn({
            let id = id.clone();
            async move { home_data::map(&store, &id) }
        });
        self.map_task = Some(cx.spawn(async move |this, cx| {
            let map = task.await;
            this.update(cx, |this, cx| {
                if this.book.as_deref() == Some(id.as_str()) {
                    this.map = map.ok().flatten();
                    cx.notify();
                }
            })
            .ok();
        }));
    }

    /// Writes `keys` with the model, one at a time, so a failure names its
    /// own section.
    fn write(&mut self, keys: Vec<String>, cx: &mut Context<Self>) {
        let Some(id) = self.book.clone() else {
            return;
        };
        let maker: Maker = ai_feature::maker(cx);
        let model = cx.global::<AppState>().config.ai.model.clone();
        let store = self.store.clone();
        for key in &keys {
            self.working.insert(key.clone());
            self.trouble.remove(key);
        }
        self.write_task = Some(cx.spawn(async move |this, cx| {
            for key in keys {
                let result = cx
                    .background_executor()
                    .spawn({
                        let (store, id, key, model, maker) = (
                            store.clone(),
                            id.clone(),
                            key.clone(),
                            model.clone(),
                            maker.clone(),
                        );
                        async move { home_data::refresh(&store, &id, Some(&key), &model, &maker) }
                    })
                    .await;
                let stop = this
                    .update(cx, |this, cx| {
                        if this.book.as_deref() != Some(id.as_str()) {
                            return true;
                        }
                        this.working.remove(&key);
                        if let Err(reason) = result {
                            this.trouble.insert(key.clone(), reason);
                        }
                        this.reload(false, cx);
                        cx.notify();
                        false
                    })
                    .unwrap_or(true);
                if stop {
                    return;
                }
            }
        }));
    }

    /// The AI sections that are not kept, for Refresh all.
    fn refreshable(&self) -> Vec<String> {
        self.loaded
            .iter()
            .flat_map(|l| &l.view.page.sections)
            .filter(|s| {
                s.kind == SectionKind::Ai && !matches!(s.state, SectionState::Pinned { .. })
            })
            .map(|s| s.key.clone())
            .collect()
    }

    /// Whether any AI section was never written and AI could write it: the
    /// first visit with an API key shows a button instead of spending money.
    #[cfg(test)]
    pub fn first_visit(&self) -> bool {
        self.loaded.as_ref().is_some_and(|l| {
            l.view.ai_on
                && l.view
                    .page
                    .sections
                    .iter()
                    .any(|s| s.kind == SectionKind::Ai && s.state == SectionState::Unwritten)
        })
    }

    fn page(&self, cx: &App) -> Option<NotebookPage> {
        let id = self.book.as_deref()?;
        NotebookPage::load(&self.store, id, crate::clock::Clock::now_ms(cx))
            .ok()
            .flatten()
    }

    fn run(&mut self, key: &str, action: SectionAction, cx: &mut Context<Self>) {
        let Some(page) = self.page(cx) else { return };
        let outcome = match action {
            SectionAction::Refresh => {
                self.write(vec![key.to_owned()], cx);
                return;
            }
            SectionAction::Keep => {
                let pinned = self
                    .loaded
                    .iter()
                    .flat_map(|l| &l.view.page.sections)
                    .any(|s| s.key == key && matches!(s.state, SectionState::Pinned { .. }));
                pages::pin(&self.store, &page, key, !pinned, now_ms()).map(|_| ())
            }
            SectionAction::Hide => pages::hide(&self.store, &page, key, true, now_ms()).map(|_| ()),
            SectionAction::CopyIntoHome => {
                let section = self
                    .loaded
                    .iter()
                    .flat_map(|l| &l.view.page.sections)
                    .find(|s| s.key == key)
                    .cloned();
                if let Some(section) = section {
                    match actions::copy_into_home(
                        &self.store,
                        &page.id,
                        &page.name,
                        &section.title,
                        &pages::section_markdown(&section),
                        now_ms(),
                    ) {
                        Ok(_) => cx.emit(HomeEvent::Notice("Copied into the Home note.".into())),
                        Err(err) => cx.emit(HomeEvent::Notice(err.to_string())),
                    }
                }
                self.reload(false, cx);
                return;
            }
        };
        if let Err(err) = outcome {
            cx.emit(HomeEvent::Notice(format!(
                "Couldn't change the page: {err}"
            )));
        }
        self.reload(false, cx);
    }

    fn show_hidden(&mut self, cx: &mut Context<Self>) {
        if let Some(page) = self.page(cx) {
            let _ = pages::show_hidden(&self.store, &page, now_ms());
            self.reload(false, cx);
        }
    }

    fn not_right(&mut self, key: &str, index: usize, cx: &mut Context<Self>) {
        if let Some(page) = self.page(cx) {
            let _ = pages::not_right(&self.store, &page, key, index);
            self.reload(false, cx);
        }
    }

    fn show_all(&mut self, key: &str, cx: &mut Context<Self>) {
        self.shown_all.insert(key.to_owned());
        self.reload(false, cx);
    }

    /// Makes `note` the Home note.
    pub fn choose_home(&mut self, note: &str, cx: &mut Context<Self>) {
        let Some(book) = self.book.clone() else {
            return;
        };
        match actions::choose_home(&self.store, &book, note, now_ms()) {
            Ok(()) => self.reload(false, cx),
            Err(err) => cx.emit(HomeEvent::Notice(err.to_string())),
        }
    }

    /// Makes "About <Notebook>" the Home note and opens it to write in.
    pub fn new_home(&mut self, cx: &mut Context<Self>) {
        let Some(page) = self.page(cx) else { return };
        match actions::new_home(&self.store, &page.id, &page.name, now_ms()) {
            Ok(id) => cx.emit(HomeEvent::EditHome(id)),
            Err(err) => cx.emit(HomeEvent::Notice(err.to_string())),
        }
    }

    /// Stops the notebook having a Home note; the note stays.
    pub fn clear_home(&mut self, cx: &mut Context<Self>) {
        if let Some(book) = self.book.clone() {
            let _ = actions::clear_home(&self.store, &book, now_ms());
            self.reload(false, cx);
        }
    }

    /// Sets the order of the sections and which are hidden.
    pub fn customise(&mut self, order: Vec<String>, hidden: Vec<String>, cx: &mut Context<Self>) {
        let Some(page) = self.page(cx) else { return };
        if let Err(err) = actions::customise(&self.store, &page, &order, &hidden, now_ms()) {
            cx.emit(HomeEvent::Notice(err.to_string()));
        }
        self.reload(false, cx);
    }

    /// The sections in the order shown, hidden ones too, for Customise.
    pub fn layout_rows(&self) -> Vec<(String, String, bool)> {
        let Some(book) = self.book.as_deref() else {
            return Vec::new();
        };
        let cache = self.store.page_cache(book).unwrap_or_default();
        let mut keys: Vec<&str> = kanso_core::home::SECTION_KEYS.to_vec();
        if let Some(layout) = &cache.layout {
            keys.sort_by_key(|key| {
                layout.order.iter().position(|k| k == key).unwrap_or(
                    layout.order.len()
                        + kanso_core::home::SECTION_KEYS
                            .iter()
                            .position(|k| k == key)
                            .unwrap_or(0),
                )
            });
        }
        keys.into_iter()
            .map(|key| {
                (
                    key.to_owned(),
                    title_of(key).unwrap_or(key).to_owned(),
                    cache.is_hidden(key),
                )
            })
            .collect()
    }

    /// Ticks a task a line shows, in its own note.
    fn tick(&mut self, item: &Item, checked: bool, cx: &mut Context<Self>) {
        let Some(task) = &item.task else { return };
        if cx
            .try_global::<crate::services::Services>()
            .is_some_and(|services| services.writes_held)
        {
            return;
        }
        match self.store.set_task_checked(
            &task.note_id,
            task.line,
            &task.text,
            checked,
            crate::clock::Clock::now_ms(cx),
            Origin::Ui,
        ) {
            Ok(CheckOff::Missing) => cx.emit(HomeEvent::Notice(
                "That task changed in its note. The page is up to date now.".into(),
            )),
            Ok(_) => {}
            Err(err) => cx.emit(HomeEvent::Notice(format!(
                "Couldn't change the task: {err}"
            ))),
        }
        self.reload(false, cx);
    }

    /// Opens the lines a source points at. Lines that moved since the
    /// section was written are found again; lines that are gone open the
    /// note and say so.
    fn open_source(&mut self, receipt: &Receipt, cx: &mut Context<Self>) {
        let body = self
            .store
            .note(&receipt.note_id)
            .ok()
            .flatten()
            .map(|note| note.body);
        match body.and_then(|body| relocate(&body, receipt)) {
            Some((first, _)) => cx.emit(HomeEvent::OpenLines {
                id: receipt.note_id.clone(),
                first,
            }),
            None => {
                cx.emit(HomeEvent::Notice(
                    kanso_engine::ask::LINES_CHANGED.to_owned(),
                ));
                cx.emit(HomeEvent::OpenNote(receipt.note_id.clone()));
            }
        }
    }
}

// MARK: Drawing

impl NotebookHomeView {
    fn render_item(
        &self,
        section: &Section,
        index: usize,
        item: &Item,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = Colors::get(cx);
        let view = cx.entity().downgrade();
        let key = section.key.clone();
        let ai = section.kind == SectionKind::Ai;
        let meta: Vec<String> = [
            item.tag.clone().filter(|t| t != "locked"),
            item.meta.clone(),
        ]
        .into_iter()
        .flatten()
        .filter(|s| !s.is_empty())
        .collect();
        let locked = item.tag.as_deref() == Some("locked");
        let sources: Vec<Receipt> = item.sources.clone();
        let target = item.target.clone();
        let inside = section.key == "inside";
        let line = item.clone();
        let unique = section_index(&section.key) * 1000 + index;
        h_flex()
            .id(("home-item", unique))
            .items_start()
            .gap(px(8.))
            .py(px(3.))
            .when_some(item.task.clone(), |row, task| {
                let ticker = view.clone();
                let item = item.clone();
                row.child(
                    Checkbox::new(("home-task", unique))
                        .checked(task.checked)
                        .on_click(move |checked, _, cx| {
                            let item = item.clone();
                            let checked = *checked;
                            ticker
                                .update(cx, |this, cx| this.tick(&item, checked, cx))
                                .ok();
                        }),
                )
            })
            .when(locked, |row| {
                row.child(
                    Icon::new(Lucide::Lock)
                        .size(px(13.))
                        .mt(px(3.))
                        .text_color(colors.text_muted),
                )
            })
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .child({
                        let opener = view.clone();
                        let target = target.clone();
                        let first = sources.first().cloned();
                        div()
                            .id(("home-text", unique))
                            .text_size(px(13.5))
                            .line_height(px(19.))
                            .text_color(colors.text)
                            .when(first.is_some() || target.is_some(), |el| {
                                el.cursor_pointer()
                            })
                            .on_click(move |_, _, cx| {
                                opener
                                    .update(cx, |this, cx| {
                                        if let Some(target) = &target {
                                            if inside {
                                                cx.emit(HomeEvent::OpenBook(target.clone()));
                                            } else {
                                                cx.emit(HomeEvent::OpenNote(target.clone()));
                                            }
                                        } else if let Some(receipt) = &first {
                                            this.open_source(receipt, cx);
                                        }
                                    })
                                    .ok();
                            })
                            .child(line.text.clone())
                    })
                    .when(!meta.is_empty(), |col| {
                        col.child(
                            div()
                                .text_size(px(11.5))
                                .text_color(colors.text_muted)
                                .child(meta.join(" \u{b7} ")),
                        )
                    })
                    .when(ai, |col| {
                        col.children(sources.iter().enumerate().map(|(n, receipt)| {
                            let opener = view.clone();
                            let receipt_for_click = receipt.clone();
                            div()
                                .id(("home-source", unique * 16 + n))
                                .text_size(px(11.5))
                                .text_color(colors.link)
                                .cursor_pointer()
                                .on_click(move |_, _, cx| {
                                    opener
                                        .update(cx, |this, cx| {
                                            this.open_source(&receipt_for_click, cx)
                                        })
                                        .ok();
                                })
                                .child(receipt.source_line())
                        }))
                    }),
            )
            .when(ai, |row| {
                let marker = view.clone();
                let key = key.clone();
                row.child(
                    Button::new(("home-not-right", unique))
                        .ghost()
                        .xsmall()
                        .icon(Lucide::ThumbsDown)
                        .tooltip("Not right")
                        .on_click(move |_, _, cx| {
                            let key = key.clone();
                            marker
                                .update(cx, |this, cx| this.not_right(&key, index, cx))
                                .ok();
                        }),
                )
            })
            .into_any_element()
    }

    /// The Home note's words with an Edit button, or the offer to make one.
    fn render_home_note(&self, section: &Section, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let view = cx.entity().downgrade();
        let item = section.items.first();
        let text = item.map(|i| i.text.clone()).unwrap_or_default();
        let title = item.and_then(|i| i.meta.clone()).unwrap_or_default();
        let target = item.and_then(|i| i.target.clone());
        let editor = view.clone();
        v_flex()
            .debug_selector(|| "home-note".into())
            .w_full()
            .p(px(14.))
            .gap(px(6.))
            .rounded(px(10.))
            .border_1()
            .border_color(colors.divider)
            .bg(colors.fill)
            .child(
                h_flex()
                    .gap(px(8.))
                    .items_center()
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .text_size(px(12.))
                            .text_color(colors.text_muted)
                            .child(format!("Home note: \u{201c}{title}\u{201d}")),
                    )
                    .child(
                        Button::new("home-note-edit")
                            .ghost()
                            .xsmall()
                            .label("Edit")
                            .on_click(move |_, _, cx| {
                                if let Some(id) = &target {
                                    let id = id.clone();
                                    editor
                                        .update(cx, |_, cx| cx.emit(HomeEvent::EditHome(id)))
                                        .ok();
                                }
                            }),
                    ),
            )
            .child(
                div()
                    .text_size(px(14.))
                    .line_height(px(21.))
                    .text_color(colors.text)
                    .child(text),
            )
            .into_any_element()
    }

    /// The AI summary as one paragraph with its receipts.
    fn render_about(&self, section: &Section, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let view = cx.entity().downgrade();
        let text = section
            .items
            .iter()
            .map(|i| i.text.trim())
            .collect::<Vec<_>>()
            .join(" ");
        let receipts: Vec<Receipt> = section
            .items
            .iter()
            .flat_map(|i| i.sources.iter().cloned())
            .collect();
        v_flex()
            .gap(px(4.))
            .when(!text.is_empty(), |col| {
                col.child(
                    div()
                        .text_size(px(14.))
                        .line_height(px(21.))
                        .text_color(colors.text)
                        .child(text),
                )
            })
            .when(!receipts.is_empty(), |col| {
                col.child(h_flex().flex_wrap().gap(px(8.)).children(
                    receipts.iter().enumerate().map(|(n, receipt)| {
                        let opener = view.clone();
                        let r = receipt.clone();
                        div()
                            .id(("home-about-source", n))
                            .text_size(px(11.5))
                            .text_color(colors.link)
                            .cursor_pointer()
                            .on_click(move |_, _, cx| {
                                opener.update(cx, |this, cx| this.open_source(&r, cx)).ok();
                            })
                            .child(format!("[{}] {}", n + 1, receipt.source_line()))
                    }),
                ))
            })
            .into_any_element()
    }

    /// The map: the notebook's most linked notes and how they link.
    fn render_map(&self, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let view = cx.entity().downgrade();
        let Some(map) = self.map.clone() else {
            return div()
                .h(px(120.))
                .text_size(px(12.))
                .text_color(colors.text_muted)
                .child("Drawing the map\u{2026}")
                .into_any_element();
        };
        let book = self.book.clone().unwrap_or_default();
        let edge = Hsla::from(colors.text_muted).opacity(0.45);
        let node = Hsla::from(colors.accent);
        let stub = Hsla::from(colors.text_muted).opacity(0.6);
        let (w, h) = (map.width, map.height);
        let painter = map.clone();
        v_flex()
            .gap(px(6.))
            .child(
                div()
                    .debug_selector(|| "home-map".into())
                    .w(px(w))
                    .h(px(h))
                    .max_w_full()
                    .rounded(px(10.))
                    .border_1()
                    .border_color(colors.divider)
                    .bg(colors.fill)
                    .child(
                        canvas(
                            |bounds, _, _| bounds,
                            move |_, bounds, window, _| {
                                paint_map(&painter, bounds, edge, node, stub, window);
                            },
                        )
                        .size_full(),
                    ),
            )
            .child(
                h_flex().child(
                    Button::new("home-open-graph")
                        .ghost()
                        .xsmall()
                        .label("Open in graph \u{203a}")
                        .on_click(move |_, _, cx| {
                            let book = book.clone();
                            view.update(cx, |_, cx| cx.emit(HomeEvent::Graph(book)))
                                .ok();
                        }),
                ),
            )
            .into_any_element()
    }

    fn render_section(
        &self,
        section: &Section,
        ai_on: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = Colors::get(cx);
        let view = cx.entity().downgrade();
        let key = section.key.clone();
        let ix = section_index(&key);
        let folded = self.folded.contains(&key);
        let working = self.working.contains(&key);
        let pinned = matches!(section.state, SectionState::Pinned { .. });
        let ai = section.kind == SectionKind::Ai;
        let choices = section_actions(section, ai_on);
        let menu_view = view.clone();
        let menu_key = key.clone();
        let menu = (!choices.is_empty()).then(|| {
            Button::new(("home-section-menu", ix))
                .ghost()
                .xsmall()
                .icon(Lucide::Ellipsis)
                .dropdown_menu(move |mut menu, _, _| {
                    for action in choices.iter().copied() {
                        let (view, key) = (menu_view.clone(), menu_key.clone());
                        menu =
                            menu.item(PopupMenuItem::new(action_label(action, pinned)).on_click(
                                move |_, _, cx| {
                                    let key = key.clone();
                                    view.update(cx, |this, cx| this.run(&key, action, cx)).ok();
                                },
                            ));
                    }
                    menu
                })
        });
        let toggler = view.clone();
        let toggle_key = key.clone();
        let written = section.written_at.map(|ms| {
            format!(
                "Written {}",
                kanso_engine::pages::notebook::relative_day(now_ms(), ms)
            )
        });
        let header = h_flex()
            .gap(px(6.))
            .items_center()
            .child(
                h_flex()
                    .id(("home-section-title", ix))
                    .flex_1()
                    .min_w_0()
                    .gap(px(6.))
                    .cursor_pointer()
                    .on_click(move |_, _, cx| {
                        let key = toggle_key.clone();
                        toggler
                            .update(cx, |this, cx| {
                                if !this.folded.remove(&key) {
                                    this.folded.insert(key);
                                }
                                cx.notify();
                            })
                            .ok();
                    })
                    .child(
                        Icon::new(if folded {
                            Lucide::ChevronRight
                        } else {
                            Lucide::ChevronDown
                        })
                        .size(px(12.))
                        .text_color(colors.text_muted),
                    )
                    .child(
                        div()
                            .text_size(px(13.))
                            .font_semibold()
                            .text_color(colors.text)
                            .child(section.title.clone()),
                    )
                    .when(pinned, |row| {
                        row.child(
                            Icon::new(Lucide::Pin)
                                .size(px(11.))
                                .text_color(colors.text_muted),
                        )
                    }),
            )
            .when(ai, |row| {
                row.child(
                    div()
                        .id(("home-ai-tag", ix))
                        .text_size(px(10.5))
                        .text_color(colors.text_muted)
                        .when_some(written, |tag, written| {
                            tag.tooltip(move |window, cx| {
                                gpui_kit::component::tooltip::Tooltip::new(written.clone())
                                    .build(window, cx)
                            })
                        })
                        .child(section.kind.tag()),
                )
            })
            .when(section.key == "open_tasks" && section.total > 0, |row| {
                row.child(
                    div()
                        .text_size(px(11.5))
                        .text_color(colors.text_muted)
                        .child(format!("{} open", section.total)),
                )
            })
            .when(working, |row| {
                row.child(
                    div()
                        .text_size(px(11.5))
                        .text_color(colors.text_muted)
                        .child("Writing\u{2026}"),
                )
            })
            .children(menu);

        let body = (!folded).then(|| {
            let mut group: Option<String> = None;
            let mut col = v_flex().pl(px(18.)).gap(px(1.));
            if let Some(stale) = section.stale_line() {
                let refresh = view.clone();
                let refresh_key = key.clone();
                col = col.child(
                    h_flex()
                        .gap(px(8.))
                        .items_center()
                        .child(
                            div()
                                .text_size(px(11.5))
                                .text_color(colors.warning)
                                .child(stale),
                        )
                        .when(ai_on && !working && section.wants_refresh(), |row| {
                            row.child(
                                Button::new(("home-refresh", ix))
                                    .ghost()
                                    .xsmall()
                                    .label("Refresh")
                                    .on_click(move |_, _, cx| {
                                        let key = refresh_key.clone();
                                        refresh
                                            .update(cx, |this, cx| this.write(vec![key], cx))
                                            .ok();
                                    }),
                            )
                        }),
                );
            }
            if section.state == SectionState::Unwritten && ai_on && !working {
                let write = view.clone();
                let write_key = key.clone();
                col = col.child(
                    h_flex().py(px(4.)).child(
                        Button::new(("home-write", ix))
                            .outline()
                            .small()
                            .icon(Lucide::Sparkles)
                            .label(FIRST_VISIT_BUTTON)
                            .on_click(move |_, _, cx| {
                                let key = write_key.clone();
                                write.update(cx, |this, cx| this.write(vec![key], cx)).ok();
                            }),
                    ),
                );
            }
            if let Some(reason) = self.trouble.get(&key) {
                let (retry, dismiss) = (view.clone(), view.clone());
                let (retry_key, dismiss_key) = (key.clone(), key.clone());
                col = col.child(ai_error_card(
                    &ai_trouble(reason),
                    move |_, cx| {
                        let key = retry_key.clone();
                        retry.update(cx, |this, cx| this.write(vec![key], cx)).ok();
                    },
                    move |_, cx| {
                        let key = dismiss_key.clone();
                        dismiss
                            .update(cx, |this, cx| {
                                this.trouble.remove(&key);
                                cx.notify();
                            })
                            .ok();
                    },
                    window,
                    cx,
                ));
            }
            if section.items.is_empty()
                && !self.trouble.contains_key(&key)
                && section.state != SectionState::Unwritten
                && let Some(line) = pages::empty_line(section)
            {
                col = col.child(
                    div()
                        .text_size(px(12.5))
                        .text_color(colors.text_muted)
                        .child(line),
                );
            }
            match section.key.as_str() {
                "home_note" => col = col.child(self.render_home_note(section, cx)),
                "about" => col = col.child(self.render_about(section, cx)),
                "map" => col = col.child(self.render_map(cx)),
                _ => {
                    for (index, item) in section.items.iter().enumerate() {
                        if let Some(heading) =
                            item.group.as_ref().filter(|g| group.as_ref() != Some(*g))
                        {
                            group = Some(heading.clone());
                            col = col.child(
                                div()
                                    .pt(px(4.))
                                    .text_size(px(11.5))
                                    .text_color(colors.text_muted)
                                    .child(heading.clone()),
                            );
                        }
                        col = col.child(self.render_item(section, index, item, cx));
                    }
                }
            }
            if section.total > section.items.len() && section.kind == SectionKind::Computed {
                let all = view.clone();
                let all_key = key.clone();
                col = col.child(
                    h_flex().child(
                        Button::new(("home-show-all", ix))
                            .ghost()
                            .xsmall()
                            .label(format!("Show all {} \u{203a}", section.total))
                            .on_click(move |_, _, cx| {
                                let key = all_key.clone();
                                all.update(cx, |this, cx| this.show_all(&key, cx)).ok();
                            }),
                    ),
                );
            }
            if let Some(note) = &section.note
                && section.items.is_empty()
            {
                col = col.child(
                    div()
                        .text_size(px(12.5))
                        .text_color(colors.text_muted)
                        .child(note.clone()),
                );
            }
            col
        });
        let selector = format!("home-section-{}", section.key);
        v_flex()
            .debug_selector(move || selector.clone())
            .flex_1()
            .min_w_0()
            .gap(px(4.))
            .py(px(8.))
            .child(header)
            .children(body)
            .into_any_element()
    }

    fn render_header(&self, loaded: &Loaded, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let view = cx.entity().downgrade();
        let ai_enabled = cx.global::<AppState>().config.ai.enabled;
        let ai_on = loaded.view.ai_on;
        let locked_open = loaded.locked_open;
        let book = self.book.clone().unwrap_or_default();
        let refreshable = self.refreshable();
        let hidden = loaded.view.page.hidden.len();
        let working = !self.working.is_empty();
        let has_home = loaded.view.header.has_home_note;
        let candidates: Vec<(String, String)> = self
            .store
            .notes_in_books(std::slice::from_ref(&book))
            .unwrap_or_default()
            .into_iter()
            .filter(|n| !n.is_trashed() && !n.is_locked())
            .take(15)
            .map(|n| (n.id, n.title))
            .collect();

        let menu = {
            let (refresh, shower, home, customiser) =
                (view.clone(), view.clone(), view.clone(), view.clone());
            Button::new("home-menu")
                .ghost()
                .small()
                .icon(Lucide::Ellipsis)
                .tooltip("More")
                .dropdown_menu(move |mut menu, window, cx| {
                    if ai_on && !refreshable.is_empty() && !locked_open {
                        let (view, keys) = (refresh.clone(), refreshable.clone());
                        menu = menu.item(PopupMenuItem::new("Refresh all").on_click(
                            move |_, _, cx| {
                                let keys = keys.clone();
                                view.update(cx, |this, cx| this.write(keys, cx)).ok();
                            },
                        ));
                    }
                    if hidden > 0 {
                        let view = shower.clone();
                        menu = menu.item(PopupMenuItem::new("Show hidden sections").on_click(
                            move |_, _, cx| {
                                view.update(cx, |this, cx| this.show_hidden(cx)).ok();
                            },
                        ));
                    }
                    let items = candidates.clone();
                    let chooser = home.clone();
                    menu = menu.submenu(
                        "Choose Home note\u{2026}",
                        window,
                        cx,
                        move |mut sub, _, _| {
                            for (id, title) in items.clone() {
                                let view = chooser.clone();
                                let label = if title.trim().is_empty() {
                                    "Untitled".to_owned()
                                } else {
                                    title
                                };
                                sub = sub.item(PopupMenuItem::new(label).on_click(
                                    move |_, _, cx| {
                                        let id = id.clone();
                                        view.update(cx, |this, cx| this.choose_home(&id, cx)).ok();
                                    },
                                ));
                            }
                            sub
                        },
                    );
                    let view = home.clone();
                    menu = menu.item(PopupMenuItem::new("New Home note").on_click(
                        move |_, _, cx| {
                            view.update(cx, |this, cx| this.new_home(cx)).ok();
                        },
                    ));
                    if has_home {
                        let view = home.clone();
                        menu = menu.item(PopupMenuItem::new("Stop using a Home note").on_click(
                            move |_, _, cx| {
                                view.update(cx, |this, cx| this.clear_home(cx)).ok();
                            },
                        ));
                    }
                    let view = customiser.clone();
                    menu.separator()
                        .item(PopupMenuItem::new("Customise home\u{2026}").on_click(
                            move |_, window, cx| {
                                crate::home_dialogs::open_customise(view.clone(), window, cx);
                            },
                        ))
                })
        };

        let ask_view = view.clone();
        let ask_book = book.clone();
        let refresh_all_view = view.clone();
        let refresh_keys = self.refreshable();
        h_flex()
            .id("home-header")
            .flex_none()
            .w_full()
            .px(px(24.))
            .pt(px(14.))
            .pb(px(10.))
            .gap(px(10.))
            .items_start()
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap(px(2.))
                    .child(
                        div()
                            .debug_selector(|| "home-title".into())
                            .truncate()
                            .text_size(px(22.))
                            .font_semibold()
                            .text_color(colors.text)
                            .child(loaded.view.header.name.clone()),
                    )
                    .child(
                        div()
                            .debug_selector(|| "home-line".into())
                            .text_size(px(12.5))
                            .text_color(colors.text_muted)
                            .child(loaded.view.header.line.clone()),
                    ),
            )
            .when(ai_enabled && !locked_open, |row| {
                row.child(
                    Button::new("home-ask")
                        .debug_selector(|| "home-ask".into())
                        .outline()
                        .small()
                        .icon(Lucide::MessageCircle)
                        .label(ASK_BUTTON)
                        .on_click(move |_, _, cx| {
                            let book = ask_book.clone();
                            ask_view
                                .update(cx, |_, cx| cx.emit(HomeEvent::Ask(book)))
                                .ok();
                        }),
                )
            })
            .when(ai_on && !locked_open && !refresh_keys.is_empty(), |row| {
                row.child(
                    Button::new("home-refresh-all")
                        .outline()
                        .small()
                        .icon(Lucide::RefreshCw)
                        .label("Refresh")
                        .loading(working)
                        .on_click(move |_, _, cx| {
                            let keys = refresh_keys.clone();
                            refresh_all_view
                                .update(cx, |this, cx| this.write(keys, cx))
                                .ok();
                        }),
                )
            })
            .child(menu)
            .into_any_element()
    }
}

fn section_index(key: &str) -> usize {
    kanso_core::home::SECTION_KEYS
        .iter()
        .position(|k| *k == key)
        .unwrap_or(0)
}

/// Draws the map into `bounds`: links as thin lines, notes as dots, and a
/// short stub on a note for each link that leaves the notebook.
fn paint_map(
    map: &MapView,
    bounds: Bounds<Pixels>,
    edge: Hsla,
    node: Hsla,
    stub: Hsla,
    window: &mut Window,
) {
    let scale_x = f32::from(bounds.size.width) / map.width.max(1.);
    let scale_y = f32::from(bounds.size.height) / map.height.max(1.);
    let at = |x: f32, y: f32| {
        point(
            bounds.origin.x + gpui_kit::px(x * scale_x),
            bounds.origin.y + gpui_kit::px(y * scale_y),
        )
    };
    let mut lines = PathBuilder::stroke(gpui_kit::px(1.));
    let mut any = false;
    for &(a, b) in &map.edges {
        if a == b {
            continue;
        }
        let (pa, pb) = (&map.nodes[a as usize], &map.nodes[b as usize]);
        lines.move_to(at(pa.x, pa.y));
        lines.line_to(at(pb.x, pb.y));
        any = true;
    }
    if any && let Ok(path) = lines.build() {
        window.paint_path(path, edge);
    }
    let mut stubs = PathBuilder::stroke(gpui_kit::px(1.));
    let mut any_stub = false;
    for n in &map.nodes {
        for k in 0..n.stubs.min(4) {
            let angle = -0.8 + k as f32 * 0.55;
            let (dx, dy) = (angle.cos() * (n.radius + 9.), angle.sin() * (n.radius + 9.));
            stubs.move_to(at(n.x, n.y));
            stubs.line_to(at(n.x + dx, n.y + dy));
            any_stub = true;
        }
    }
    if any_stub && let Ok(path) = stubs.build() {
        window.paint_path(path, stub);
    }
    for n in &map.nodes {
        let r = n.radius.clamp(2., 9.);
        let c = at(n.x, n.y);
        window.paint_quad(
            fill(
                Bounds::new(
                    point(c.x - gpui_kit::px(r), c.y - gpui_kit::px(r)),
                    size(gpui_kit::px(r * 2.), gpui_kit::px(r * 2.)),
                ),
                node,
            )
            .corner_radii(gpui_kit::px(r)),
        );
    }
}

impl Render for NotebookHomeView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let measure = {
            let view = cx.entity().downgrade();
            canvas(
                |bounds, _, _| bounds,
                move |_, bounds, window, cx| {
                    let width = crate::zoom::unscaled(bounds.size.width);
                    let view = view.clone();
                    window.defer(cx, move |_, cx| {
                        view.update(cx, |this, cx| this.set_pane_width(width, cx))
                            .ok();
                    });
                },
            )
            .absolute()
            .size_full()
        };
        let Some(loaded) = self.loaded.clone() else {
            return div()
                .id("notebook-home")
                .debug_selector(|| "notebook-home".into())
                .size_full()
                .relative()
                .bg(colors.editor)
                .child(measure)
                .into_any_element();
        };
        let ai_on = loaded.view.ai_on;
        let two = self.pane_width >= TWO_COLUMNS;
        let keys: Vec<String> = loaded
            .view
            .page
            .sections
            .iter()
            .map(|s| s.key.clone())
            .collect();
        let rows = rows_of(&keys, two);
        let hidden = loaded.view.page.hidden.len();
        let mut body = v_flex()
            .id("home-body")
            .w_full()
            .max_w(px(if two { WIDE_COLUMN } else { NARROW_COLUMN }))
            .px(px(24.))
            .pb(px(48.));
        for row in rows {
            let mut line = h_flex().w_full().gap(px(32.)).items_start();
            let alone = row.len() == 1 && two && !full_width(&keys[row[0]]);
            for ix in &row {
                line = line.child(self.render_section(
                    &loaded.view.page.sections[*ix],
                    ai_on,
                    window,
                    cx,
                ));
            }
            if alone {
                line = line.child(div().flex_1());
            }
            body = body.child(line);
        }
        if hidden > 0 {
            let shower = cx.entity().downgrade();
            body = body.child(
                h_flex().child(
                    Button::new("home-show-hidden")
                        .ghost()
                        .xsmall()
                        .label(format!(
                            "Show {hidden} hidden {}",
                            if hidden == 1 { "section" } else { "sections" }
                        ))
                        .on_click(move |_, _, cx| {
                            shower.update(cx, |this, cx| this.show_hidden(cx)).ok();
                        }),
                ),
            );
        }
        if let Some(footer) = loaded.view.footer.clone() {
            body = body.child(
                div()
                    .debug_selector(|| "home-footer".into())
                    .pt(px(14.))
                    .text_size(px(12.))
                    .text_color(colors.text_muted)
                    .child(footer),
            );
        }
        div()
            .id("notebook-home")
            .debug_selector(|| "notebook-home".into())
            .size_full()
            .relative()
            .bg(colors.editor)
            .child(
                v_flex()
                    .size_full()
                    .items_center()
                    .child(
                        div()
                            .w_full()
                            .max_w(px(if two { WIDE_COLUMN } else { NARROW_COLUMN }))
                            .child(self.render_header(&loaded, cx)),
                    )
                    .child(
                        div()
                            .id("home-scroll")
                            .flex_1()
                            .min_h_0()
                            .w_full()
                            .overflow_y_scrollbar()
                            .track_scroll(&self.scroll)
                            .child(h_flex().w_full().justify_center().child(body)),
                    ),
            )
            .child(measure)
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn keys(n: usize, wide_at: &[usize]) -> Vec<String> {
        (0..n)
            .map(|i| {
                if wide_at.contains(&i) {
                    "about".to_owned()
                } else {
                    format!("s{i}")
                }
            })
            .collect()
    }

    fn check_rows(n: usize, wide: &[usize], two: bool) -> Result<(), TestCaseError> {
        let keys = keys(n, wide);
        let rows = rows_of(&keys, two);
        let flat: Vec<usize> = rows.iter().flatten().copied().collect();
        prop_assert_eq!(flat, (0..n).collect::<Vec<_>>());
        let per_row = if two { 2 } else { 1 };
        for row in &rows {
            prop_assert!(row.len() <= per_row);
            if row.len() == 2 {
                prop_assert!(row.iter().all(|&i| !full_width(&keys[i])));
            }
        }
        Ok(())
    }

    /// Every section is in exactly one row, in order, a wide one is alone,
    /// and a row holds at most two (one in one column).
    #[test]
    fn rows_hold_each_section_once_in_order() {
        let mut runner = proptest::test_runner::TestRunner::new(ProptestConfig {
            cases: 64,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        });
        let strategy = (
            0usize..20,
            proptest::collection::vec(0usize..20, 0..4),
            any::<bool>(),
        );
        runner
            .run(&strategy, |(n, wide, two)| check_rows(n, &wide, two))
            .unwrap();
    }

    #[test]
    fn the_menu_offers_only_what_applies() {
        let section = |key: &str, kind| Section {
            key: key.to_owned(),
            title: key.to_owned(),
            kind,
            items: Vec::new(),
            state: SectionState::Fresh,
            written_at: None,
            total: 0,
            note: None,
            none: false,
        };
        let ai = section("about", SectionKind::Ai);
        assert_eq!(
            section_actions(&ai, true),
            vec![
                SectionAction::Refresh,
                SectionAction::Keep,
                SectionAction::Hide
            ]
        );
        assert_eq!(
            section_actions(&ai, false),
            vec![SectionAction::Keep, SectionAction::Hide]
        );
        let note = section("home_note", SectionKind::Computed);
        assert!(section_actions(&note, true).is_empty());
    }
}
