//! The app's view of the store: notebooks, tags, counts and the note list for
//! the current filter and search. Views read it; writes go to the store, and
//! the store's change feed brings the lists back up to date.

use std::collections::HashMap;
use std::sync::Arc;

use gpui_kit::*;
use kanso_core::tags::{TagInfo, TagNode, tag_tree};
use kanso_core::{Book, NoteStatus, Tag};
use kanso_store::{Change, NoteSummary, Sort, StatusCounts, Store, StoreError};

use crate::clock::Clock;
use crate::selection::{Click, Selection};

pub use kanso_engine::list::{
    DAY_MS, Filter, RECENT_DAYS, book_path, book_total_counts, has_notes_inside, is_book_locked,
    list_notes, notebook_tree as book_tree, refine_status, search_params,
};

/// The place a board lays out for the list's `filter`: the same notebook
/// or tag, but every status, since the board's columns are the statuses.
/// Tasks and the trash have no board of their own; theirs is all notes.
pub fn board_scope(filter: &Filter) -> Filter {
    match filter {
        Filter::Status(_)
        | Filter::Tasks
        | Filter::OpenLoops
        | Filter::Overview
        | Filter::Trash => Filter::All,
        other => other.clone(),
    }
}

pub enum LibraryEvent {
    /// The selected note changed: `asked` when someone picked it, rather
    /// than the app on its own (the first note of a new list, the last note
    /// at launch).
    Selected { id: Option<String>, asked: bool },
    /// A place was asked for (a notebook, tag, status or list), even the
    /// one already showing: the notes should be in view.
    Shown,
    /// A document changed in the store.
    Changed(Change),
    /// The notebook homepage was opened or closed, or the place it belongs
    /// to changed.
    Home,
}

/// Whether notebook homepages are on (Settings › Notebooks): a notebook then
/// offers its Home row and opens its homepage when it has nothing else to
/// show. Off in a window without settings.
pub fn homepages_on(cx: &App) -> bool {
    cx.try_global::<crate::workspace::AppState>()
        .is_some_and(|state| state.config.people_projects.notebook_homepages)
}

pub struct Library {
    pub store: Arc<Store>,
    pub books: Vec<Book>,
    /// Notes in each notebook and every notebook inside it, by notebook ID.
    book_counts: HashMap<String, u32>,
    pub tags: Vec<Tag>,
    /// The tags nested by `/` (`kanso_core::tags::tag_tree`).
    pub tag_tree: Vec<TagNode>,
    pub counts: StatusCounts,
    /// How many notes Recent lists, without the search.
    pub recent: u32,
    /// How many open tasks are overdue, for the sidebar's Tasks badge.
    pub overdue: u32,
    /// How many loops are open, for the sidebar's Open Loops count.
    pub open_loops: u32,
    /// How many notes others shared with this account.
    pub shared: u32,
    /// The person and project pages: `(note ID, name, kind)`.
    pub people: Vec<(String, String, kanso_core::entities::EntityKind)>,
    /// How many names Kanso suggests a page for, for the sidebar's count.
    pub people_new: u32,
    /// The suggestions: `(key, name, kind)`, for "Create a page for X" in
    /// the editor.
    pub people_offers: Vec<(String, String, kanso_core::entities::EntityKind)>,
    pub filter: Filter,
    pub search: String,
    pub sort: Sort,
    pub descending: bool,
    pub notes: Vec<NoteSummary>,
    /// Changes whenever `notes` is replaced, so views can tell a new list
    /// from a repaint.
    pub notes_version: u64,
    /// The note list's own narrowing on top of the filter: one status, or
    /// pinned notes only. A new filter clears both.
    pub status_only: Option<NoteStatus>,
    pub pinned_only: bool,
    pub selected: Option<String>,
    /// The notes selected in the list: `selected` alone, unless ⇧- or
    /// ⌘-click or ⌘A selected more. Its focus is always `selected`.
    pub selection: Selection,
    /// The selected notebook's homepage shows in the editor's place.
    pub home: bool,
    pub load_error: Option<String>,
    generation: u64,
    /// After a filter or search change, select the first note when the
    /// selection is not in the new list.
    reselect: bool,
    /// The last place that wasn't one of the pages (the brief, the review,
    /// open loops), for Escape to go back to.
    pub previous: Filter,
    reload_task: Option<Task<()>>,
    _watch: Task<()>,
}

impl EventEmitter<LibraryEvent> for Library {}

struct Loaded {
    books: Vec<Book>,
    book_counts: HashMap<String, u32>,
    tags: Vec<Tag>,
    tag_tree: Vec<TagNode>,
    counts: StatusCounts,
    recent: u32,
    overdue: u32,
    open_loops: u32,
    shared: u32,
    people: Vec<(String, String, kanso_core::entities::EntityKind)>,
    people_new: u32,
    people_offers: Vec<(String, String, kanso_core::entities::EntityKind)>,
    /// `None` while the Tasks view shows: the list keeps its notes.
    notes: Option<Vec<NoteSummary>>,
}

impl Library {
    pub fn new(store: Arc<Store>, cx: &mut Context<Self>) -> Self {
        let changes = store.subscribe();
        let watch = cx.spawn(async move |this, cx| {
            while let Ok(first) = changes.recv().await {
                let mut batch = vec![first];
                while let Ok(change) = changes.try_recv() {
                    batch.push(change);
                }
                let alive = this.update(cx, |this, cx| {
                    this.reload(cx);
                    for change in batch {
                        cx.emit(LibraryEvent::Changed(change));
                    }
                });
                if alive.is_err() {
                    break;
                }
            }
        });
        let mut library = Library {
            store,
            books: Vec::new(),
            book_counts: HashMap::new(),
            tags: Vec::new(),
            tag_tree: Vec::new(),
            counts: StatusCounts::default(),
            recent: 0,
            overdue: 0,
            open_loops: 0,
            shared: 0,
            people: Vec::new(),
            people_new: 0,
            people_offers: Vec::new(),
            filter: Filter::All,
            search: String::new(),
            sort: Sort::UpdatedAt,
            descending: true,
            notes: Vec::new(),
            notes_version: 0,
            status_only: None,
            pinned_only: false,
            selected: None,
            selection: Selection::default(),
            home: false,
            load_error: None,
            generation: 0,
            reselect: true,
            previous: Filter::All,
            reload_task: None,
            _watch: watch,
        };
        library.reload(cx);
        library
    }

    /// Reads the overdue and open-loop counts again on a background
    /// thread: tasks fall overdue as time passes. A reload in flight reads
    /// both itself and wins.
    pub fn refresh_overdue(&mut self, cx: &mut Context<Self>) {
        let now =
            kanso_core::dates::DateTime::from_local_ms(Clock::local_ms(cx, Clock::now_ms(cx)));
        let store = self.store.clone();
        let generation = self.generation;
        let task = cx.background_executor().spawn(async move {
            let overdue = store.overdue_count(now).ok()?;
            let open = store
                .open_loop_count(&kanso_store::LoopQuery::default())
                .ok()?;
            Some((overdue, open))
        });
        cx.spawn(async move |this, cx| {
            let Some((overdue, open)) = task.await else {
                return;
            };
            this.update(cx, |this, cx| {
                if this.generation == generation
                    && (overdue != this.overdue || open != this.open_loops)
                {
                    this.overdue = overdue;
                    this.open_loops = open;
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    /// Reloads everything on a background thread; a newer reload wins.
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        self.generation += 1;
        let generation = self.generation;
        let store = self.store.clone();
        let mut params = search_params(&self.filter, &self.search, self.sort, self.descending);
        refine_status(&mut params, self.status_only);
        let filter = self.filter.clone();
        let pinned_only = filter == Filter::Pinned || self.pinned_only;
        // Recent as listed, unnarrowed, is the count itself.
        let recent_is_listed =
            filter == Filter::Recent && self.search.trim().is_empty() && !self.is_refined();
        let recent_since = Clock::now_ms(cx) - RECENT_DAYS * DAY_MS;
        let now =
            kanso_core::dates::DateTime::from_local_ms(Clock::local_ms(cx, Clock::now_ms(cx)));
        let people_settings = cx
            .try_global::<crate::workspace::AppState>()
            .map(|state| state.config.people_projects.clone())
            .unwrap_or_default();
        let task = cx.background_executor().spawn(async move {
            let notes = match filter {
                Filter::Tasks | Filter::OpenLoops | Filter::Overview => None,
                _ => Some(list_notes(
                    &store,
                    &filter,
                    &params,
                    pinned_only,
                    recent_since,
                )?),
            };
            let recent = match &notes {
                Some(notes) if recent_is_listed => u32::try_from(notes.len()).unwrap_or(u32::MAX),
                _ => store.count_recent(recent_since)?,
            };
            let tags = store.tags()?;
            let books = store.books()?;
            let infos: Vec<TagInfo> = tags
                .iter()
                .map(|tag| TagInfo {
                    id: tag.id.clone(),
                    name: tag.name.clone(),
                })
                .collect();
            let people_offers: Vec<_> =
                kanso_engine::entities::suggest::suggestions(&store, &people_settings)?
                    .into_iter()
                    .filter(|s| s.alias_of.is_none())
                    .map(|s| (s.key, s.name, s.kind))
                    .collect();
            Ok::<_, StoreError>(Loaded {
                book_counts: book_total_counts(&books),
                books,
                tag_tree: tag_tree(&infos, &store.tag_links()?),
                tags,
                counts: store.status_counts()?,
                recent,
                overdue: store.overdue_count(now)?,
                open_loops: store.open_loop_count(&kanso_store::LoopQuery::default())?,
                shared: u32::try_from(store.shared_with_me()?.len()).unwrap_or(u32::MAX),
                people: store
                    .entities()?
                    .into_iter()
                    .map(|page| (page.id, page.title, page.kind))
                    .collect(),
                people_new: u32::try_from(people_offers.len()).unwrap_or(u32::MAX),
                people_offers,
                notes,
            })
        });
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            let loaded = task.await;
            this.update(cx, |this, cx| {
                if this.generation != generation {
                    return;
                }
                match loaded {
                    Ok(loaded) => {
                        this.books = loaded.books;
                        this.book_counts = loaded.book_counts;
                        this.tags = loaded.tags;
                        this.tag_tree = loaded.tag_tree;
                        this.counts = loaded.counts;
                        this.recent = loaded.recent;
                        this.overdue = loaded.overdue;
                        this.open_loops = loaded.open_loops;
                        this.shared = loaded.shared;
                        this.people = loaded.people;
                        this.people_new = loaded.people_new;
                        this.people_offers = loaded.people_offers;
                        if let Some(notes) = loaded.notes {
                            this.notes = notes;
                            this.notes_version += 1;
                            let listed: Vec<&str> =
                                this.notes.iter().map(|n| n.id.as_str()).collect();
                            this.selection.retain(&listed);
                        }
                        this.load_error = None;
                    }
                    Err(err) => this.load_error = Some(err.to_string()),
                }
                if std::mem::take(&mut this.reselect) {
                    let listed = this
                        .selected
                        .as_ref()
                        .is_some_and(|id| this.notes.iter().any(|n| &n.id == id));
                    if !listed {
                        let first = this.notes.first().map(|n| n.id.clone());
                        this.select_quietly(first, cx);
                    }
                }
                cx.notify();
            })
            .ok();
        }));
    }

    pub fn set_filter(&mut self, filter: Filter, cx: &mut Context<Self>) {
        cx.emit(LibraryEvent::Shown);
        if self.filter != filter {
            if !matches!(self.filter, Filter::OpenLoops | Filter::Overview) {
                self.previous = self.filter.clone();
            }
            // The Tasks view and the other places leave the open note as it is.
            self.reselect = !filter.is_place();
            self.filter = filter;
            self.status_only = None;
            self.pinned_only = false;
            self.home = false;
            self.open_home_if_due(homepages_on(cx));
            cx.emit(LibraryEvent::Home);
            self.reload(cx);
        }
    }

    /// Opens the selected notebook's homepage on its own when homepages
    /// are on and the notebook has nothing else to show: it has no notes
    /// directly in it but has notebooks inside it, or no note of it is the
    /// open one. Otherwise the open note stays open.
    fn open_home_if_due(&mut self, homepages: bool) {
        let Filter::Book(id) = &self.filter else {
            return;
        };
        if !homepages || self.is_book_locked(id) || self.book(id).is_none() {
            return;
        }
        let direct = self.book(id).map_or(0, |b| b.count);
        let parent_only = direct == 0 && self.has_notes_inside(id);
        let note_open = self.selected.as_deref().is_some_and(|note| {
            self.store
                .get(note)
                .ok()
                .flatten()
                .and_then(|doc| {
                    doc.fields
                        .get("bookId")
                        .and_then(|b| b.as_str().map(str::to_owned))
                })
                .is_some_and(|book| &book == id)
        });
        // A notebook with nothing in it keeps the editor's own empty block.
        let empty = direct == 0 && !parent_only;
        if parent_only || (!note_open && !empty) {
            self.home = true;
            self.reselect = false;
        }
    }

    /// Shows (or closes) the selected notebook's homepage. Does nothing
    /// unless a notebook is selected and homepages are on.
    pub fn set_home(&mut self, on: bool, cx: &mut Context<Self>) {
        let on = on && homepages_on(cx) && matches!(self.filter, Filter::Book(_));
        if self.home != on {
            self.home = on;
            cx.emit(LibraryEvent::Home);
            cx.notify();
        }
    }

    /// The notebook whose homepage shows, if one does.
    pub fn home_book(&self, cx: &App) -> Option<&str> {
        match &self.filter {
            Filter::Book(id) if self.home && homepages_on(cx) => Some(id),
            _ => None,
        }
    }

    /// Leaves a page for `filter` with note `keep` still the one selected,
    /// rather than the list's first.
    pub fn leave_page(&mut self, filter: Filter, keep: bool, cx: &mut Context<Self>) {
        self.set_filter(filter, cx);
        if keep {
            self.reselect = false;
        }
    }

    pub fn set_status_only(&mut self, status: Option<NoteStatus>, cx: &mut Context<Self>) {
        if self.status_only != status {
            self.status_only = status;
            self.reselect = true;
            self.reload(cx);
        }
    }

    pub fn set_pinned_only(&mut self, pinned_only: bool, cx: &mut Context<Self>) {
        if self.pinned_only != pinned_only {
            self.pinned_only = pinned_only;
            self.reselect = true;
            self.reload(cx);
        }
    }

    /// Whether the note list narrows the filter further.
    pub fn is_refined(&self) -> bool {
        self.status_only.is_some() || self.pinned_only
    }

    pub fn clear_refinements(&mut self, cx: &mut Context<Self>) {
        if self.is_refined() {
            self.status_only = None;
            self.pinned_only = false;
            self.reselect = true;
            self.reload(cx);
        }
    }

    pub fn set_search(&mut self, search: String, cx: &mut Context<Self>) {
        if self.search != search {
            self.search = search;
            self.reselect = true;
            self.reload(cx);
        }
    }

    pub fn set_sort(&mut self, sort: Sort, descending: bool, cx: &mut Context<Self>) {
        self.sort = sort;
        self.descending = descending;
        self.reload(cx);
    }

    /// Selects the note someone picked.
    pub fn select(&mut self, id: Option<String>, cx: &mut Context<Self>) {
        self.select_as(id, true, cx);
    }

    /// Selects a note the app picked on its own, which doesn't take the
    /// window away from what it shows (Settings, say).
    pub fn select_quietly(&mut self, id: Option<String>, cx: &mut Context<Self>) {
        self.select_as(id, false, cx);
    }

    /// A note picked again that is already selected still says so, so
    /// whatever hides it (Settings) gives way.
    fn select_as(&mut self, id: Option<String>, asked: bool, cx: &mut Context<Self>) {
        if self.selection != Selection::one(id.clone()) {
            self.selection = Selection::one(id.clone());
            cx.notify();
        }
        self.show_focus(id, asked, cx);
    }

    /// Shows `id` in the editor, leaving the selection as it is.
    fn show_focus(&mut self, id: Option<String>, asked: bool, cx: &mut Context<Self>) {
        let changed = self.selected != id;
        if asked && id.is_some() && self.home {
            // Someone picked a note: it takes the editor back.
            self.home = false;
            cx.emit(LibraryEvent::Home);
        }
        if changed || asked {
            self.selected = id.clone();
            cx.emit(LibraryEvent::Selected { id, asked });
        }
        if changed {
            cx.notify();
        }
    }

    /// A click on the listed note `id`: alone, toggled, or a run from the
    /// last plain click.
    pub fn click(&mut self, id: &str, how: Click, cx: &mut Context<Self>) {
        let listed: Vec<&str> = self.notes.iter().map(|n| n.id.as_str()).collect();
        self.selection.click(id, how, &listed);
        let focus = self.selection.focus().map(str::to_owned);
        cx.notify();
        self.show_focus(focus, true, cx);
    }

    /// Selects every listed note, ⌘A.
    pub fn select_all(&mut self, cx: &mut Context<Self>) {
        let listed: Vec<&str> = self.notes.iter().map(|n| n.id.as_str()).collect();
        self.selection.select_all(&listed);
        let focus = self.selection.focus().map(str::to_owned);
        cx.notify();
        self.show_focus(focus, false, cx);
    }

    /// Back to the note the editor shows alone, Escape.
    pub fn select_one(&mut self, cx: &mut Context<Self>) {
        if self.selection.is_several() {
            self.selection.collapse();
            cx.notify();
        }
    }

    /// The selected notes when there are several, for a change to all of
    /// them; `None` when one note is selected, or none.
    pub fn several_selected(&self) -> Option<Vec<String>> {
        self.selection
            .is_several()
            .then(|| self.selection.ids().to_vec())
    }

    /// Selects the note `offset` rows away from the current one.
    pub fn select_relative(&mut self, offset: isize, cx: &mut Context<Self>) {
        if self.notes.is_empty() {
            return;
        }
        let current = self
            .selected
            .as_ref()
            .and_then(|id| self.notes.iter().position(|n| &n.id == id));
        let next = match current {
            Some(ix) => ix.saturating_add_signed(offset).min(self.notes.len() - 1),
            None => 0,
        };
        let id = self.notes[next].id.clone();
        self.select(Some(id), cx);
    }

    pub fn book(&self, id: &str) -> Option<&Book> {
        self.books.iter().find(|b| b.id == id)
    }

    /// Notes in notebook `id` and every notebook inside it, at any depth.
    pub fn book_total_count(&self, id: &str) -> u32 {
        self.book_counts.get(id).copied().unwrap_or(0)
    }

    pub fn tag(&self, id: &str) -> Option<&Tag> {
        self.tags.iter().find(|t| t.id == id)
    }

    /// The notebook a new note goes into: the selected one, else the first.
    pub fn default_book(&self) -> Option<String> {
        kanso_engine::list::new_note_target(
            &self.filter,
            kanso_engine::list::Narrowing::default(),
            None,
            &self.books,
        )
        .book_id
    }

    /// Notebooks in tree order with their depth.
    pub fn book_tree(&self) -> Vec<(usize, &Book)> {
        book_tree(&self.books)
    }

    /// The notebook `id` and the ones it is inside, outermost first.
    pub fn book_path(&self, id: &str) -> Vec<&Book> {
        book_path(id, &self.books)
    }

    /// Whether notebook `id` is locked: by its own lock, or by one of the
    /// notebooks it is inside.
    pub fn is_book_locked(&self, id: &str) -> bool {
        is_book_locked(id, &self.books)
    }

    /// Whether the notebooks inside `id` hold notes, though it may have
    /// none of its own.
    pub fn has_notes_inside(&self, id: &str) -> bool {
        has_notes_inside(id, &self.books)
    }

    /// The title for an empty place, "No notes in Research" or "No notes
    /// yet", when the list is empty only because all notes or a notebook
    /// has none: loaded, no search, no narrowing. The editor then offers
    /// the first note. A notebook whose notes are all in the notebooks
    /// inside it isn't empty.
    // no-prop-test: a few plain conditions; the note list's
    // `empty_place_cases` covers each.
    pub fn empty_place(&self) -> Option<String> {
        let empty = self.notes_version > 0
            && self.notes.is_empty()
            && self.load_error.is_none()
            && self.search.trim().is_empty()
            && !self.is_refined();
        if !empty {
            return None;
        }
        match &self.filter {
            Filter::All => Some("No notes yet".into()),
            Filter::Book(id) if self.has_notes_inside(id) => None,
            Filter::Book(id) => Some(format!(
                "No notes in {}",
                self.book(id).map_or("this notebook", |b| b.name.as_str())
            )),
            _ => None,
        }
    }

    /// Whether the library has no notes at all, not even in the trash, as
    /// on first run: the editor then offers the first note or an import.
    // no-prop-test: a conjunction of loaded counts; `first_run_cases` in
    // the note list tests covers it.
    pub fn is_empty_library(&self) -> bool {
        self.notes_version > 0
            && self.load_error.is_none()
            && self.counts.all == 0
            && self.counts.trash == 0
    }

    pub fn filter_title(&self) -> String {
        self.title_of(&self.filter)
    }

    /// A place's name, as the list's title says it.
    pub fn title_of(&self, filter: &Filter) -> String {
        match filter {
            Filter::All => "All Notes".into(),
            Filter::Pinned => "Pinned".into(),
            Filter::Recent => "Recent".into(),
            Filter::Book(id) => self.book(id).map_or("Notebook".into(), |b| b.name.clone()),
            Filter::Tag(id) => self
                .tag(id)
                .map_or("Tag".into(), |t| format!("#{}", t.name)),
            Filter::TagPath(path) => format!("#{path}"),
            Filter::Status(status) => status.label().into(),
            Filter::Trash => "Trash".into(),
            Filter::Tasks => "Tasks".into(),
            Filter::OpenLoops => "Open Loops".into(),
            Filter::Overview => "Overview".into(),
            Filter::Shared => "Shared with me".into(),
        }
    }
}
