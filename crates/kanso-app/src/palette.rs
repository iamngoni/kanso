//! The command palette (⌘K) and Open Note (⌘P): a search field over the
//! commands, the places in the library and the notes, drawn over the window
//! with the selected entry explained beside the list.
//!
//! It is part of the main window's view rather than a dialog, and a command
//! is sent from the workspace itself, so it reaches the window's own
//! handlers whatever had the keyboard. Closing it, by Escape, a click
//! outside or a command, gives the keyboard back to whatever had it before,
//! and a command that moves the keyboard on purpose (into the search box, or
//! the title of a new note) does so afterwards. When something else takes
//! the keyboard while it is open, such as a shortcut for a new note, it
//! closes.

use std::sync::Arc;
use std::time::Duration;

use gpui_kit::assets::IconName;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::kbd::Kbd;
use gpui_kit::component::notification::Notification;
use gpui_kit::component::scroll::Scrollbar;
use gpui_kit::component::{ActiveTheme as _, Icon, StyledExt as _, WindowExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_core::{DocType, Note, NoteStatus};
use kanso_engine::navigate::quick_open::{
    HEADING_LIMIT, NOTE_LIMIT, heading_words, merge_by_id, title_search,
};
use kanso_engine::palette::item_score;
use kanso_store::{
    HeadingHit, MEANING_SIMILARITY, MeaningSearch, NoteSummary, Origin, Put, Sort, Store,
    blend_meaning,
};
use serde_json::json;

use crate::book_icons::NotebookIcon;
use crate::commands::{self, Command, Group, Needs, Picker, Run};
use crate::config::ViewMode;
use crate::editor::{NoteEditor, rgb_color};
use crate::fuzzy;
use crate::icon_themes::{self, Slot};
use crate::keys::keycap;
use crate::library::{Filter, Library, RECENT_DAYS, search_params};
use crate::note_batch::Change;
use crate::services::Services;
use crate::text_stats::format_count;
use crate::theme::{
    Colors, PALETTE_SELECTED_ALPHA, Rgb, muted_tag_color, section_label, status_glyph,
};
use crate::word_goal::{self, GoalSetting};
use crate::workspace::AppState;
use crate::zoom::px;

actions!(
    palette,
    [SelectNext, SelectPrevious, Confirm, Dismiss, Back]
);

/// Keys in the palette's search field. They are bound after the field's
/// own keys and at the same depth, so they win: the arrows move through the
/// list rather than the text, Enter chooses, Escape closes, and Backspace in
/// an empty field leaves a second list.
pub fn bind_keys(cx: &mut App) {
    let context = Some("Palette > Input");
    cx.bind_keys([
        KeyBinding::new("down", SelectNext, context),
        KeyBinding::new("up", SelectPrevious, context),
        KeyBinding::new("enter", Confirm, context),
        KeyBinding::new("escape", Dismiss, context),
        KeyBinding::new("backspace", Back, context),
    ]);
}

/// What the palette lists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// ⌘K: commands, places and, once something is typed, notes.
    Commands,
    /// ⌘P: notes, the most recently edited first.
    Notes,
    /// A list a command opened: notebooks, tags or statuses.
    Pick(Picker),
}

pub(crate) const PANEL_WIDTH: f32 = 680.;
const LIST_WIDTH: f32 = 380.;
const SEARCH_HEIGHT: f32 = 60.;
const BODY_HEIGHT: f32 = 380.;
const FOOTER_HEIGHT: f32 = 36.;
/// The panel's height: the parts above and their two dividers.
pub(crate) const PANEL_HEIGHT: f32 = SEARCH_HEIGHT + BODY_HEIGHT + FOOTER_HEIGHT + 2.;
const ROW_HEIGHT: f32 = 30.;
/// How many notes are listed for being about the search, after the notes
/// that have its words.
const MEANING_LIMIT: usize = 5;
/// The shortest search that is also searched by meaning: a letter or two
/// says little about a subject.
const MEANING_MIN_CHARS: usize = 3;
/// How long typing pauses before notes are searched.
const SEARCH_DELAY: Duration = Duration::from_millis(80);

/// What choosing an entry does.
#[derive(Clone)]
enum Target {
    Command(&'static Command),
    /// Show these notes in the note list.
    Show(Filter),
    Note(NoteSummary),
    /// Open a note at one of its headings.
    Heading(HeadingHit),
    /// Move the open note into this notebook.
    Book(String),
    /// Add this tag to the open note, or take it off.
    Tag(String),
    /// Create a tag with this name and add it.
    NewTag(String),
    Status(NoteStatus),
    /// Give the open note this word goal.
    WordGoal(GoalSetting),
    /// Make a note from this template.
    Template(String),
    /// Make a template.
    NewTemplate,
    /// Give the image on the cursor's line this width (`None`: its own).
    ImageWidth(Option<u32>),
    /// Draw the app's icons with the icon theme of this ID.
    IconTheme(String),
    /// Run a plugin's command.
    Plugin(crate::plugins_model::PluginCommand),
}

#[derive(Clone)]
struct Item {
    target: Target,
    label: SharedString,
    icon: IconName,
    /// A status entry's status: its glyph and color stand in for `icon`,
    /// as the note list draws them.
    status: NoteStatus,
    /// Indent, for notebooks inside other notebooks.
    depth: usize,
    /// The current choice: the open note's notebook, tags and status, or the
    /// place the note list shows.
    checked: bool,
    /// Muted text at the end of the row, such as a count.
    hint: Option<SharedString>,
    /// The path of the icon drawn in place of `icon`: a notebook's own, or
    /// a library slot's, which the icon theme draws.
    icon_path: Option<SharedString>,
}

impl Item {
    fn new(target: Target, label: impl Into<SharedString>, icon: IconName) -> Self {
        Item {
            target,
            label: label.into(),
            icon,
            status: NoteStatus::None,
            depth: 0,
            checked: false,
            hint: None,
            icon_path: None,
        }
    }

    /// An entry for a notebook, with the notebook's own icon.
    fn notebook(target: Target, label: impl Into<SharedString>, icon: NotebookIcon) -> Self {
        Item {
            icon_path: Some(icon.path()),
            ..Item::new(target, label, crate::book_icons::DEFAULT)
        }
    }

    /// An entry drawn with the library's icon for `slot`.
    fn slot(target: Target, label: impl Into<SharedString>, slot: Slot) -> Self {
        Item {
            icon_path: Some(icon_themes::slot_path(slot)),
            ..Item::new(target, label, crate::book_icons::DEFAULT)
        }
    }

    fn depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    fn status(mut self, status: NoteStatus) -> Self {
        self.status = status;
        self
    }

    /// The entry's icon and its color, `quiet` unless it is a status's.
    fn glyph(&self, colors: &Colors, quiet: Rgb) -> (IconName, Rgb) {
        status_glyph(self.status, colors, quiet).unwrap_or((self.icon, quiet))
    }

    fn hint(mut self, hint: impl Into<SharedString>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Identifies the entry across rebuilds, to keep it selected.
    fn key(&self) -> String {
        match &self.target {
            Target::Command(command) => format!("command:{}", command.id),
            Target::Show(filter) => format!("show:{filter:?}"),
            Target::Note(note) => format!("note:{}", note.id),
            Target::Heading(hit) => format!("heading:{}:{}", hit.note_id, hit.line),
            Target::Book(id) => format!("book:{id}"),
            Target::Tag(id) => format!("tag:{id}"),
            Target::NewTag(name) => format!("new-tag:{name}"),
            Target::Status(status) => format!("status:{}", status.as_str()),
            Target::WordGoal(goal) => format!("goal:{goal:?}"),
            Target::Template(id) => format!("template:{id}"),
            Target::NewTemplate => "new-template".to_owned(),
            Target::ImageWidth(width) => format!("image-width:{width:?}"),
            Target::IconTheme(id) => format!("icon-theme:{id}"),
            Target::Plugin(command) => format!("plugin:{}/{}", command.plugin_id, command.spec.id),
        }
    }
}

/// A heading and its entries, each with how well it fits the query.
struct Section {
    heading: SharedString,
    entries: Vec<(Item, Option<i64>)>,
    /// Keep entries that don't fit the query, after the ones that do: notes
    /// the store found by their text rather than their title.
    keep_unmatched: bool,
}

enum Row {
    Heading(SharedString),
    /// An index into `Palette::items`.
    Item(usize),
    Separator,
}

pub struct Palette {
    store: Arc<Store>,
    library: Entity<Library>,
    editor: Entity<NoteEditor>,
    /// The workspace's own focus, always drawn: commands are sent from
    /// here, and the keyboard goes here when nothing else should have it.
    home: FocusHandle,
    /// The note list's focus, for opening a note while the editor shows
    /// only its preview.
    list_focus: FocusHandle,
    input: Entity<InputState>,
    /// `None` while closed.
    mode: Option<Mode>,
    /// Whether the second list was opened from the commands, so Escape goes
    /// back to them. The editor's header opens one straight away; Escape
    /// closes that.
    from_commands: bool,
    /// What had the keyboard before the palette opened.
    restore_focus: Option<FocusHandle>,
    /// The note that was open when the palette opened.
    note: Option<Note>,
    /// The notes selected in the list, when there are several: moving,
    /// tagging, pinning, setting the status and trashing act on all of them.
    several: Vec<NoteSummary>,
    /// Whether the open note is out of sight (behind Settings): the palette
    /// then offers nothing that acts on it.
    notes_hidden: bool,
    items: Vec<Item>,
    rows: Vec<Row>,
    selected: Option<usize>,
    /// The selected entry, kept while results change under it once the
    /// keyboard or pointer chose it.
    chosen_key: Option<String>,
    /// Notes found for `notes_for`.
    notes: Vec<NoteSummary>,
    /// Headings found for `notes_for`, in ⌘P.
    headings: Vec<HeadingHit>,
    /// Notes about `notes_for` that don't have its words, each with how
    /// close it is, while related notes are on.
    about: Vec<(NoteSummary, f32)>,
    notes_for: Option<String>,
    search_task: Option<Task<()>>,
    scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl Palette {
    pub fn new(
        store: Arc<Store>,
        library: Entity<Library>,
        editor: Entity<NoteEditor>,
        home: FocusHandle,
        list_focus: FocusHandle,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let input = cx.new(|cx| InputState::new(window, cx));
        let subscriptions = vec![
            cx.subscribe_in(&input, window, |this, _, event, window, cx| match event {
                InputEvent::Change => this.query_changed(cx),
                // Something else took the keyboard, a shortcut for a new
                // note or the search box say: the palette gives way rather
                // than cover what is now being typed into. Switching apps
                // keeps it.
                InputEvent::Blur if this.mode.is_some() && window.is_window_active() => {
                    this.close(window, cx);
                }
                _ => {}
            }),
            // Tag suggestions for the open note arrive in the background.
            cx.observe(&editor.read(cx).meaning().clone(), |this, _, cx| {
                if this.mode == Some(Mode::Pick(Picker::Tag)) {
                    this.rebuild(cx);
                    cx.notify();
                }
            }),
            // Notebooks and tags can change while the palette is open, by
            // sync or the API.
            cx.observe(&library, |this, _, cx| {
                if this.mode.is_some() {
                    this.note = this.open_note(cx);
                    this.several = this.several_notes(cx);
                    this.rebuild(cx);
                }
            }),
        ];
        Palette {
            store,
            library,
            editor,
            home,
            list_focus,
            input,
            mode: None,
            from_commands: false,
            restore_focus: None,
            note: None,
            several: Vec::new(),
            notes_hidden: false,
            items: Vec::new(),
            rows: Vec::new(),
            selected: None,
            chosen_key: None,
            notes: Vec::new(),
            headings: Vec::new(),
            about: Vec::new(),
            notes_for: None,
            search_task: None,
            scroll: ScrollHandle::new(),
            _subscriptions: subscriptions,
        }
    }

    #[cfg(test)]
    pub fn is_open(&self) -> bool {
        self.mode.is_some()
    }

    #[cfg(test)]
    pub fn mode(&self) -> Option<Mode> {
        self.mode
    }

    /// Opens the palette in `mode`, or closes it when it already shows that,
    /// so the shortcut toggles it.
    pub fn open(&mut self, mode: Mode, window: &mut Window, cx: &mut Context<Self>) {
        if self.mode == Some(mode) {
            return self.close(window, cx);
        }
        if self.mode.is_none() {
            self.restore_focus = window.focused(cx);
            self.note = self.open_note(cx);
            self.several = self.several_notes(cx);
        }
        self.from_commands = false;
        self.show(mode, window, cx);
    }

    /// Whether Escape, or Backspace in the empty field, goes back to the
    /// commands rather than closing.
    fn leads_back(&self) -> bool {
        matches!(self.mode, Some(Mode::Pick(_))) && self.from_commands
    }

    /// Switches to `mode` with an empty field.
    fn show(&mut self, mode: Mode, window: &mut Window, cx: &mut Context<Self>) {
        self.mode = Some(mode);
        self.chosen_key = None;
        self.input.update(cx, |input, cx| {
            input.set_value("", window, cx);
            input.set_placeholder(placeholder(mode), window, cx);
            input.focus(window, cx);
        });
        self.search_notes(cx);
        self.rebuild(cx);
        cx.notify();
    }

    /// Closes the palette. The keyboard goes back to what had it before,
    /// unless something else took it meanwhile.
    pub fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.mode.take().is_none() {
            return;
        }
        self.search_task = None;
        self.notes.clear();
        self.about.clear();
        self.notes_for = None;
        self.items.clear();
        self.rows.clear();
        self.selected = None;
        let restore = self.restore_focus.take();
        if self.input.read(cx).focus_handle(cx).is_focused(window) {
            // A handle that is no longer drawn gives way to the workspace
            // once the next frame shows it missing.
            window.focus(restore.as_ref().unwrap_or(&self.home), cx);
        }
        cx.notify();
    }

    /// Selects the first note in the list, for pictures of the detail pane.
    #[cfg(test)]
    pub fn select_first_note(&mut self, cx: &mut Context<Self>) {
        if let Some(ix) = self
            .items
            .iter()
            .position(|item| matches!(item.target, Target::Note(_)))
        {
            self.select(ix, true, cx);
        }
    }

    /// Selects the first heading listed, so the detail pane shows it.
    #[cfg(test)]
    pub fn select_first_heading(&mut self, cx: &mut Context<Self>) {
        if let Some(ix) = self
            .items
            .iter()
            .position(|item| matches!(item.target, Target::Heading(_)))
        {
            self.select(ix, true, cx);
        }
    }

    /// The labels of the entries listed now.
    #[cfg(test)]
    pub fn item_labels(&self) -> Vec<String> {
        self.items
            .iter()
            .map(|item| item.label.to_string())
            .collect()
    }

    /// The entries listed now with the path of the icon each draws (tests).
    #[cfg(test)]
    pub fn item_icons(&self) -> Vec<(String, SharedString)> {
        self.items
            .iter()
            .map(|item| {
                let path = item
                    .icon_path
                    .clone()
                    .unwrap_or_else(|| gpui_kit::component::IconNamed::path(item.icon));
                (item.label.to_string(), path)
            })
            .collect()
    }

    /// Types `query` into the field, as the user would.
    #[cfg(test)]
    pub fn set_query(&mut self, query: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.input
            .update(cx, |input, cx| input.replace_all(query, window, cx));
    }

    fn query(&self, cx: &App) -> String {
        self.input.read(cx).value().trim().to_owned()
    }

    fn query_changed(&mut self, cx: &mut Context<Self>) {
        if self.mode.is_none() {
            return;
        }
        self.chosen_key = None;
        self.search_notes(cx);
        self.rebuild(cx);
        cx.notify();
    }

    /// The open note is out of sight, or back in view.
    pub fn set_notes_hidden(&mut self, hidden: bool) {
        self.notes_hidden = hidden;
    }

    fn several_notes(&self, cx: &App) -> Vec<NoteSummary> {
        let library = self.library.read(cx);
        match library.several_selected() {
            Some(ids) if !self.notes_hidden => library
                .notes
                .iter()
                .filter(|n| ids.contains(&n.id))
                .cloned()
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Makes `change` to the several selected notes, when there are; says
    /// whether it did.
    fn change_several(
        &mut self,
        change: Change,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.several.is_empty() {
            return false;
        }
        let ids: Vec<String> = self.several.iter().map(|n| n.id.clone()).collect();
        crate::note_batch::apply(&ids, change, &self.library, window, cx);
        true
    }

    /// What every one of the several selected notes has, by `of`: the
    /// value they share, or `None` when they differ.
    fn shared_by_several<T: PartialEq>(&self, of: impl Fn(&NoteSummary) -> T) -> Option<T> {
        let mut values = self.several.iter().map(of);
        let first = values.next()?;
        values.all(|v| v == first).then_some(first)
    }

    fn open_note(&self, cx: &App) -> Option<Note> {
        if self.notes_hidden {
            return None;
        }
        let id = self.editor.read(cx).note_id()?;
        self.store.note(id).ok().flatten()
    }

    /// Searches the store for notes when the mode lists them, in the
    /// background: notes whose titles have the words first, then notes that
    /// have them anywhere, and, with related notes on, notes about the
    /// words that don't have them. Results for another query never replace
    /// the current one's.
    fn search_notes(&mut self, cx: &mut Context<Self>) {
        let query = self.query(cx);
        let wanted = match self.mode {
            Some(Mode::Notes) => true,
            Some(Mode::Commands) => !query.is_empty(),
            _ => false,
        };
        if !wanted {
            self.search_task = None;
            self.notes.clear();
            self.headings.clear();
            self.about.clear();
            self.notes_for = None;
            return;
        }
        let heading_query = heading_query(self.mode, &query);
        let meaning_query = meaning_query(&query);
        if self.notes_for.as_deref() == Some(query.as_str()) {
            // Back to the query already shown: a search for another one
            // still on its way would replace it.
            self.search_task = None;
            return;
        }
        let store = self.store.clone();
        self.search_task = Some(cx.spawn(async move |this, cx| {
            if !query.is_empty() {
                cx.background_executor().timer(SEARCH_DELAY).await;
            }
            let mut params = search_params(&Filter::All, &query, Sort::UpdatedAt, true);
            params.limit = Some(NOTE_LIMIT);
            let by_title = title_search(&params);
            let found = cx
                .background_executor()
                .spawn(async move {
                    let titled = match by_title {
                        Some(params) => store.search_summaries(&params)?,
                        None => Vec::new(),
                    };
                    let anywhere = store.search_summaries(&params)?;
                    let headings = match heading_query {
                        Some(words) => store.search_headings(&words, HEADING_LIMIT)?,
                        None => Vec::new(),
                    };
                    let notes = merge_by_id(titled, anywhere, NOTE_LIMIT);
                    let about = match meaning_query {
                        Some(words) => about_notes(&store, &words, &notes)?,
                        None => Vec::new(),
                    };
                    Ok::<_, kanso_store::StoreError>((notes, headings, about))
                })
                .await;
            this.update(cx, |this, cx| {
                if this.mode.is_none() || this.query(cx) != query {
                    return;
                }
                // A query the store can't run finds nothing.
                let (notes, headings, about) = found.unwrap_or_default();
                this.notes = notes;
                this.headings = headings;
                this.about = about;
                this.notes_for = Some(query);
                this.rebuild(cx);
                cx.notify();
            })
            .ok();
        }));
    }

    /// Whether notes for the current query are still on their way.
    fn searching(&self, cx: &App) -> bool {
        self.search_task.is_some() && self.notes_for.as_deref() != Some(self.query(cx).as_str())
    }

    // MARK: Entries

    /// The entries for the current mode and query, before ranking.
    fn sections(&self, query: &str, cx: &App) -> Vec<Section> {
        let section = |heading: &str, entries, keep_unmatched| Section {
            heading: SharedString::from(heading.to_owned()),
            entries,
            keep_unmatched,
        };
        let fit = |item: Item, keywords: &[&str]| {
            let score = item_score(query, &item.label, keywords);
            (item, score)
        };
        match self.mode {
            None => Vec::new(),
            Some(Mode::Commands) => {
                let mut sections: Vec<Section> = [Group::Note, Group::View]
                    .into_iter()
                    .map(|group| section(group.label(), self.commands(group, query, cx), false))
                    .collect();
                let mut places = self.places(query, cx);
                places.extend(self.commands(Group::GoTo, query, cx));
                sections.push(section(Group::GoTo.label(), places, false));
                sections.push(section(
                    Group::App.label(),
                    self.commands(Group::App, query, cx),
                    false,
                ));
                sections.push(section("Plugins", self.plugin_commands(query, cx), false));
                if !query.is_empty() {
                    sections.push(section("Notes", self.note_entries(query, cx), true));
                    sections.push(section(ABOUT_HEADING, self.about_entries(query), true));
                }
                sections
            }
            Some(Mode::Notes) => {
                let heading = if query.is_empty() {
                    "Recently edited"
                } else {
                    "Notes"
                };
                let mut sections = vec![section(heading, self.note_entries(query, cx), true)];
                let headings = self.heading_entries(query);
                if !headings.is_empty() {
                    sections.push(section("Headings", headings, true));
                }
                sections.push(section(ABOUT_HEADING, self.about_entries(query), true));
                sections
            }
            Some(Mode::Pick(Picker::Notebook)) => {
                let library = self.library.read(cx);
                let shared_book = self.shared_by_several(|n| n.book_id.clone());
                let current = if self.several.is_empty() {
                    self.note.as_ref().map(|n| n.book_id.as_str())
                } else {
                    shared_book.as_deref()
                };
                let entries = library
                    .book_tree()
                    .into_iter()
                    .map(|(depth, book)| {
                        let item = Item::notebook(
                            Target::Book(book.id.clone()),
                            book.name.clone(),
                            crate::book_icons::shown(
                                crate::book_icons::book_icon(book, cx),
                                library.is_book_locked(&book.id),
                                crate::locked::LockKeys::is_unlocked(cx),
                            ),
                        )
                        // Typing flattens the tree into a ranked list.
                        .depth(if query.is_empty() { depth } else { 0 })
                        .checked(current == Some(book.id.as_str()))
                        .hint(book.count.to_string());
                        fit(item, &[])
                    })
                    .collect();
                vec![section("Move to notebook", entries, false)]
            }
            Some(Mode::Pick(Picker::GoToNotebook)) => {
                let library = self.library.read(cx);
                let showing = &library.filter;
                let entries = library
                    .book_tree()
                    .into_iter()
                    .map(|(depth, book)| {
                        let filter = Filter::Book(book.id.clone());
                        let item = Item::notebook(
                            Target::Show(filter.clone()),
                            book.name.clone(),
                            crate::book_icons::shown(
                                crate::book_icons::book_icon(book, cx),
                                library.is_book_locked(&book.id),
                                crate::locked::LockKeys::is_unlocked(cx),
                            ),
                        )
                        .depth(if query.is_empty() { depth } else { 0 })
                        .checked(*showing == filter)
                        .hint(book.count.to_string());
                        fit(item, &[])
                    })
                    .collect();
                vec![section("Notebooks", entries, false)]
            }
            Some(Mode::Pick(Picker::GoToTag)) => {
                let library = self.library.read(cx);
                let showing = &library.filter;
                let mut tags: Vec<_> = library.tags.iter().collect();
                tags.sort_by_key(|t| t.name.to_lowercase());
                let name = tag_name(query);
                let entries = tags
                    .iter()
                    .map(|tag| {
                        let filter = Filter::Tag(tag.id.clone());
                        let item =
                            Item::slot(Target::Show(filter.clone()), tag.name.clone(), Slot::Tag)
                                .checked(*showing == filter)
                                .hint(tag.count.to_string());
                        let score = item_score(name, &item.label, &[]);
                        (item, score)
                    })
                    .collect();
                vec![section("Tags", entries, false)]
            }
            Some(Mode::Pick(Picker::Tag)) => {
                let library = self.library.read(cx);
                // With several notes, a tag is checked when they all have it.
                let on_all: Vec<String> = match self.several.first() {
                    Some(first) => first
                        .tags
                        .iter()
                        .filter(|t| self.several.iter().all(|n| n.tags.contains(t)))
                        .cloned()
                        .collect(),
                    None => Vec::new(),
                };
                let on_note = if self.several.is_empty() {
                    self.note.as_ref().map_or(&[][..], |n| &n.tags[..])
                } else {
                    &on_all[..]
                };
                // Before anything is typed, the tags close notes have lead.
                let suggested: Vec<_> = if query.is_empty() && self.note.is_some() {
                    self.editor
                        .read(cx)
                        .meaning()
                        .read(cx)
                        .suggested_tags(on_note)
                        .into_iter()
                        .filter_map(|s| library.tag(&s.tag_id).cloned())
                        .collect()
                } else {
                    Vec::new()
                };
                let mut tags: Vec<_> = library
                    .tags
                    .iter()
                    .filter(|t| !suggested.iter().any(|s| s.id == t.id))
                    .collect();
                tags.sort_by_key(|t| t.name.to_lowercase());
                // Tags show as "#mcp", so "#mcp" typed finds "mcp".
                let name = tag_name(query);
                let mut entries: Vec<_> = tags
                    .iter()
                    .map(|tag| {
                        let item =
                            Item::slot(Target::Tag(tag.id.clone()), tag.name.clone(), Slot::Tag)
                                .checked(on_note.contains(&tag.id))
                                .hint(tag.count.to_string());
                        let score = item_score(name, &item.label, &[]);
                        (item, score)
                    })
                    .collect();
                let exists = tags.iter().any(|t| t.name.eq_ignore_ascii_case(name));
                if !name.is_empty() && !exists {
                    let item = Item::new(
                        Target::NewTag(name.to_owned()),
                        format!("Create tag “{name}”"),
                        IconName::Plus,
                    );
                    // Last, but always there.
                    entries.push((item, Some(i64::MIN / 8)));
                }
                let mut sections = Vec::new();
                if !suggested.is_empty() {
                    let entries = suggested
                        .iter()
                        .map(|tag| {
                            let item = Item::new(
                                Target::Tag(tag.id.clone()),
                                tag.name.clone(),
                                IconName::Sparkles,
                            )
                            .hint(tag.count.to_string());
                            (item, None)
                        })
                        .collect();
                    sections.push(section(SUGGESTED_HEADING, entries, true));
                }
                sections.push(section("Tags", entries, false));
                sections
            }
            Some(Mode::Pick(Picker::Status)) => {
                let current = if self.several.is_empty() {
                    self.note.as_ref().map(|n| n.status)
                } else {
                    self.shared_by_several(|n| n.status)
                };
                let entries = NoteStatus::ALL
                    .into_iter()
                    .map(|status| {
                        let item = Item::new(
                            Target::Status(status),
                            status_label(status),
                            IconName::Minus,
                        )
                        .status(status)
                        .checked(current == Some(status));
                        fit(item, &[])
                    })
                    .collect();
                vec![section("Status", entries, false)]
            }
            Some(Mode::Pick(Picker::Template)) => {
                let mut entries: Vec<_> = self
                    .store
                    .templates()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(id, title)| {
                        let label = if title.trim().is_empty() {
                            "Untitled".to_owned()
                        } else {
                            title
                        };
                        fit(
                            Item::new(Target::Template(id), label, IconName::LayoutTemplate),
                            &[],
                        )
                    })
                    .collect();
                let item = Item::new(Target::NewTemplate, "New template", IconName::Plus);
                // Last, but always there.
                entries.push((item, Some(i64::MIN / 8)));
                vec![section("Templates", entries, false)]
            }
            Some(Mode::Pick(Picker::WordGoal)) => {
                let editor = self.editor.read(cx);
                let (current, default) = (editor.word_goal(), editor.default_word_goal());
                vec![section(
                    "Word goal",
                    goal_entries(query, current, default),
                    true,
                )]
            }
            Some(Mode::Pick(Picker::IconTheme)) => {
                let current = icon_themes::active(cx).id.clone();
                let entries = cx
                    .try_global::<icon_themes::IconThemes>()
                    .map(|themes| themes.entries.clone())
                    .unwrap_or_default()
                    .iter()
                    .map(|theme| {
                        let item = Item::new(
                            Target::IconTheme(theme.id.clone()),
                            theme.name.clone(),
                            IconName::Shapes,
                        )
                        .checked(theme.id == current)
                        .hint(if theme.path.is_some() {
                            "Your folder"
                        } else {
                            "Built in"
                        });
                        let score = item_score(query, &item.label, &[theme.id.as_str()]);
                        (item, score)
                    })
                    .collect();
                vec![section("Icon themes", entries, false)]
            }
            Some(Mode::Pick(Picker::ImageSize)) => {
                let current = self
                    .editor
                    .read(cx)
                    .image_on_caret_line(cx)
                    .map(|image| image.width);
                vec![section(
                    "Image width",
                    image_width_entries(query, current),
                    true,
                )]
            }
        }
    }

    /// The registry's commands in `group` that apply now.
    fn commands(&self, group: Group, query: &str, cx: &App) -> Vec<(Item, Option<i64>)> {
        let note = self.note.as_ref();
        let signed_in = cx
            .try_global::<Services>()
            .is_some_and(|services| services.signed_in());
        let signed_in_locked = cx
            .try_global::<Services>()
            .is_some_and(Services::has_locked_session);
        let in_trash = note.is_some_and(|n| n.book_id == TRASH_BOOK_ID);
        let image_line = !in_trash && self.editor.read(cx).image_on_caret_line(cx).is_some();
        let audio_line = !in_trash && self.editor.read(cx).audio_on_caret_line(cx).is_some();
        let shared_note = self.editor.read(cx).shared_role().is_some();
        let ai_on = cx
            .try_global::<AppState>()
            .is_some_and(|state| state.config.ai.enabled);
        commands::all()
            .iter()
            .filter(|command| command.group == group)
            .filter(|command| match command.needs {
                Needs::Nothing => true,
                Needs::Note => note.is_some(),
                Needs::LiveNote => note.is_some() && !in_trash,
                Needs::UnlockedNote => note.is_some_and(|n| !n.is_locked()) && !in_trash,
                Needs::LockedNote => note.is_some_and(|n| n.is_locked()),
                Needs::Settings => self.notes_hidden,
                Needs::SignedIn => signed_in,
                Needs::SignedOut => !signed_in && !signed_in_locked,
                Needs::ImageLine => image_line,
                Needs::AudioLine => audio_line,
                Needs::SharedNote => shared_note,
                Needs::AiOn => ai_on,
                Needs::AiNote => ai_on && note.is_some_and(|n| !n.is_locked()) && !in_trash,
            })
            .map(|command| {
                let pinned = match self.shared_by_several(|n| n.pinned) {
                    _ if self.several.is_empty() => note.is_some_and(|n| n.pinned),
                    all_pinned => all_pinned == Some(true),
                };
                let (label, icon) = match command.label_when_pinned {
                    Some(set) if pinned => set,
                    _ => (command.label, command.icon),
                };
                let label = several_label(command.id, label, self.several.len());
                let score = item_score(query, &label, command.keywords);
                (Item::new(Target::Command(command), label, icon), score)
            })
            .collect()
    }

    /// The plugins' commands that apply now: those for an open note only
    /// with a live, unlocked one, and those for a selection only with text
    /// selected.
    fn plugin_commands(&self, query: &str, cx: &App) -> Vec<(Item, Option<i64>)> {
        use kanso_plugins::protocol::When;
        let note = self.note.as_ref();
        let note_open = note.is_some_and(|n| !n.is_locked() && n.book_id != TRASH_BOOK_ID);
        let selection = note_open && self.editor.read(cx).plugin_selection(cx).is_some();
        crate::plugins::Plugins::commands(cx)
            .into_iter()
            .filter(|command| match command.spec.when {
                When::Always => true,
                When::NoteOpen => note_open,
                When::Selection => selection,
            })
            .map(|command| {
                let mut keywords: Vec<&str> =
                    command.spec.keywords.iter().map(String::as_str).collect();
                keywords.push(&command.plugin_name);
                let score = item_score(query, &command.spec.title, &keywords);
                let label = command.spec.title.clone();
                (
                    Item::new(Target::Plugin(command), label, IconName::Puzzle),
                    score,
                )
            })
            .collect()
    }

    /// The places the note list can show: everything, pinned and recent
    /// notes, each notebook and status, and the trash.
    fn places(&self, query: &str, cx: &App) -> Vec<(Item, Option<i64>)> {
        let library = self.library.read(cx);
        let counts = &library.counts;
        let showing = &library.filter;
        let place =
            |filter: Filter, label: String, icon: NotebookIcon, count: u32, keywords: &[&str]| {
                let score = item_score(query, &label, keywords);
                let item = Item::notebook(Target::Show(filter.clone()), label, icon)
                    .checked(*showing == filter)
                    .hint(count.to_string());
                (item, score)
            };
        let mut places = vec![
            place(
                Filter::All,
                "All Notes".into(),
                NotebookIcon::Slot(Slot::AllNotes),
                counts.all,
                &["everything", "library"],
            ),
            place(
                Filter::Pinned,
                "Pinned".into(),
                NotebookIcon::Slot(Slot::Pinned),
                counts.pinned,
                &["starred"],
            ),
            place(
                Filter::Recent,
                "Recent".into(),
                NotebookIcon::Slot(Slot::Recent),
                library.recent,
                &["latest", "edited", "week"],
            ),
            place(
                Filter::Tasks,
                "Tasks".into(),
                NotebookIcon::Slot(Slot::Tasks),
                library.overdue,
                &["todo", "to do", "due", "overdue", "checklist"],
            ),
        ];
        for (depth, book) in library.book_tree() {
            let (item, score) = place(
                Filter::Book(book.id.clone()),
                book.name.clone(),
                crate::book_icons::shown(
                    crate::book_icons::book_icon(book, cx),
                    library.is_book_locked(&book.id),
                    crate::locked::LockKeys::is_unlocked(cx),
                ),
                book.count,
                &[],
            );
            places.push((item.depth(if query.is_empty() { depth } else { 0 }), score));
        }
        for status in NoteStatus::ALL
            .into_iter()
            .filter(|s| *s != NoteStatus::None)
        {
            let count = counts.by_status.get(&status).copied().unwrap_or(0);
            let (item, score) = place(
                Filter::Status(status),
                status.label().into(),
                NotebookIcon::Choice(IconName::Minus),
                count,
                &["status"],
            );
            places.push((item.status(status), score));
        }
        if library.shared > 0 || cx.try_global::<Services>().is_some_and(Services::signed_in) {
            places.push(place(
                Filter::Shared,
                "Shared with me".into(),
                NotebookIcon::Slot(Slot::Shared),
                library.shared,
                &["shared", "collaboration", "invitations", "others"],
            ));
        }
        places.push(place(
            Filter::Trash,
            "Trash".into(),
            NotebookIcon::Slot(Slot::Trash),
            counts.trash,
            &["deleted"],
        ));
        places
    }

    /// Notes from the store. Until the notes for the current query arrive,
    /// the previous ones stay if their title still fits, so the list doesn't
    /// flicker while typing.
    fn note_entries(&self, query: &str, cx: &App) -> Vec<(Item, Option<i64>)> {
        let current = self.notes_for.as_deref() == Some(query);
        let open = self.note.as_ref().map(|n| n.id.as_str());
        let library = self.library.read(cx);
        self.notes
            .iter()
            .map(|note| {
                let title = note_title(&note.title);
                let score = fuzzy::score(query, &title);
                let mut item = Item::slot(Target::Note(note.clone()), title, Slot::Note)
                    .checked(open == Some(note.id.as_str()));
                if let Some(book) = library.book(&note.book_id) {
                    item = item.hint(book.name.clone());
                }
                (item, score)
            })
            .filter(|(_, score)| current || score.is_some())
            .collect()
    }

    /// Notes about the query that don't have its words, closest first, each
    /// saying how close it is. They go after every match, unranked by the
    /// letters typed: their titles needn't have them.
    fn about_entries(&self, query: &str) -> Vec<(Item, Option<i64>)> {
        if self.notes_for.as_deref() != Some(query) {
            return Vec::new();
        }
        let open = self.note.as_ref().map(|n| n.id.as_str());
        self.about
            .iter()
            .map(|(note, score)| {
                let item = Item::new(
                    Target::Note(note.clone()),
                    note_title(&note.title),
                    IconName::Sparkles,
                )
                .checked(open == Some(note.id.as_str()))
                .hint(crate::related::similarity_label(*score));
                (item, None)
            })
            .collect()
    }

    /// Headings found for the query, each with the note it is in.
    fn heading_entries(&self, query: &str) -> Vec<(Item, Option<i64>)> {
        if self.notes_for.as_deref() != Some(query) {
            return Vec::new();
        }
        self.headings
            .iter()
            .map(|hit| {
                let score = fuzzy::score(query, &hit.text);
                let item = Item::new(
                    Target::Heading(hit.clone()),
                    hit.text.clone(),
                    IconName::Heading,
                )
                .hint(note_title(&hit.note_title));
                (item, score)
            })
            .collect()
    }

    /// Ranks the entries for the query and lays out the rows: with a query,
    /// sections are ordered by their best entry, so the best fit comes first.
    fn rebuild(&mut self, cx: &mut Context<Self>) {
        let query = self.query(cx);
        let mut sections: Vec<(SharedString, Vec<Item>, Option<i64>)> = Vec::new();
        for section in self.sections(&query, cx) {
            let scores: Vec<Option<i64>> = section.entries.iter().map(|(_, s)| *s).collect();
            let mut order = if query.is_empty() {
                (0..scores.len()).collect()
            } else {
                fuzzy::rank_scores(&scores)
            };
            if section.keep_unmatched {
                order.extend((0..scores.len()).filter(|&i| scores[i].is_none()));
            }
            let best = order.first().and_then(|&i| scores[i]);
            let mut entries: Vec<Option<Item>> = section
                .entries
                .into_iter()
                .map(|(item, _)| Some(item))
                .collect();
            let items: Vec<Item> = order
                .into_iter()
                .filter_map(|i| entries[i].take())
                .collect();
            if !items.is_empty() {
                sections.push((section.heading, items, best));
            }
        }
        if !query.is_empty() {
            // Stable, so equally good sections keep their order.
            sections.sort_by_key(|(_, _, best)| std::cmp::Reverse(*best));
        }

        self.items.clear();
        self.rows.clear();
        for (heading, items, _) in sections {
            if !self.rows.is_empty() {
                self.rows.push(Row::Separator);
            }
            self.rows.push(Row::Heading(heading));
            for item in items {
                self.rows.push(Row::Item(self.items.len()));
                self.items.push(item);
            }
        }
        let kept = self
            .chosen_key
            .as_ref()
            .and_then(|key| self.items.iter().position(|item| &item.key() == key));
        // Listing recent notes, the open one comes first; start on the one
        // before it, so ⌘P and Enter go back to the previous note.
        // The editor's note, even behind Settings, where `note` is none.
        let open = self.editor.read(cx).note_id();
        let open_first = self.mode == Some(Mode::Notes)
            && query.is_empty()
            && self.items.len() > 1
            && matches!(
                (&self.items[0].target, open),
                (Target::Note(first), Some(open)) if first.id == open
            );
        let first = if open_first { 1 } else { 0 };
        self.selected = kept.or((!self.items.is_empty()).then_some(first));
        if kept.is_none() {
            self.scroll.scroll_to_item(0);
        }
    }

    // MARK: Actions

    fn select(&mut self, ix: usize, scroll: bool, cx: &mut Context<Self>) {
        let Some(item) = self.items.get(ix) else {
            return;
        };
        if self.selected == Some(ix) {
            return;
        }
        self.selected = Some(ix);
        self.chosen_key = Some(item.key());
        if scroll
            && let Some(row) = self
                .rows
                .iter()
                .position(|row| matches!(row, Row::Item(i) if *i == ix))
        {
            // Bring the heading into view with the first entry.
            self.scroll.scroll_to_item(if ix == 0 { 0 } else { row });
        }
        cx.notify();
    }

    fn select_by(&mut self, step: isize, cx: &mut Context<Self>) {
        let len = self.items.len();
        if len == 0 {
            return;
        }
        let next = match self.selected {
            Some(ix) => (ix as isize + step).rem_euclid(len as isize) as usize,
            None => 0,
        };
        self.select(next, true, cx);
    }

    fn select_next(&mut self, _: &SelectNext, _: &mut Window, cx: &mut Context<Self>) {
        self.select_by(1, cx);
    }

    fn select_previous(&mut self, _: &SelectPrevious, _: &mut Window, cx: &mut Context<Self>) {
        self.select_by(-1, cx);
    }

    fn confirm(&mut self, _: &Confirm, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(ix) = self.selected {
            self.choose(ix, window, cx);
        }
    }

    /// Escape leaves a second list for the commands it came from, and
    /// otherwise closes.
    fn dismiss(&mut self, _: &Dismiss, window: &mut Window, cx: &mut Context<Self>) {
        if self.leads_back() {
            self.show(Mode::Commands, window, cx);
        } else {
            self.close(window, cx);
        }
    }

    /// Backspace in an empty field leaves a second list for the commands it
    /// came from; otherwise it deletes as usual.
    fn back(&mut self, _: &Back, window: &mut Window, cx: &mut Context<Self>) {
        if self.leads_back() && self.input.read(cx).value().is_empty() {
            self.show(Mode::Commands, window, cx);
        } else {
            cx.propagate();
        }
    }

    /// Chooses entry `ix`, as Enter on it would.
    #[cfg(test)]
    pub fn choose_for_test(&mut self, ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.choose(ix, window, cx);
    }

    /// Does what entry `ix` stands for.
    fn choose(&mut self, ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(item) = self.items.get(ix).cloned() else {
            return;
        };
        match item.target {
            Target::Command(command) => match command.run {
                Run::Pick(picker) => {
                    self.from_commands = true;
                    self.show(Mode::Pick(picker), window, cx);
                }
                Run::Action(action) => {
                    self.close(window, cx);
                    // From the workspace, which is always drawn, the action
                    // finds the window's handlers whatever has the keyboard;
                    // after this one, so a handler can open the palette.
                    let home = self.home.clone();
                    let action = action();
                    window.defer(cx, move |window, cx| {
                        home.dispatch_action(action.as_ref(), window, cx);
                    });
                }
            },
            Target::Show(filter) => {
                self.close(window, cx);
                self.library
                    .update(cx, |library, cx| library.set_filter(filter, cx));
            }
            Target::Note(note) => {
                self.close(window, cx);
                self.library
                    .update(cx, |library, cx| library.select(Some(note.id), cx));
                // The preview alone has no text to type into; the list keeps
                // the keyboard then.
                if self.editor.read(cx).view_mode == ViewMode::Preview {
                    window.focus(&self.list_focus, cx);
                } else {
                    self.editor
                        .update(cx, |editor, cx| editor.focus_body(window, cx));
                }
            }
            Target::Heading(hit) => {
                self.close(window, cx);
                let open = self.editor.read(cx).note_id() == Some(hit.note_id.as_str());
                if open {
                    self.editor
                        .update(cx, |editor, cx| editor.reveal_line(hit.line, window, cx));
                } else {
                    self.editor.update(cx, |editor, _| {
                        editor.reveal_after_open(hit.note_id.clone(), hit.line)
                    });
                    self.library
                        .update(cx, |library, cx| library.select(Some(hit.note_id), cx));
                }
                if self.editor.read(cx).view_mode == ViewMode::Preview {
                    window.focus(&self.list_focus, cx);
                }
            }
            Target::Book(id) => {
                self.close(window, cx);
                if !self.change_several(Change::Move(id.clone()), window, cx) {
                    self.editor.update(cx, |editor, cx| editor.set_book(id, cx));
                }
            }
            Target::Tag(id) => {
                self.close(window, cx);
                if !self.change_several(Change::ToggleTag(id.clone()), window, cx) {
                    self.editor
                        .update(cx, |editor, cx| editor.toggle_tag(id, cx));
                }
            }
            Target::NewTag(name) => {
                self.close(window, cx);
                match self
                    .store
                    .put(DocType::Tag, json!({"name": name}), Put::Upsert, Origin::Ui)
                {
                    Ok(created) => {
                        if !self.change_several(Change::AddTag(created.id.clone()), window, cx) {
                            self.editor
                                .update(cx, |editor, cx| editor.toggle_tag(created.id, cx));
                        }
                    }
                    Err(err) => window.push_notification(
                        Notification::error(format!("Couldn't create the tag: {err}")),
                        cx,
                    ),
                }
            }
            Target::Status(status) => {
                self.close(window, cx);
                if !self.change_several(Change::Status(status), window, cx) {
                    self.editor
                        .update(cx, |editor, cx| editor.set_status(status, cx));
                }
            }
            Target::WordGoal(goal) => {
                self.close(window, cx);
                self.editor
                    .update(cx, |editor, cx| editor.set_word_goal(goal, cx));
            }
            Target::IconTheme(id) => {
                self.close(window, cx);
                icon_themes::choose(Some(id), cx);
            }
            Target::ImageWidth(width) => {
                self.close(window, cx);
                self.editor.update(cx, |editor, cx| {
                    editor.resize_caret_image(width, window, cx)
                });
            }
            Target::Template(id) => {
                self.close(window, cx);
                crate::journal::new_from_template(&self.library, &self.editor, &id, window, cx);
            }
            Target::NewTemplate => {
                self.close(window, cx);
                crate::journal::new_template(&self.library, &self.editor, window, cx);
            }
            Target::Plugin(command) => {
                self.close(window, cx);
                crate::plugins::run_command(
                    command.plugin_id,
                    command.spec.id,
                    self.editor.clone(),
                    cx,
                );
            }
        }
    }

    // MARK: Rendering

    fn render_search(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        div()
            .h(px(SEARCH_HEIGHT))
            .flex_none()
            .px_3()
            .flex()
            .items_center()
            .child(
                h_flex()
                    .w_full()
                    .h(px(40.))
                    .px_3()
                    .gap_2()
                    .rounded(px(8.))
                    .border_1()
                    .border_color(Hsla::from(colors.accent).opacity(0.55))
                    .bg(colors.editor)
                    .child(
                        Icon::new(IconName::Search)
                            .size_4()
                            .text_color(colors.text_muted),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_size(px(15.))
                            .child(Input::new(&self.input).appearance(false).cleanable(true)),
                    ),
            )
    }

    fn render_row(
        &self,
        ix: usize,
        item: &Item,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let colors = Colors::get(cx);
        let selected = self.selected == Some(ix);
        let keys = match &item.target {
            Target::Command(command) => command
                .action()
                .and_then(|action| shortcut(action.as_ref(), window)),
            _ => None,
        };
        h_flex()
            .id(("palette-item", ix))
            .flex_none()
            .h(px(ROW_HEIGHT))
            .mx(px(6.))
            .pl(px(8. + item.depth as f32 * 14.))
            .pr_2()
            .gap(px(10.))
            .rounded(px(6.))
            .cursor_default()
            .when(selected, |el| el.bg(selected_fill(cx)))
            .child({
                let (icon, color) = item.glyph(colors, colors.quiet(selected));
                match &item.icon_path {
                    // A status's glyph stands in for any icon.
                    Some(path) if item.status == NoteStatus::None => {
                        Icon::default().path(path.clone())
                    }
                    _ => Icon::new(icon),
                }
                .size_4()
                .flex_none()
                .text_color(color)
            })
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_size(px(14.))
                    .child(item.label.clone()),
            )
            .when_some(item.hint.clone(), |el, hint| {
                el.child(
                    div()
                        .flex_none()
                        .max_w(px(120.))
                        .truncate()
                        .text_size(px(12.))
                        .text_color(colors.quiet(selected))
                        .child(hint),
                )
            })
            .when(item.checked, |el| {
                el.child(
                    Icon::new(IconName::Check)
                        .size(px(14.))
                        .flex_none()
                        .text_color(colors.accent),
                )
            })
            .when_some(keys, |el, keys| {
                el.child(render_keys(&keys, false, selected, cx))
            })
            .on_mouse_move(cx.listener(move |this, _, _, cx| this.select(ix, false, cx)))
            .on_click(cx.listener(move |this, _, window, cx| this.choose(ix, window, cx)))
    }

    fn render_list(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let mut list = v_flex()
            .id("palette-list")
            .size_full()
            .py_1p5()
            .overflow_y_scroll()
            .track_scroll(&self.scroll);
        for row in &self.rows {
            list = match row {
                Row::Heading(heading) => list.child(
                    div()
                        .flex_none()
                        .px(px(14.))
                        .pt_2()
                        .pb_1()
                        .map(|el| section_label(el, colors))
                        .child(heading.clone()),
                ),
                Row::Separator => list.child(
                    div()
                        .flex_none()
                        .py(px(6.))
                        .px(px(10.))
                        .child(div().h(px(1.)).bg(colors.divider)),
                ),
                Row::Item(ix) => list.child(self.render_row(*ix, &self.items[*ix], window, cx)),
            };
        }
        if self.items.is_empty() {
            let message = if self.searching(cx) {
                "Searching notes…"
            } else {
                match self.mode {
                    Some(Mode::Notes) => "No matching notes",
                    Some(Mode::Pick(Picker::Notebook | Picker::GoToNotebook)) => {
                        "No matching notebooks"
                    }
                    // Only with nothing typed: a name typed offers to make it.
                    Some(Mode::Pick(Picker::Tag)) => "No tags yet. Type a name to create one.",
                    Some(Mode::Pick(Picker::GoToTag)) => "No matching tags",
                    Some(Mode::Pick(Picker::Status)) => "No matching status",
                    _ => "No matching commands or notes",
                }
            };
            list = list.child(
                div()
                    .py_6()
                    .w_full()
                    .text_center()
                    .text_size(px(13.))
                    .text_color(colors.text_muted)
                    .child(message),
            );
        }
        list
    }

    fn render_detail(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let Some(item) = self.selected.and_then(|ix| self.items.get(ix)) else {
            return div().into_any_element();
        };
        let library = self.library.read(cx);
        let mut detail = Detail {
            icon: item.glyph(colors, colors.text_secondary),
            title: item.label.clone(),
            meta: Vec::new(),
            description: None,
            description_lines: 6,
            sections: Vec::new(),
        };
        let note_title = self.note.as_ref().map(|n| note_title(&n.title));
        match &item.target {
            Target::Command(command) => {
                detail.description = Some(command.description.into());
                if let Some(keys) = command
                    .action()
                    .and_then(|action| shortcut(action.as_ref(), window))
                {
                    detail.sections.push((
                        "Shortcut",
                        render_keys(&keys, true, false, cx).into_any_element(),
                    ));
                }
                if matches!(
                    command.needs,
                    Needs::Note
                        | Needs::LiveNote
                        | Needs::UnlockedNote
                        | Needs::LockedNote
                        | Needs::SharedNote
                ) && let Some(title) = note_title.clone()
                {
                    detail
                        .sections
                        .push(("Note", chip(title, cx).into_any_element()));
                }
            }
            Target::Show(filter) => {
                detail.description = Some(
                    match filter {
                        Filter::All => "Shows every note outside the trash.".to_owned(),
                        Filter::Pinned => "Shows the pinned notes.".to_owned(),
                        Filter::Recent => format!(
                            "Shows the notes edited in the last {RECENT_DAYS} days, newest first."
                        ),
                        Filter::Trash => {
                            "Shows the notes in the trash. They can be restored from there."
                                .to_owned()
                        }
                        Filter::Status(status) => {
                            format!("Shows the notes marked {}.", status.label())
                        }
                        Filter::Book(_) => format!("Shows the notes in {}.", item.label),
                        Filter::Tag(_) => format!("Shows the notes tagged #{}.", item.label),
                        Filter::TagPath(path) => {
                            format!("Shows the notes tagged #{path} or a tag inside it.")
                        }
                        Filter::Tasks => "Shows the open tasks in every note, overdue first, then today's, the upcoming ones and those without a date.".to_owned(),
                        Filter::Shared => "Shows the notes other people shared with you, with invitations waiting above them.".to_owned(),
                    }
                    .into(),
                );
                if *filter == Filter::Tasks {
                    if let Some(count) = &item.hint {
                        detail.meta.push((
                            Icon::new(IconName::CircleCheck),
                            format!("{count} overdue").into(),
                        ));
                    }
                } else if let Some(count) = &item.hint {
                    detail
                        .meta
                        .push((Icon::new(IconName::FileText), notes_count(count)));
                }
                if let Filter::Book(id) = filter
                    && let Some(parent) = library
                        .book(id)
                        .and_then(|b| b.parent_book_id.as_deref())
                        .and_then(|p| library.book(p))
                {
                    detail.meta.push((
                        Icon::new(IconName::Notebook),
                        format!("In {}", parent.name).into(),
                    ));
                }
            }
            Target::Note(note) => {
                // Found by meaning: say so, and how close.
                if item.icon == IconName::Sparkles
                    && let Some(close) = &item.hint
                {
                    detail.meta.push((
                        Icon::new(IconName::Sparkles),
                        format!("{close} in meaning to what you typed").into(),
                    ));
                }
                let book = if note.book_id == TRASH_BOOK_ID {
                    "Trash".to_owned()
                } else {
                    library
                        .book(&note.book_id)
                        .map_or_else(|| "No notebook".to_owned(), |b| b.name.clone())
                };
                detail
                    .meta
                    .push((Icon::new(IconName::Notebook), book.into()));
                // The day as the note list names it.
                let edited = crate::clock::Clock::edited_label(cx, note.updated_at);
                detail.meta.push((
                    Icon::new(IconName::Clock),
                    format!("Edited {edited}").into(),
                ));
                // As the note list shows it, Dropped as quiet as the lines.
                if let Some((icon, tint)) = status_glyph(note.status, colors, colors.text_muted) {
                    detail
                        .meta
                        .push((Icon::new(icon).text_color(tint), note.status.label().into()));
                }
                let dark = cx.theme().is_dark();
                let tags: Vec<AnyElement> = note
                    .tags
                    .iter()
                    .filter_map(|id| library.tag(id))
                    .map(|tag| {
                        tag_chip(&tag.name, muted_tag_color(tag.color.rgb(), dark), cx)
                            .into_any_element()
                    })
                    .collect();
                if !tags.is_empty() {
                    detail.sections.push((
                        "Tags",
                        h_flex()
                            .flex_wrap()
                            .gap_1()
                            .children(tags)
                            .into_any_element(),
                    ));
                }
                if !note.snippet.is_empty() {
                    detail.description = Some(note.snippet.clone().into());
                    // Room for the metadata and tags above and below.
                    detail.description_lines = 4;
                }
            }
            Target::Heading(hit) => {
                detail.description = Some(
                    format!(
                        "Opens “{}” at this heading.",
                        self::note_title(&hit.note_title)
                    )
                    .into(),
                );
                detail.meta.push((
                    Icon::new(IconName::FileText),
                    self::note_title(&hit.note_title).into(),
                ));
                detail.meta.push((
                    Icon::new(IconName::Heading),
                    format!("Level {} heading, line {}", hit.level, hit.line + 1).into(),
                ));
            }
            Target::Book(id) => {
                let here = self.note.as_ref().is_some_and(|n| &n.book_id == id);
                detail.description = Some(
                    if here {
                        "The note is in this notebook now.".to_owned()
                    } else {
                        format!("Moves the note into {}.", item.label)
                    }
                    .into(),
                );
                if let Some(count) = &item.hint {
                    detail
                        .meta
                        .push((Icon::new(IconName::FileText), notes_count(count)));
                }
                if let Some(title) = note_title.clone() {
                    detail
                        .sections
                        .push(("Note", chip(title, cx).into_any_element()));
                }
            }
            Target::Tag(_) => {
                detail.title = format!("#{}", item.label).into();
                detail.description = Some(
                    if item.checked {
                        format!(
                            "The note has #{}; choosing it takes the tag off.",
                            item.label
                        )
                    } else if item.icon == IconName::Sparkles {
                        format!(
                            "Adds #{} to the note. Notes about the same things have it.",
                            item.label
                        )
                    } else {
                        format!("Adds #{} to the note.", item.label)
                    }
                    .into(),
                );
                if let Some(count) = &item.hint {
                    detail
                        .meta
                        .push((Icon::new(IconName::FileText), notes_count(count)));
                }
                if let Some(title) = note_title.clone() {
                    detail
                        .sections
                        .push(("Note", chip(title, cx).into_any_element()));
                }
            }
            Target::NewTag(name) => {
                detail.title = format!("#{name}").into();
                detail.description =
                    Some(format!("Creates the tag #{name} and adds it to the note.").into());
                if let Some(title) = note_title.clone() {
                    detail
                        .sections
                        .push(("Note", chip(title, cx).into_any_element()));
                }
            }
            Target::Status(status) => {
                detail.description = Some(status_description(*status).into());
                if let Some(title) = note_title.clone() {
                    detail
                        .sections
                        .push(("Note", chip(title, cx).into_any_element()));
                }
            }
            Target::WordGoal(goal) => {
                detail.description = Some(goal_description(*goal).into());
                if let Some(title) = note_title.clone() {
                    detail
                        .sections
                        .push(("Note", chip(title, cx).into_any_element()));
                }
            }
            Target::Template(id) => {
                // The template's opening lines, as written.
                let body = self.store.note(id).ok().flatten().map(|n| n.body);
                let opening: String = body
                    .as_deref()
                    .unwrap_or_default()
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .take(6)
                    .collect::<Vec<_>>()
                    .join("\n");
                detail.description = Some(if opening.is_empty() {
                    "An empty template.".into()
                } else {
                    opening.into()
                });
                let place = library
                    .default_book()
                    .and_then(|id| library.book(&id).map(|b| b.name.clone()))
                    .unwrap_or_else(|| "Inbox".to_owned());
                detail.meta.push((
                    Icon::new(IconName::Notebook),
                    format!("New note in {place}").into(),
                ));
            }
            Target::IconTheme(id) => {
                if let Some(theme) = cx
                    .try_global::<icon_themes::IconThemes>()
                    .and_then(|themes| themes.get(id))
                {
                    detail.description = Some(
                        theme
                            .description
                            .clone()
                            .unwrap_or_else(|| crate::settings_themes::icon_theme_detail(theme))
                            .into(),
                    );
                    if !theme.author.is_empty() {
                        detail.meta.push((
                            Icon::new(IconName::User),
                            format!("By {}", theme.author).into(),
                        ));
                    }
                    if !theme.license.is_empty() {
                        detail
                            .meta
                            .push((Icon::new(IconName::Info), theme.license.clone().into()));
                    }
                }
            }
            Target::ImageWidth(width) => {
                detail.description = Some(
                    match width {
                        Some(width) => format!("Shows the image {width} pixels wide, or the width of the note if that is less."),
                        None => "Shows the image at its own size, or the width of the note if that is less.".to_owned(),
                    }
                    .into(),
                );
            }
            Target::NewTemplate => {
                detail.description = Some(
                    "Creates a note in the Templates notebook. {{date}}, {{time}}, {{title}}, \
                     {{date:%A %e %B}} and {{cursor}} are filled in when a note is made from it."
                        .into(),
                );
            }
            Target::Plugin(command) => {
                detail.description = Some(
                    format!(
                        "A command from the {} plugin. It runs in its own sandbox and \
                         reaches only what you gave it in Settings › Plugins.",
                        command.plugin_name
                    )
                    .into(),
                );
            }
        }

        v_flex()
            .size_full()
            .overflow_hidden()
            .p(px(22.))
            .gap_2()
            .child(
                Icon::new(detail.icon.0)
                    .size(px(28.))
                    .text_color(detail.icon.1),
            )
            .child(
                div()
                    .mt_2()
                    .text_size(px(18.))
                    .font_semibold()
                    .line_clamp(2)
                    .text_ellipsis()
                    .child(detail.title),
            )
            .when(!detail.meta.is_empty(), |el| {
                el.child(
                    v_flex()
                        .gap_1()
                        .text_size(px(12.5))
                        .text_color(colors.text_muted)
                        .children(detail.meta.into_iter().map(|(icon, text)| {
                            h_flex()
                                .gap_1p5()
                                .child(icon.size(px(13.)).flex_none())
                                .child(div().min_w_0().truncate().child(text))
                        })),
                )
            })
            .when_some(detail.description, |el, description| {
                el.child(
                    div()
                        .mt_1()
                        .flex_none()
                        .text_size(px(13.5))
                        .line_height(relative(1.5))
                        .text_color(colors.text_secondary)
                        .line_clamp(detail.description_lines)
                        .text_ellipsis()
                        .child(description),
                )
            })
            .children(detail.sections.into_iter().map(|(label, content)| {
                v_flex()
                    .mt_3()
                    .gap_1p5()
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(colors.text_muted)
                            .child(label),
                    )
                    .child(content)
            }))
            .into_any_element()
    }

    fn render_footer(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let leave = if self.leads_back() { "Back" } else { "Close" };
        let cell = |glyph: AnyElement, label: &'static str| {
            h_flex()
                .flex_1()
                .h_full()
                .justify_center()
                .gap_1p5()
                .child(glyph)
                .child(label)
        };
        let icon = |name: IconName| Icon::new(name).size(px(14.)).into_any_element();
        let divider = || div().w(px(1.)).h_full().bg(colors.divider);
        h_flex()
            .h(px(FOOTER_HEIGHT))
            .flex_none()
            .text_size(px(12.5))
            .text_color(colors.text_muted)
            .child(
                h_flex()
                    .w(px(LIST_WIDTH))
                    .h_full()
                    .flex_none()
                    .child(cell(icon(IconName::CornerDownLeft), "Select"))
                    .child(divider())
                    .child(cell(icon(IconName::ArrowUpDown), "Navigate")),
            )
            .child(divider())
            .child(cell(keycap("esc", cx).into_any_element(), leave))
    }
}

/// What the detail pane shows for an entry.
struct Detail {
    /// The entry's glyph and its color.
    icon: (IconName, Rgb),
    title: SharedString,
    /// Small lines under the title: where a note lives, when it changed.
    meta: Vec<(Icon, SharedString)>,
    description: Option<SharedString>,
    /// At most this many lines of the description show.
    description_lines: usize,
    /// Labelled parts at the bottom: shortcut, the note a command acts on.
    sections: Vec<(&'static str, AnyElement)>,
}

impl Render for Palette {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.mode.is_none() {
            return div().into_any_element();
        }
        let colors = Colors::get(cx);
        let divider = || div().h(px(1.)).w_full().flex_none().bg(colors.divider);
        div()
            .id("palette-overlay")
            .absolute()
            .inset_0()
            .occlude()
            .bg(cx.theme().overlay)
            .flex()
            .items_center()
            .justify_center()
            .child(
                v_flex()
                    .id("palette")
                    .key_context("Palette")
                    .on_action(cx.listener(Self::select_next))
                    .on_action(cx.listener(Self::select_previous))
                    .on_action(cx.listener(Self::confirm))
                    .on_action(cx.listener(Self::dismiss))
                    .on_action(cx.listener(Self::back))
                    .on_mouse_down_out(cx.listener(|this, _, window, cx| this.close(window, cx)))
                    .w(px(PANEL_WIDTH))
                    .h(px(PANEL_HEIGHT))
                    .overflow_hidden()
                    .rounded(px(10.))
                    .border_1()
                    .border_color(colors.divider)
                    .bg(colors.elevated)
                    .text_color(colors.text)
                    .child(self.render_search(cx))
                    .child(divider())
                    .child(
                        h_flex()
                            .h(px(BODY_HEIGHT))
                            .flex_none()
                            .items_start()
                            .child(
                                div()
                                    .relative()
                                    .w(px(LIST_WIDTH))
                                    .h_full()
                                    .flex_none()
                                    .child(self.render_list(window, cx))
                                    .child(Scrollbar::vertical(&self.scroll)),
                            )
                            .child(div().w(px(1.)).h_full().bg(colors.divider))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .h_full()
                                    .child(self.render_detail(window, cx)),
                            ),
                    )
                    .child(divider())
                    .child(self.render_footer(cx)),
            )
            .into_any_element()
    }
}

/// The heading over the tags close notes suggest.
const SUGGESTED_HEADING: &str = "Suggested by close notes";

/// The heading over notes found by meaning.
const ABOUT_HEADING: &str = "Similar in meaning";

/// The words searched by meaning: the query, when it is plain words long
/// enough to say something, rather than a search with qualifiers. Pure.
fn meaning_query(query: &str) -> Option<String> {
    let words = query.trim();
    let plain = !words.contains(':') && !words.starts_with('-') && !words.contains('"');
    (plain && words.chars().filter(|c| c.is_alphanumeric()).count() >= MEANING_MIN_CHARS)
        .then(|| words.to_owned())
}

/// The notes about `words` that `listed` (the text matches) leaves out,
/// as list rows with their similarity, closest first; none while related
/// notes are off or the model isn't loaded.
fn about_notes(
    store: &Store,
    words: &str,
    listed: &[NoteSummary],
) -> kanso_store::Result<Vec<(NoteSummary, f32)>> {
    let MeaningSearch::Found(found) =
        store.search_by_meaning(words, MEANING_LIMIT + listed.len())?
    else {
        return Ok(Vec::new());
    };
    let ids: Vec<&str> = listed.iter().map(|n| n.id.as_str()).collect();
    let kept = blend_meaning(&ids, found, MEANING_SIMILARITY, MEANING_LIMIT);
    let wanted: Vec<&str> = kept.iter().map(|n| n.id.as_str()).collect();
    let rows = store.summaries_of(&wanted)?;
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let score = kept.iter().find(|n| n.id == row.id)?.score;
            Some((row, score))
        })
        .collect())
}

/// The tag name in what was typed in the tag list: without the "#" tags are
/// shown with, and the spaces around it.
/// The words ⌘P looks for in headings: the query, when ⌘P is open and the
/// query is plain words rather than a search with qualifiers. Pure.
fn heading_query(mode: Option<Mode>, query: &str) -> Option<String> {
    (mode == Some(Mode::Notes))
        .then(|| heading_words(query))
        .flatten()
}

fn tag_name(query: &str) -> &str {
    query
        .trim_start_matches(|c: char| c == '#' || c.is_whitespace())
        .trim_end()
}

/// The keys that run `action`, as the keymap has them now, split for
/// showing apart: "⇧", "⌘", "F".
pub(crate) fn shortcut(action: &dyn Action, window: &Window) -> Option<Vec<String>> {
    let binding = window.highest_precedence_binding_for_action(action)?;
    let keystroke = binding.keystrokes().first()?.as_keystroke().clone();
    Some(shortcut_parts(&Kbd::format(&keystroke)))
}

const MODIFIER_SYMBOLS: [char; 4] = ['⌃', '⌥', '⇧', '⌘'];

/// Splits a shortcut as written ("⇧⌘F") into its modifier symbols and the
/// key. The key is what's left after the leading modifiers, so a key that is
/// itself a modifier (a bare "⌘") stays.
pub(crate) fn shortcut_parts(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut rest = text;
    while let Some(c) = rest.chars().next() {
        if !MODIFIER_SYMBOLS.contains(&c) || rest.len() == c.len_utf8() {
            break;
        }
        parts.push(c.to_string());
        rest = &rest[c.len_utf8()..];
    }
    if !rest.is_empty() {
        parts.push(rest.to_owned());
    }
    parts
}

/// A shortcut: as one key chip in the detail pane (`caps`), or as glyphs
/// spaced like a menu's on a row, in secondary text on the `selected` row.
// no-prop-test: drawing; the parts come from `shortcut_parts`.
fn render_keys(keys: &[String], caps: bool, selected: bool, cx: &App) -> AnyElement {
    if caps {
        // In a row of its own, so the chip keeps to its text's width.
        return h_flex().child(keycap(keys.concat(), cx)).into_any_element();
    }
    let colors = Colors::get(cx);
    h_flex()
        .flex_none()
        .gap(px(3.))
        .text_size(px(12.5))
        .text_color(colors.quiet(selected))
        .children(
            keys.iter()
                .map(|key| div().min_w(px(9.)).text_center().child(key.clone())),
        )
        .into_any_element()
}

/// A bordered chip, like the scope chips in the design, as wide as its
/// text up to the pane's width.
fn chip(text: impl Into<SharedString>, cx: &App) -> impl IntoElement {
    let colors = Colors::get(cx);
    h_flex().child(
        div()
            .min_w_0()
            .px_2p5()
            .py_1()
            .rounded(px(6.))
            .border_1()
            .border_color(colors.divider)
            .text_size(px(12.5))
            .text_color(colors.text)
            .truncate()
            .child(text.into()),
    )
}

/// A tag as a chip, "#name" as everywhere else, the "#" in the tag's color.
/// Bordered rather than filled: the dark theme's fill is the panel's own
/// color.
fn tag_chip(name: &str, rgb: u32, cx: &App) -> impl IntoElement {
    let colors = Colors::get(cx);
    h_flex()
        .px_2()
        .py_0p5()
        .rounded(px(6.))
        .border_1()
        .border_color(colors.divider)
        .text_size(px(12.))
        .text_color(colors.text_secondary)
        .child(div().text_color(rgb_color(rgb)).child("#"))
        .child(name.to_owned())
}

/// The selected row: a quiet tint of the accent, which shows on the
/// elevated panel in both themes.
fn selected_fill(cx: &App) -> Hsla {
    let alpha = if cx.theme().is_dark() {
        PALETTE_SELECTED_ALPHA.0
    } else {
        PALETTE_SELECTED_ALPHA.1
    };
    Hsla::from(Colors::get(cx).accent).opacity(alpha)
}

fn placeholder(mode: Mode) -> &'static str {
    match mode {
        Mode::Commands => "Search commands and notes…",
        Mode::Notes => "Open a note by title…",
        Mode::Pick(Picker::Notebook) => "Move to notebook…",
        Mode::Pick(Picker::Tag) => "Add or create a tag…",
        Mode::Pick(Picker::Status) => "Set status…",
        Mode::Pick(Picker::GoToNotebook) => "Go to notebook…",
        Mode::Pick(Picker::GoToTag) => "Go to tag…",
        Mode::Pick(Picker::WordGoal) => "Words to aim for, such as 1,500…",
        Mode::Pick(Picker::Template) => "New note from template…",
        Mode::Pick(Picker::ImageSize) => "Width in pixels, such as 320…",
        Mode::Pick(Picker::IconTheme) => "Icon theme…",
    }
}

/// The widths an image can be given: what was typed, when it reads as a
/// width, first; then the preview's small, medium and full.
fn image_width_entries(query: &str, current: Option<Option<u32>>) -> Vec<(Item, Option<i64>)> {
    let mut entries = Vec::new();
    let typed = crate::image_size::parse_width(query);
    if let Some(width) = typed {
        let item = Item::new(
            Target::ImageWidth(Some(width)),
            format!("{width} pixels wide"),
            IconName::Image,
        )
        .checked(current == Some(Some(width)));
        entries.push((item, Some(i64::MAX / 8)));
    }
    for (label, width) in crate::preview::IMAGE_SIZES {
        let hint = match width {
            Some(width) => format!("{width} px"),
            None => "Its own size".to_owned(),
        };
        let item = Item::new(Target::ImageWidth(width), label, IconName::Image)
            .checked(current == Some(width))
            .hint(hint);
        let score = item_score(query, label, &["size", "width"]);
        entries.push((item, score));
    }
    entries
}

/// The word goals to choose from: what was typed, when it reads as a
/// number of words, first; then the usual goals, and the default or none.
fn goal_entries(
    query: &str,
    current: GoalSetting,
    default: Option<u32>,
) -> Vec<(Item, Option<i64>)> {
    let words = |n: u32| format!("{} words", format_count(n as usize));
    let mut entries = Vec::new();
    let typed = word_goal::parse_goal(query);
    if let Some(n) = typed {
        let item = Item::new(
            Target::WordGoal(GoalSetting::Words(n)),
            format!("Aim for {}", words(n)),
            IconName::Goal,
        )
        .checked(current == GoalSetting::Words(n));
        entries.push((item, Some(i64::MAX / 8)));
    }
    for n in word_goal::PRESETS {
        if typed == Some(n) {
            continue;
        }
        let item = Item::new(
            Target::WordGoal(GoalSetting::Words(n)),
            words(n),
            IconName::Goal,
        )
        .checked(current == GoalSetting::Words(n));
        let score = item_score(query, &item.label, &[]);
        entries.push((item, score));
    }
    let mut plain = |setting: GoalSetting, label: String| {
        let item = Item::new(Target::WordGoal(setting), label, IconName::Minus)
            .checked(current == setting);
        let score = item_score(query, &item.label, &["none", "off", "clear"]);
        entries.push((item, score));
    };
    match default {
        Some(n) => {
            plain(GoalSetting::Default, format!("Default ({})", words(n)));
            plain(GoalSetting::Off, "No goal for this note".to_owned());
        }
        None => plain(GoalSetting::Default, "No goal".to_owned()),
    }
    entries
}

fn goal_description(goal: GoalSetting) -> String {
    match goal {
        GoalSetting::Words(n) => format!(
            "Focus mode shows how far the note is toward {} words.",
            format_count(n as usize)
        ),
        GoalSetting::Default => {
            "The note follows the default goal set in Settings › Editor.".to_owned()
        }
        GoalSetting::Off => "This note has no word goal, whatever the default.".to_owned(),
    }
}

fn note_title(title: &str) -> String {
    if title.trim().is_empty() {
        "Untitled".to_owned()
    } else {
        title.to_owned()
    }
}

/// A command's label with `several` notes selected: the ones that act on
/// all of them say how many.
fn several_label(id: &str, label: &'static str, several: usize) -> SharedString {
    if several < 2 {
        return label.into();
    }
    match id {
        "pin" if label.starts_with("Unpin") => format!("Unpin {several} notes"),
        "pin" => format!("Pin {several} notes"),
        "move" => format!("Move {several} notes to notebook…"),
        "add-tag" => format!("Add tag to {several} notes…"),
        "status" => format!("Set status of {several} notes…"),
        "trash" => format!("Move {several} notes to trash"),
        _ => return label.into(),
    }
    .into()
}

fn notes_count(count: &str) -> SharedString {
    if count == "1" {
        "1 note".into()
    } else {
        format!("{count} notes").into()
    }
}

fn status_label(status: NoteStatus) -> &'static str {
    match status {
        NoteStatus::None => "No status",
        other => other.label(),
    }
}

fn status_description(status: NoteStatus) -> &'static str {
    match status {
        NoteStatus::None => "Clears the note's status.",
        NoteStatus::Active => "Marks the note Active: being worked on.",
        NoteStatus::OnHold => "Marks the note On hold: paused for now.",
        NoteStatus::Completed => "Marks the note Completed: done.",
        NoteStatus::Dropped => "Marks the note Dropped: it won't be done.",
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};
    use kanso_core::{DocType, NoteStatus};
    use kanso_store::{Origin, Put, Store};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;
    use serde_json::json;

    use super::{
        ABOUT_HEADING, MODIFIER_SYMBOLS, Mode, Palette, Row, SUGGESTED_HEADING, meaning_query,
        merge_by_id, shortcut, shortcut_parts, tag_name, title_search,
    };
    use crate::config::AppConfig;
    use crate::library::Filter;
    use crate::services::Services;
    use crate::workspace::{AppState, FocusSearch, NewNote, SearchAllNotes, Workspace};

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Invariant: the parts put back together are the shortcut; every
        /// part but the last is one modifier symbol, and the last is never
        /// a modifier followed by more (the split takes them all).
        #[test]
        fn prop_shortcut_parts_split_off_every_leading_modifier(text in "[⌃⌥⇧⌘]{0,4}[A-Z1↓\\\\,]{0,2}") {
            let parts = shortcut_parts(&text);
            prop_assert_eq!(parts.concat(), text.clone());
            prop_assert!(parts.iter().all(|p| !p.is_empty()));
            if let Some((key, modifiers)) = parts.split_last() {
                for part in modifiers {
                    let mut chars = part.chars();
                    prop_assert!(chars.next().is_some_and(|c| MODIFIER_SYMBOLS.contains(&c)));
                    prop_assert!(chars.next().is_none());
                }
                let mut chars = key.chars();
                let first = chars.next().unwrap();
                prop_assert!(!MODIFIER_SYMBOLS.contains(&first) || chars.next().is_none(), "{:?}", parts);
            }
        }

        /// Invariants: the name is part of what was typed, with no "#" or
        /// space in front and no space after, and a second pass keeps it.
        #[test]
        fn prop_tag_name_drops_the_hash_and_spaces(query in "[# a-z\t]{0,12}") {
            let name = tag_name(&query);
            prop_assert!(query.contains(name));
            prop_assert!(!name.starts_with(|c: char| c == '#' || c.is_whitespace()));
            prop_assert!(!name.ends_with(char::is_whitespace));
            prop_assert_eq!(tag_name(name), name);
        }

        /// Metamorphic: any run of "#" and spaces in front changes nothing,
        /// and a plain name comes back as it is.
        #[test]
        fn prop_tag_name_ignores_what_tags_are_shown_with(
            prefix in "[# ]{0,4}",
            name in "[a-z][a-z#-]{0,8}",
        ) {
            prop_assert_eq!(tag_name(&name), name.as_str());
            let typed = format!("{prefix}{name}");
            prop_assert_eq!(tag_name(&typed), name.as_str());
        }
    }

    fn summary(id: usize) -> kanso_store::NoteSummary {
        kanso_store::NoteSummary {
            id: format!("note:{id}"),
            rev: String::new(),
            title: String::new(),
            snippet: String::new(),
            book_id: String::new(),
            status: NoteStatus::None,
            tags: Vec::new(),
            pinned: false,
            updated_at: 0,
            created_at: 0,
            tasks_done: 0,
            tasks_total: 0,
            locked: false,
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Invariants: the merge lists each note once, the first list's
        /// notes first and in their order, then the second's in theirs, as
        /// many as the limit allows of the notes in either.
        #[test]
        fn prop_merge_puts_the_first_list_first(
            first in prop::collection::vec(0usize..30, 0..25),
            then in prop::collection::vec(0usize..30, 0..25),
            limit in 0usize..30,
        ) {
            let dedup = |ids: &[usize]| {
                let mut seen = Vec::new();
                for id in ids {
                    if !seen.contains(id) {
                        seen.push(*id);
                    }
                }
                seen
            };
            let (first, then) = (dedup(&first), dedup(&then));
            let merged: Vec<String> = merge_by_id(
                first.iter().map(|&i| summary(i)).collect(),
                then.iter().map(|&i| summary(i)).collect(),
                limit,
            )
            .into_iter()
            .map(|n| n.id)
            .collect();
            let union = dedup(&[first.clone(), then.clone()].concat());
            prop_assert_eq!(merged.len(), union.len().min(limit));
            for (ix, id) in merged.iter().enumerate() {
                prop_assert!(!merged[..ix].contains(id));
            }
            let lead: Vec<String> = first.iter().take(limit).map(|&i| format!("note:{i}")).collect();
            prop_assert_eq!(&merged[..lead.len()], &lead[..]);
            let rest: Vec<String> = then.iter().map(|&i| format!("note:{i}")).filter(|id| !lead.contains(id)).collect();
            prop_assert_eq!(&merged[lead.len()..], &rest[..merged.len() - lead.len()]);
        }
    }

    #[test]
    fn title_searches_ask_the_title() {
        let params = crate::library::search_params(
            &Filter::All,
            "sync -draft tag:x",
            Default::default(),
            true,
        );
        let titled = title_search(&params).unwrap();
        assert_eq!(titled.query.to_string(), "title:sync -draft tag:x");
        let none = crate::library::search_params(&Filter::All, "tag:x", Default::default(), true);
        assert!(title_search(&none).is_none());
        let merged: Vec<String> = merge_by_id(
            vec![summary(1), summary(2)],
            vec![summary(2), summary(3)],
            3,
        )
        .into_iter()
        .map(|n| n.id)
        .collect();
        assert_eq!(merged, ["note:1", "note:2", "note:3"]);
    }

    #[test]
    fn tag_name_examples() {
        assert_eq!(tag_name("#mcp"), "mcp");
        assert_eq!(tag_name("  ##mcp "), "mcp");
        assert_eq!(tag_name("c#"), "c#");
        assert_eq!(tag_name("to do"), "to do");
        assert_eq!(tag_name("#"), "");
        assert_eq!(tag_name(""), "");
    }

    #[test]
    fn shortcut_examples() {
        assert_eq!(shortcut_parts("⇧⌘F"), ["⇧", "⌘", "F"]);
        assert_eq!(shortcut_parts("⌘\\"), ["⌘", "\\"]);
        assert_eq!(shortcut_parts("⌥⌘↓"), ["⌥", "⌘", "↓"]);
        assert_eq!(shortcut_parts("⌘"), ["⌘"]);
        assert_eq!(shortcut_parts("Space"), ["Space"]);
        assert!(shortcut_parts("").is_empty());
    }

    struct Fixture {
        _dir: tempfile::TempDir,
        store: Arc<Store>,
        window: AnyWindowHandle,
        workspace: Entity<Workspace>,
        /// The open note, in notebook "Work".
        note: String,
        other_book: String,
        /// What has the keyboard while writing the note.
        body: gpui_kit::FocusHandle,
    }

    fn setup(cx: &mut TestAppContext) -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
        let book = |name: &str| {
            store
                .put(
                    DocType::Book,
                    json!({"name": name}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap()
                .id
        };
        let work = book("Work");
        let other_book = book("Garden");
        // A minute apart, so the most recently edited is the same however
        // fast the notes are made.
        let now = kanso_core::now_ms();
        let note = |title: &str, body: &str, at: i64| {
            store
                .put(
                    DocType::Note,
                    json!({"title": title, "body": body, "bookId": work,
                        "createdAt": at, "updatedAt": at}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap()
                .id
        };
        note("Quarterly planning", "goals", now - 60_000);
        let open = note("Weekly sync notes", "agenda", now);
        let app_store = store.clone();
        let data_dir = dir.path().to_path_buf();
        let (window, workspace) = cx.update(|cx| {
            gpui_kit::init(cx);
            cx.set_global(AppState {
                store: app_store,
                data_dir,
                config: AppConfig::default(),
            });
            cx.set_global(Services::default());
            crate::workspace::bind_keys(cx);
            gpui_kit::open_window(Default::default(), cx, |window, cx| {
                cx.new(|cx| Workspace::new(window, cx))
            })
            .unwrap()
        });
        // Active, as the window in front is: focus changes report blurs.
        window
            .update(cx, |_, window, _| window.activate_window())
            .unwrap();
        cx.run_until_parked();
        let library = cx.read(|cx| workspace.read(cx).library.clone());
        let id = open.clone();
        library.update(cx, |library, cx| library.select(Some(id), cx));
        cx.run_until_parked();
        // The keyboard in the note's text, as when writing.
        let body = window
            .update(cx, |_, window, cx| {
                let editor = workspace.read(cx).editor.clone();
                editor.update(cx, |editor, cx| editor.focus_body(window, cx));
                window.focused(cx).expect("the body takes the keyboard")
            })
            .unwrap();
        cx.run_until_parked();
        Fixture {
            _dir: dir,
            store,
            window,
            workspace,
            note: open,
            other_book,
            body,
        }
    }

    impl Fixture {
        fn update<R>(
            &self,
            cx: &mut TestAppContext,
            f: impl FnOnce(
                &mut Workspace,
                &mut gpui_kit::Window,
                &mut gpui_kit::Context<Workspace>,
            ) -> R,
        ) -> R {
            let workspace = self.workspace.clone();
            let result = self
                .window
                .update(cx, |_, window, cx| {
                    workspace.update(cx, |workspace, cx| f(workspace, window, cx))
                })
                .unwrap();
            cx.run_until_parked();
            result
        }

        fn palette(&self, cx: &mut TestAppContext) -> Entity<Palette> {
            cx.read(|cx| self.workspace.read(cx).palette.clone())
        }

        fn mode(&self, cx: &mut TestAppContext) -> Option<Mode> {
            let palette = self.palette(cx);
            cx.read(|cx| palette.read(cx).mode())
        }

        fn keys(&self, cx: &mut TestAppContext, keys: &str) {
            cx.simulate_keystrokes(self.window, keys);
            // Notes are searched after a pause in typing.
            cx.executor()
                .advance_clock(std::time::Duration::from_millis(200));
            cx.run_until_parked();
        }

        fn type_query(&self, cx: &mut TestAppContext, query: &str) {
            let palette = self.palette(cx);
            self.window
                .update(cx, |_, window, cx| {
                    palette.update(cx, |palette, cx| palette.set_query(query, window, cx))
                })
                .unwrap();
            cx.executor()
                .advance_clock(std::time::Duration::from_millis(200));
            cx.run_until_parked();
        }

        fn selected_label(&self, cx: &mut TestAppContext) -> Option<String> {
            let palette = self.palette(cx);
            cx.read(|cx| {
                let palette = palette.read(cx);
                palette
                    .selected
                    .map(|ix| palette.items[ix].label.to_string())
            })
        }

        fn labels(&self, cx: &mut TestAppContext) -> Vec<String> {
            let palette = self.palette(cx);
            cx.read(|cx| {
                palette
                    .read(cx)
                    .items
                    .iter()
                    .map(|i| i.label.to_string())
                    .collect()
            })
        }

        fn body_focused(&self, cx: &mut TestAppContext) -> bool {
            let body = self.body.clone();
            self.update(cx, |_, window, _| body.is_focused(window))
        }

        /// Draws, so focus changes reach their listeners and actions find
        /// what is on screen.
        fn draw(&self, cx: &mut TestAppContext) {
            self.window
                .update(cx, |_, window, cx| {
                    window.refresh();
                    window.draw(cx).clear(cx);
                })
                .unwrap();
            cx.run_until_parked();
        }

        fn open(&self, cx: &mut TestAppContext, mode: Mode) {
            self.update(cx, |workspace, window, cx| {
                workspace
                    .palette
                    .update(cx, |palette, cx| palette.open(mode, window, cx));
            });
        }

        /// Clicks the middle of what is drawn with `selector`.
        fn click(&self, cx: &mut TestAppContext, selector: &'static str) {
            self.draw(cx);
            let mut vcx = gpui_kit::VisualTestContext::from_window(self.window, cx);
            let bounds = vcx
                .debug_bounds(selector)
                .unwrap_or_else(|| panic!("{selector} is drawn"));
            vcx.simulate_click(bounds.center(), gpui_kit::Modifiers::none());
            cx.run_until_parked();
        }
    }

    #[gpui_kit::test]
    fn cmd_k_opens_and_escape_gives_the_keyboard_back(cx: &mut TestAppContext) {
        let f = setup(cx);
        assert!(f.body_focused(cx));
        f.keys(cx, "cmd-k");
        assert_eq!(f.mode(cx), Some(Mode::Commands));
        assert!(!f.body_focused(cx), "the field has the keyboard");
        assert_eq!(f.selected_label(cx).as_deref(), Some("New note"));
        f.keys(cx, "down down");
        assert_eq!(f.selected_label(cx).as_deref(), Some("Search all notes"));
        f.keys(cx, "up");
        assert_eq!(f.selected_label(cx).as_deref(), Some("New notebook…"));
        f.keys(cx, "escape");
        assert_eq!(f.mode(cx), None);
        assert!(f.body_focused(cx), "back where it was");
        // The shortcut again closes it too.
        f.keys(cx, "cmd-k");
        f.keys(cx, "cmd-k");
        assert_eq!(f.mode(cx), None);
        assert!(f.body_focused(cx));
    }

    #[gpui_kit::test]
    fn rows_show_the_real_shortcuts(cx: &mut TestAppContext) {
        let f = setup(cx);
        f.keys(cx, "cmd-k");
        let (new_note, search_all, search_list) = f.update(cx, |_, window, _| {
            (
                shortcut(&NewNote, window),
                shortcut(&SearchAllNotes, window),
                shortcut(&FocusSearch, window),
            )
        });
        assert_eq!(new_note, Some(vec!["⌘".to_owned(), "N".to_owned()]));
        assert_eq!(
            search_all,
            Some(vec!["⇧".to_owned(), "⌘".to_owned(), "F".to_owned()])
        );
        assert_eq!(search_list, Some(vec!["⌘".to_owned(), "L".to_owned()]));
    }

    /// Global search leaves the notebook that was showing: what is typed
    /// finds notes in every notebook. Searching this list keeps to it.
    #[gpui_kit::test]
    fn search_all_notes_searches_every_notebook(cx: &mut TestAppContext) {
        let f = setup(cx);
        f.store
            .put(
                DocType::Note,
                json!({"title": "Tomato seedlings", "body": "", "bookId": f.other_book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        let library = cx.read(|cx| f.workspace.read(cx).library.clone());
        let work = cx.read(|cx| {
            let books = &library.read(cx).books;
            books.iter().find(|b| b.name == "Work").unwrap().id.clone()
        });
        library.update(cx, |library, cx| {
            library.set_filter(Filter::Book(work.clone()), cx);
            library.set_pinned_only(true, cx);
        });
        cx.run_until_parked();
        let search = cx.read(|cx| f.workspace.read(cx).note_list.read(cx).search.clone());
        let titles = |cx: &mut TestAppContext| -> Vec<String> {
            cx.read(|cx| {
                library
                    .read(cx)
                    .notes
                    .iter()
                    .map(|n| n.title.clone())
                    .collect()
            })
        };
        let search_focused = |cx: &mut TestAppContext| {
            let search = search.clone();
            f.update(cx, |_, window, cx| {
                gpui_kit::Focusable::focus_handle(search.read(cx), cx).is_focused(window)
            })
        };

        // This list: still Work, and the tomato isn't in it.
        f.keys(cx, "cmd-k");
        f.type_query(cx, "search this list");
        f.keys(cx, "enter");
        assert!(search_focused(cx));
        assert_eq!(
            cx.read(|cx| library.read(cx).filter.clone()),
            Filter::Book(work)
        );
        f.keys(cx, "t o m a t o");
        cx.executor()
            .advance_clock(std::time::Duration::from_secs(1));
        cx.run_until_parked();
        assert!(titles(cx).is_empty());

        // All notes: the same words find it in Garden.
        f.keys(cx, "cmd-k");
        f.type_query(cx, "search all notes");
        assert_eq!(f.selected_label(cx).as_deref(), Some("Search all notes"));
        f.keys(cx, "enter");
        cx.executor()
            .advance_clock(std::time::Duration::from_secs(1));
        cx.run_until_parked();
        assert!(search_focused(cx));
        let library_state = cx.read(|cx| {
            let library = library.read(cx);
            (
                library.filter.clone(),
                library.is_refined(),
                library.search.clone(),
            )
        });
        assert_eq!(library_state, (Filter::All, false, "tomato".to_owned()));
        assert_eq!(titles(cx), ["Tomato seedlings"]);
    }

    #[gpui_kit::test]
    fn typing_ranks_commands_and_finds_notes(cx: &mut TestAppContext) {
        let f = setup(cx);
        // Sync now is there while signed in.
        cx.update(|cx| {
            cx.global_mut::<Services>().account = crate::services::AccountState::SignedIn {
                username: "ngoni".into(),
            }
        });
        f.keys(cx, "cmd-k");
        f.type_query(cx, "sync");
        let labels = f.labels(cx);
        assert!(labels.contains(&"Sync now".to_owned()), "{labels:?}");
        assert!(
            labels.contains(&"Weekly sync notes".to_owned()),
            "{labels:?}"
        );
        assert!(!labels.contains(&"New note".to_owned()), "{labels:?}");
        f.type_query(cx, "trash");
        // The place called Trash fits better than the command that moves
        // notes there.
        assert_eq!(f.selected_label(cx).as_deref(), Some("Trash"));
    }

    #[gpui_kit::test]
    fn a_command_runs_through_the_window(cx: &mut TestAppContext) {
        let f = setup(cx);
        f.keys(cx, "cmd-k");
        f.type_query(cx, "pin note");
        assert_eq!(f.selected_label(cx).as_deref(), Some("Pin note"));
        f.keys(cx, "enter");
        assert_eq!(f.mode(cx), None);
        assert!(f.store.note(&f.note).unwrap().unwrap().pinned);
        assert!(f.body_focused(cx));
        f.keys(cx, "cmd-k");
        f.type_query(cx, "pin note");
        assert_eq!(f.selected_label(cx).as_deref(), Some("Unpin note"));
    }

    #[gpui_kit::test]
    fn moving_to_a_notebook_uses_a_second_list(cx: &mut TestAppContext) {
        let f = setup(cx);
        f.keys(cx, "cmd-k");
        f.type_query(cx, "move to notebook");
        f.keys(cx, "enter");
        assert_eq!(
            f.mode(cx),
            Some(Mode::Pick(crate::commands::Picker::Notebook))
        );
        assert_eq!(f.labels(cx), ["Garden", "Work"]);
        // Escape goes back to the commands rather than closing.
        f.keys(cx, "escape");
        assert_eq!(f.mode(cx), Some(Mode::Commands));
        f.type_query(cx, "move to notebook");
        f.keys(cx, "enter");
        f.type_query(cx, "gard");
        f.keys(cx, "enter");
        assert_eq!(f.mode(cx), None);
        assert_eq!(
            f.store.note(&f.note).unwrap().unwrap().book_id,
            f.other_book
        );
    }

    #[gpui_kit::test]
    fn backspace_in_an_empty_field_leaves_the_second_list(cx: &mut TestAppContext) {
        let f = setup(cx);
        f.keys(cx, "cmd-k");
        f.type_query(cx, "set status");
        f.keys(cx, "enter");
        assert_eq!(
            f.mode(cx),
            Some(Mode::Pick(crate::commands::Picker::Status))
        );
        f.keys(cx, "a backspace");
        assert_eq!(
            f.mode(cx),
            Some(Mode::Pick(crate::commands::Picker::Status))
        );
        f.keys(cx, "backspace");
        assert_eq!(f.mode(cx), Some(Mode::Commands));
    }

    #[gpui_kit::test]
    fn setting_a_status_and_creating_a_tag(cx: &mut TestAppContext) {
        let f = setup(cx);
        f.keys(cx, "cmd-k");
        f.type_query(cx, "set status");
        f.keys(cx, "enter");
        f.type_query(cx, "hold");
        f.keys(cx, "enter");
        assert_eq!(
            f.store.note(&f.note).unwrap().unwrap().status,
            NoteStatus::OnHold
        );

        f.keys(cx, "cmd-k");
        f.type_query(cx, "add tag");
        f.keys(cx, "enter");
        f.type_query(cx, "errands");
        assert_eq!(f.labels(cx), ["Create tag “errands”"]);
        f.keys(cx, "enter");
        let tags = f.store.tags().unwrap();
        let tag = tags.iter().find(|t| t.name == "errands").expect("created");
        assert_eq!(
            f.store.note(&f.note).unwrap().unwrap().tags,
            vec![tag.id.clone()]
        );
    }

    /// Tags are shown as "#errands"; typed that way, the tag list finds the
    /// tag rather than offering to make "#errands" as well.
    #[gpui_kit::test]
    fn a_tag_typed_with_its_hash_is_found(cx: &mut TestAppContext) {
        let f = setup(cx);
        let errands = f
            .store
            .put(
                DocType::Tag,
                json!({"name": "errands"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        cx.run_until_parked();
        f.keys(cx, "cmd-k");
        f.type_query(cx, "add tag");
        f.keys(cx, "enter");
        f.type_query(cx, "#");
        assert_eq!(f.labels(cx), ["errands"], "every tag, nothing to create");
        // The tag first and chosen; making "err" stays possible, last.
        f.type_query(cx, "#err");
        assert_eq!(f.labels(cx), ["errands", "Create tag “err”"]);
        assert_eq!(f.selected_label(cx).as_deref(), Some("errands"));
        f.keys(cx, "enter");
        assert_eq!(f.store.note(&f.note).unwrap().unwrap().tags, [errands]);
        let names: Vec<String> = f
            .store
            .tags()
            .unwrap()
            .into_iter()
            .map(|t| t.name)
            .collect();
        assert_eq!(names, ["errands"]);
        // A new one is made under its name, without the "#".
        f.keys(cx, "cmd-k");
        f.type_query(cx, "add tag");
        f.keys(cx, "enter");
        f.type_query(cx, " #garden ");
        assert_eq!(f.labels(cx), ["Create tag “garden”"]);
        f.keys(cx, "enter");
        let tags = f.store.tags().unwrap();
        assert!(tags.iter().any(|t| t.name == "garden"), "{tags:?}");
        assert!(!tags.iter().any(|t| t.name.contains('#')), "{tags:?}");
    }

    /// The note's notebook path and "Add tag" open the palette's lists
    /// straight away, which filter, scroll and make new tags. Escape closes
    /// them rather than going back to commands that were never shown, and a
    /// library with no tags yet can make its first one from there.
    #[gpui_kit::test]
    fn the_header_opens_the_notebook_and_tag_lists(cx: &mut TestAppContext) {
        use crate::commands::Picker;

        let f = setup(cx);
        f.click(cx, "add-tag");
        assert_eq!(f.mode(cx), Some(Mode::Pick(Picker::Tag)));
        assert!(f.labels(cx).is_empty(), "no tags yet");
        // Backspace in the empty field has no list to go back to.
        f.keys(cx, "backspace");
        assert_eq!(f.mode(cx), Some(Mode::Pick(Picker::Tag)));
        f.keys(cx, "escape");
        assert_eq!(f.mode(cx), None);
        assert!(f.body_focused(cx), "back to the note");

        f.click(cx, "add-tag");
        f.type_query(cx, "errands");
        assert_eq!(f.labels(cx), ["Create tag “errands”"]);
        f.keys(cx, "enter");
        assert_eq!(f.mode(cx), None);
        let tags = f.store.tags().unwrap();
        let errands = tags.iter().find(|t| t.name == "errands").expect("created");
        assert_eq!(
            f.store.note(&f.note).unwrap().unwrap().tags,
            std::slice::from_ref(&errands.id)
        );
        assert!(f.body_focused(cx));

        f.click(cx, "breadcrumb");
        assert_eq!(f.mode(cx), Some(Mode::Pick(Picker::Notebook)));
        assert_eq!(f.labels(cx), ["Garden", "Work"]);
        f.type_query(cx, "gard");
        f.keys(cx, "enter");
        assert_eq!(f.mode(cx), None);
        assert_eq!(
            f.store.note(&f.note).unwrap().unwrap().book_id,
            f.other_book
        );
    }

    #[gpui_kit::test]
    fn cmd_p_opens_a_note_by_title(cx: &mut TestAppContext) {
        let f = setup(cx);
        f.keys(cx, "cmd-p");
        assert_eq!(f.mode(cx), Some(Mode::Notes));
        // The most recently edited first, with nothing typed.
        assert_eq!(f.labels(cx), ["Weekly sync notes", "Quarterly planning"]);
        // The open note is first; Enter goes to the one before it.
        assert_eq!(f.selected_label(cx).as_deref(), Some("Quarterly planning"));
        f.type_query(cx, "qplan");
        assert_eq!(
            f.labels(cx),
            Vec::<String>::new(),
            "the store searches text"
        );
        f.type_query(cx, "plan");
        assert_eq!(f.labels(cx), ["Quarterly planning"]);
        f.keys(cx, "enter");
        assert_eq!(f.mode(cx), None);
        let library = cx.read(|cx| f.workspace.read(cx).library.clone());
        let selected = cx.read(|cx| library.read(cx).selected.clone()).unwrap();
        assert_eq!(
            f.store.note(&selected).unwrap().unwrap().title,
            "Quarterly planning"
        );
        assert!(f.body_focused(cx));
    }

    /// A stand-in model that knows two subjects: plants and planning.
    struct Subjects;

    impl kanso_store::Embedder for Subjects {
        fn model(&self) -> &str {
            "subjects"
        }
        fn dim(&self) -> usize {
            2
        }
        fn embed(&self, texts: &[String]) -> Vec<Vec<f32>> {
            let count = |t: &str, words: &[&str]| {
                words.iter().map(|w| t.matches(w).count()).sum::<usize>() as f32
            };
            texts
                .iter()
                .map(|t| {
                    let t = t.to_lowercase();
                    vec![
                        count(&t, &["vegetable", "tomato", "basil", "garden"]),
                        count(&t, &["plan", "goal", "agenda", "schedule"]),
                    ]
                })
                .collect()
        }
    }

    /// With related notes on, a search also lists notes about its words
    /// that don't have them, after the ones that do, never twice, and
    /// says how close each is.
    #[gpui_kit::test]
    fn notes_about_the_search_follow_the_matches(cx: &mut TestAppContext) {
        let f = setup(cx);
        let book = f.store.note(&f.note).unwrap().unwrap().book_id;
        f.store
            .put(
                DocType::Note,
                json!({"title": "Raised beds", "body": "tomatoes and basil", "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        let embed_all = |store: &Store| {
            let queue = store.embedding_queue("subjects").unwrap();
            let texts: Vec<String> = queue.pending.iter().map(|p| p.text.clone()).collect();
            let rows: Vec<_> = queue
                .pending
                .into_iter()
                .zip(kanso_store::Embedder::embed(&Subjects, &texts))
                .map(|(p, v)| (p.id, p.hash, v))
                .collect();
            store.put_embeddings("subjects", &rows).unwrap();
        };
        // Off: words only.
        f.keys(cx, "cmd-p");
        f.type_query(cx, "vegetables");
        assert!(f.labels(cx).is_empty());
        f.store.set_embedding_model(Some("subjects")).unwrap();
        embed_all(&f.store);
        f.store.lend_embedder(Some(Arc::new(Subjects)));
        f.type_query(cx, "vegetable garden");
        assert_eq!(f.labels(cx), ["Raised beds"]);
        let palette = f.palette(cx);
        let (heading, hint) = cx.read(|cx| {
            let palette = palette.read(cx);
            let heading = palette.rows.iter().find_map(|row| match row {
                Row::Heading(h) => Some(h.to_string()),
                _ => None,
            });
            (
                heading,
                palette.items[0].hint.clone().map(|h| h.to_string()),
            )
        });
        assert_eq!(heading.as_deref(), Some(ABOUT_HEADING));
        assert_eq!(hint.as_deref(), Some("Very close"));
        // A note with the words is listed once, with the matches.
        f.type_query(cx, "planning");
        assert_eq!(f.labels(cx), ["Quarterly planning", "Weekly sync notes"]);
        let rows = cx.read(|cx| {
            palette
                .read(cx)
                .rows
                .iter()
                .filter_map(|row| match row {
                    Row::Heading(h) => Some(h.to_string()),
                    _ => None,
                })
                .collect::<Vec<_>>()
        });
        assert_eq!(rows, ["Notes", ABOUT_HEADING]);
        // Enter opens it, like any note.
        f.type_query(cx, "tomato garden");
        f.keys(cx, "enter");
        let library = cx.read(|cx| f.workspace.read(cx).library.clone());
        let selected = cx.read(|cx| library.read(cx).selected.clone()).unwrap();
        assert_eq!(
            f.store.note(&selected).unwrap().unwrap().title,
            "Raised beds"
        );
        f.store.lend_embedder(None);
    }

    /// The tag list leads with the tags close notes have and the open note
    /// hasn't, once each; adding one takes it off the suggestions.
    #[gpui_kit::test]
    fn the_tag_list_suggests_tags_from_close_notes(cx: &mut TestAppContext) {
        use crate::commands::Picker;
        use crate::related::{MODEL_ID, Related};
        let f = setup(cx);
        let book = f.store.note(&f.note).unwrap().unwrap().book_id;
        let tagged = |title: &str, body: &str, tag: &str| {
            let tag = f.store.tag_named(tag, Origin::Ui).unwrap();
            f.store
                .put(
                    DocType::Note,
                    json!({"title": title, "body": body, "bookId": book, "tags": [tag]}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap();
        };
        tagged("Retro", "the schedule and agenda", "planning");
        tagged("Raised beds", "tomatoes and basil", "garden");
        f.store.set_embedding_model(Some(MODEL_ID)).unwrap();
        let queue = f.store.embedding_queue(MODEL_ID).unwrap();
        let texts: Vec<String> = queue.pending.iter().map(|p| p.text.clone()).collect();
        let rows: Vec<_> = queue
            .pending
            .into_iter()
            .zip(kanso_store::Embedder::embed(&Subjects, &texts))
            .map(|(p, v)| (p.id, p.hash, v))
            .collect();
        f.store.put_embeddings(MODEL_ID, &rows).unwrap();
        // New embeddings: the open note's suggestions are read again.
        cx.update(|cx| {
            cx.set_global(Related::default());
            gpui_kit::BorrowAppContext::update_global::<Related, _>(cx, |related, _| {
                related.generation = 1
            });
        });
        cx.run_until_parked();
        let headings = |cx: &mut TestAppContext| {
            let palette = f.palette(cx);
            cx.read(|cx| {
                palette
                    .read(cx)
                    .rows
                    .iter()
                    .filter_map(|row| match row {
                        Row::Heading(h) => Some(h.to_string()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            })
        };
        f.open(cx, Mode::Pick(Picker::Tag));
        assert_eq!(headings(cx), [SUGGESTED_HEADING, "Tags"]);
        let labels = f.labels(cx);
        assert_eq!(labels, ["planning", "garden"]);
        // Typing searches every tag as before.
        f.type_query(cx, "gar");
        assert_eq!(headings(cx), ["Tags"]);
        f.type_query(cx, "");
        f.keys(cx, "enter");
        let tags = f.store.note(&f.note).unwrap().unwrap().tags;
        assert_eq!(tags, [f.store.tag_named("planning", Origin::Ui).unwrap()]);
        f.open(cx, Mode::Pick(Picker::Tag));
        assert_eq!(headings(cx), ["Tags"]);
    }

    #[test]
    fn meaning_searches_plain_words_only() {
        assert_eq!(meaning_query(" fruit trees "), Some("fruit trees".into()));
        assert_eq!(meaning_query("ab"), None);
        assert_eq!(meaning_query("tag:work"), None);
        assert_eq!(meaning_query("-draft"), None);
        assert_eq!(meaning_query("\"exact words\""), None);
    }

    #[gpui_kit::test]
    fn opening_a_note_from_the_command_list_switches_lists(cx: &mut TestAppContext) {
        let f = setup(cx);
        f.keys(cx, "cmd-k");
        f.type_query(cx, "open note");
        assert_eq!(f.selected_label(cx).as_deref(), Some("Open note…"));
        f.keys(cx, "enter");
        assert_eq!(f.mode(cx), Some(Mode::Notes));
        // Closing returns the keyboard to the note, not to the first list.
        f.keys(cx, "escape");
        assert!(f.body_focused(cx));
    }

    /// Every command the palette runs as an action finds a handler from
    /// where the palette sends it (the note being written, inside the
    /// workspace), whatever the layout. Quit is the app's own: main.rs
    /// registers it with `cx.on_action`, and Quick capture and zoom too
    /// (`zoom::register`). Settings open
    /// in the window.
    /// Account commands have a handler only in the state they fit, so
    /// the menu greys out Sync Now while signed out; each is checked in
    /// that state. Go back and forward have one only where they would land
    /// (checked in `nav_tests`).
    #[gpui_kit::test]
    fn every_command_reaches_a_handler(cx: &mut TestAppContext) {
        let f = setup(cx);
        // Quick capture opens its own window, from anywhere in the app.
        // Zoom changes every window at once.
        let app_wide = [
            "kanso::Quit",
            "kanso::QuickCapture",
            "kanso::ZoomIn",
            "kanso::ZoomOut",
            "kanso::ActualSize",
        ];
        for (signed_in, locked) in [(false, false), (true, false), (false, true)] {
            cx.update(|cx| {
                cx.global_mut::<Services>().account = if signed_in {
                    crate::services::AccountState::SignedIn {
                        username: "ngoni".into(),
                    }
                } else if locked {
                    crate::services::AccountState::SignedInLocked {
                        username: "ngoni".into(),
                    }
                } else {
                    crate::services::AccountState::SignedOut
                };
                cx.refresh_windows();
            });
            cx.run_until_parked();
            let missing: Vec<&str> = f.update(cx, |_, window, cx| {
                crate::commands::all()
                    .iter()
                    .filter(|command| !matches!(command.id, "go-back" | "go-forward"))
                    .filter(|command| match command.needs {
                        crate::commands::Needs::SignedIn => signed_in,
                        crate::commands::Needs::SignedOut => !signed_in && !locked,
                        _ => true,
                    })
                    .filter(|command| {
                        command.action().is_some_and(|action| {
                            !window.is_action_available(action.as_ref(), cx)
                                && !app_wide.contains(&action.name())
                        })
                    })
                    .map(|command| command.id)
                    .collect()
            });
            assert!(
                missing.is_empty(),
                "signed in {signed_in}: no handler for {missing:?}"
            );
        }
        // The exceptions are real commands, so the list can't go stale.
        for name in app_wide {
            assert!(
                crate::commands::all()
                    .iter()
                    .any(|c| c.action().is_some_and(|a| a.name() == name)),
                "{name}"
            );
        }
    }

    #[gpui_kit::test]
    fn go_to_shows_recent(cx: &mut TestAppContext) {
        let f = setup(cx);
        f.keys(cx, "cmd-k");
        f.type_query(cx, "recent");
        assert_eq!(f.selected_label(cx).as_deref(), Some("Recent"));
        f.keys(cx, "enter");
        assert_eq!(f.mode(cx), None);
        let library = cx.read(|cx| f.workspace.read(cx).library.clone());
        assert_eq!(
            cx.read(|cx| library.read(cx).filter.clone()),
            Filter::Recent
        );
    }

    #[gpui_kit::test]
    fn note_actions_need_an_open_note(cx: &mut TestAppContext) {
        let f = setup(cx);
        let library = cx.read(|cx| f.workspace.read(cx).library.clone());
        library.update(cx, |library, cx| library.select(None, cx));
        cx.run_until_parked();
        f.open(cx, Mode::Commands);
        let labels = f.labels(cx);
        assert!(labels.contains(&"New note".to_owned()));
        for hidden in [
            "Pin note",
            "Move to notebook…",
            "Move to trash",
            "Show history…",
        ] {
            assert!(
                !labels.contains(&hidden.to_owned()),
                "{hidden} in {labels:?}"
            );
        }
    }

    /// Notes with the words in their titles come first, even when many newer
    /// notes only mention them.
    #[gpui_kit::test]
    fn titled_notes_beat_newer_mentions(cx: &mut TestAppContext) {
        let f = setup(cx);
        let book = f.store.books().unwrap()[0].id.clone();
        f.store
            .put(
                DocType::Note,
                json!({"title": "Sync design", "body": "D1 and R2", "bookId": book,
                    "createdAt": 1_000, "updatedAt": 1_000}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        for n in 0..25 {
            f.store
                .put(
                    DocType::Note,
                    json!({"title": format!("Task {n}"), "body": "rechecked after the sync", "bookId": book}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap();
        }
        f.keys(cx, "cmd-p");
        f.type_query(cx, "sync");
        let labels = f.labels(cx);
        let at = labels.iter().position(|l| l == "Sync design");
        assert!(at.is_some_and(|at| at < 2), "{labels:?}");
    }

    /// Going back to the query already shown drops the search for the one
    /// in between, so its results never replace the right ones.
    #[gpui_kit::test]
    fn an_abandoned_search_never_lands(cx: &mut TestAppContext) {
        let f = setup(cx);
        f.keys(cx, "cmd-p");
        f.type_query(cx, "sync");
        let shown = f.labels(cx);
        assert_eq!(shown, ["Weekly sync notes"]);
        let palette = f.palette(cx);
        let set = |cx: &mut TestAppContext, query: &str| {
            f.window
                .update(cx, |_, window, cx| {
                    palette.update(cx, |palette, cx| palette.set_query(query, window, cx))
                })
                .unwrap();
            cx.run_until_parked();
        };
        set(cx, "syncx");
        set(cx, "sync");
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(200));
        cx.run_until_parked();
        assert_eq!(f.labels(cx), shown);
        assert!(!cx.read(|cx| palette.read(cx).searching(cx)));
    }

    /// A shortcut that moves the keyboard elsewhere closes the palette
    /// rather than leaving it over what is being typed into.
    #[gpui_kit::test]
    fn a_shortcut_that_takes_the_keyboard_closes_the_palette(cx: &mut TestAppContext) {
        let f = setup(cx);
        f.keys(cx, "cmd-k");
        assert_eq!(f.mode(cx), Some(Mode::Commands));
        f.keys(cx, "cmd-n");
        f.draw(cx);
        assert_eq!(f.mode(cx), None);
        let editor = cx.read(|cx| f.workspace.read(cx).editor.clone());
        assert!(f.update(cx, |_, window, cx| {
            editor.read(cx).title_focused(window, cx)
        }));
        // Typing goes into the new note's title.
        f.keys(cx, "h i");
        let id = cx
            .read(|cx| editor.read(cx).note_id().map(str::to_owned))
            .unwrap();
        cx.executor()
            .advance_clock(std::time::Duration::from_secs(1));
        cx.run_until_parked();
        assert_eq!(f.store.note(&id).unwrap().unwrap().title, "hi");
    }

    /// The empty notebook's palette row, with the keyboard last in a note's
    /// text that is no longer drawn: New note still makes a note there.
    #[gpui_kit::test]
    fn commands_run_whatever_had_the_keyboard(cx: &mut TestAppContext) {
        let f = setup(cx);
        let library = cx.read(|cx| f.workspace.read(cx).library.clone());
        let research = f
            .store
            .put(
                DocType::Book,
                json!({"name": "Research"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        cx.run_until_parked();
        library.update(cx, |library, cx| {
            library.set_filter(Filter::Book(research.clone()), cx)
        });
        cx.run_until_parked();
        assert!(f.update(cx, |workspace, _, cx| {
            workspace.editor.read(cx).note_id().is_none()
        }));
        let empty = cx.read(|cx| f.workspace.read(cx).editor.read(cx).empty_focus.clone());
        f.draw(cx);
        f.window
            .update(cx, |_, window, cx| {
                empty.dispatch_action(&crate::workspace::OpenCommandPalette, window, cx)
            })
            .unwrap();
        cx.run_until_parked();
        assert_eq!(f.mode(cx), Some(Mode::Commands));
        f.type_query(cx, "new note");
        assert_eq!(f.selected_label(cx).as_deref(), Some("New note"));
        f.keys(cx, "enter");
        let in_research = f
            .store
            .search_summaries(&Default::default())
            .unwrap()
            .into_iter()
            .filter(|n| n.book_id == research)
            .count();
        assert_eq!(in_research, 1);
    }

    /// Opening a note from ⌘P while only the preview shows leaves the
    /// keyboard somewhere drawn, so the shortcuts keep working.
    #[gpui_kit::test]
    fn shortcuts_work_after_opening_a_note_in_preview(cx: &mut TestAppContext) {
        let f = setup(cx);
        f.update(cx, |workspace, _, cx| {
            workspace.editor.update(cx, |editor, cx| {
                editor.set_view_mode(crate::config::ViewMode::Preview, cx)
            });
        });
        f.draw(cx);
        f.keys(cx, "cmd-p");
        assert_eq!(f.mode(cx), Some(Mode::Notes));
        f.keys(cx, "enter");
        assert_eq!(f.mode(cx), None);
        f.draw(cx);
        f.keys(cx, "cmd-p");
        assert_eq!(f.mode(cx), Some(Mode::Notes));
        // And choosing Preview from the palette, which hides the text that
        // gets the keyboard back.
        f.keys(cx, "escape");
        f.update(cx, |workspace, _, cx| {
            workspace.editor.update(cx, |editor, cx| {
                editor.set_view_mode(crate::config::ViewMode::Edit, cx)
            });
        });
        f.update(cx, |workspace, window, cx| {
            workspace
                .editor
                .update(cx, |editor, cx| editor.focus_body(window, cx));
        });
        f.keys(cx, "cmd-k");
        f.type_query(cx, "preview");
        f.keys(cx, "enter");
        f.draw(cx);
        f.keys(cx, "cmd-k");
        assert_eq!(f.mode(cx), Some(Mode::Commands));
    }
}
