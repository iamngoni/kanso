//! The note editor: a header band (where the note is, whether it is saved,
//! the view mode and note actions), the title, status and tags, the
//! formatting toolbar, the Markdown body and preview, and a footer band.
//!
//! In Split mode a draggable divider shares the editor between source and
//! preview, and the preview follows the source's scrolling loosely. In focus
//! mode the note sits alone in a centred column, and its header band and
//! toolbar wait for the pointer near the top.
//!
//! Edits autosave after a short pause. A change to the open note from
//! elsewhere (the API, sync) reloads it quietly when there are no unsaved
//! edits, and otherwise asks which version to keep. The losing version is
//! kept in the note's history either way. Leaving the note or quitting
//! before answering keeps the editor's text.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::EntityInputHandler as _;
use gpui_kit::assets::IconName as Lucide;
use gpui_kit::base::input::{
    Backspace, Copy, Cut, Enter, IndentInline, OutdentInline, Paste, SelectAll,
};
use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::{Button, ButtonGroup, ButtonVariants as _};
use gpui_kit::component::input::{Editor, EditorState, Input, InputEvent, InputState, Position};
use gpui_kit::component::menu::{ContextMenuExt as _, DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::native_menu::NativeMenu;
use gpui_kit::component::notification::Notification;
use gpui_kit::component::tag::Tag;
use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, ResizableState, Selectable as _, Sizable as _,
    StyledExt as _, h_flex, h_resizable, resizable_panel, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_core::{DocType, Note, NoteStatus};
use kanso_engine::saving::{AUTOSAVE_DELAY, same_content};
use kanso_store::{Backlink, Change, Origin, Put, Store, StoreError};

use crate::clock::Clock;
use crate::commands::Picker;
use crate::config::{Appearance, EditorFont, EditorSettings, ViewMode};
use crate::editor_footer::{EditorFooter, FooterEvent};
use crate::empty_state::EmptyNotebook;
use crate::layout::{BOTTOM_BAND_HEIGHT, TOP_BAND_HEIGHT, leading_inset};
use crate::library::{Filter, Library};
use crate::locked::LockView;
use crate::markdown_edit::{self, Edit, Format};
use crate::models::note_visit::NotePlace;
use crate::note_meaning::NoteMeaning;
use crate::pairing::Pairs;
use crate::preview::{ImageResize, LinkSender, Preview, PreviewEdit, TEXT_INSET, readable_width};
use crate::preview_tasks::{self, TaskTick};
use crate::smart_paste;
use crate::spellcheck::{
    self, CorrectSpelling, IgnoreSpelling, LearnSpelling, Misspelling, SpellMarks,
};
use crate::split::{
    scroll_sync, source_scroll_max, split_left, split_min_side, split_ratio, wrapped_rows,
};
use crate::table_format;
use crate::text_stats::{line_lengths, pieces_eq};
use crate::theme::{Colors, muted_tag_color, status_glyph};
use crate::wikilinks::{SharedIndex, WikiCompletion, WikiIndex, rewrite_for_preview};
use crate::word_goal::{self, GoalSetting};
use crate::workspace::{AppState, NewNote};
use crate::zoom::px;
use inline_glue::InlineAssist;

#[path = "editor_collab.rs"]
mod collab_glue;
#[path = "editor_format.rs"]
pub(crate) mod format_keys;
#[cfg(test)]
#[path = "editor_format_tests.rs"]
mod format_tests;
#[path = "ai_inline.rs"]
pub(crate) mod inline_glue;
#[cfg(test)]
#[path = "ai_inline_tests.rs"]
mod inline_tests;
#[path = "editor_peers.rs"]
pub(crate) mod peers;
#[cfg(test)]
#[path = "spellcheck_tests.rs"]
mod spellcheck_tests;
#[path = "editor_vim.rs"]
mod vim_glue;
#[cfg(test)]
#[path = "editor_vim_tests.rs"]
mod vim_tests;

/// While typing goes on, the preview beside the source catches up at most
/// this often: the first change shows at once, later ones on the next tick.
const PREVIEW_INTERVAL: Duration = Duration::from_millis(60);
/// Line height of the Markdown source, relative to its size. Every face and
/// view mode sets the source at the chosen size with this line height:
/// beside the preview it wraps sooner rather than getting smaller.
pub(crate) const BODY_LINE_HEIGHT: f32 = 1.6;
/// Where the source editor's text starts inside it, without line numbers:
/// its padding and the gap its gutter keeps. The pane's padding makes up the
/// rest of [`TEXT_INSET`].
const SOURCE_TEXT_LEFT: f32 = 12.;
/// What the source editor adds around its text beyond the readable measure:
/// that, its right padding, and the line number column when it shows.
const SOURCE_EDGES: f32 = SOURCE_TEXT_LEFT + 10.;
const LINE_NUMBER_COLUMN: f32 = 44.;
/// One line of the header's save state; a second line is clipped away.
const SAVE_LABEL_HEIGHT: f32 = 20.;
/// The widest column the note keeps in focus mode, each side of it in
/// Split.
const FOCUS_COLUMN: f32 = 760.;
/// The top of the editor in focus mode: hovering it shows the header.
const TOP_GROUP: &str = "editor-top";
/// About how wide a character of the source is, in ems, to estimate where
/// its lines wrap: SF Mono's advance, and a little under SF Pro's average
/// in prose. Under rather than over, so the preview reaches its end no
/// later than the source.
const MONO_EM: f32 = 0.6;
const SANS_EM: f32 = 0.44;
/// Typewriter scrolling: the caret keeps this many lines from the top and
/// bottom, which the editor caps at half its height, so the caret's line
/// stays in the middle.
const TYPEWRITER_LINES: usize = 10_000;
/// How long changes to notes settle before the titles wiki-links resolve
/// against are read again.
const WIKI_REFRESH_DELAY: Duration = Duration::from_millis(400);

pub enum EditorEvent {
    /// Something worth telling the user, such as a failed save.
    Notice(String),
    /// The history button: the workspace opens the note's history.
    ShowHistory,
    /// A choice from the note's "More" menu.
    Command(crate::note_menu::NoteCommand),
    /// The notebook path or "Add tag": the workspace opens the palette's
    /// list of notebooks or tags, which filters as you type and makes new
    /// tags.
    Pick(Picker),
    /// The backlinks button: the inspector opens on its Backlinks tab.
    ShowBacklinks,
    /// The inspector button: the inspector opens or closes.
    ToggleInspector,
    /// The editor let go of a note deleted elsewhere, after the reader
    /// chose the version from elsewhere: its tab goes with it.
    Closed(String),
    /// The note was renamed from `old` to `new` while `links` wiki-links in
    /// `notes` other notes named it by its old title: the workspace offers
    /// to update them.
    Renamed {
        id: String,
        old: String,
        new: String,
        links: usize,
        notes: usize,
    },
}

/// The ‹ › beside a daily note's title: the day before and the day after
/// that have notes (or today); and the month's calendar.
fn day_arrows(tab_stop: bool, cx: &App) -> impl IntoElement {
    let colors = Colors::get(cx);
    let arrow = |id: &'static str, icon: Lucide, tooltip: &'static str, forward: bool| {
        Button::new(id)
            .ghost()
            .xsmall()
            .tab_stop(tab_stop)
            .debug_selector(move || id.into())
            .icon(Icon::new(icon).text_color(colors.text_secondary))
            .tooltip(tooltip)
            .on_click(move |_, window, cx| {
                if forward {
                    window.dispatch_action(Box::new(crate::journal::NextDay), cx)
                } else {
                    window.dispatch_action(Box::new(crate::journal::PreviousDay), cx)
                }
            })
    };
    h_flex()
        .flex_none()
        .ml(px(6.))
        .gap(px(1.))
        .child(arrow(
            "day-previous",
            Lucide::ChevronLeft,
            "Previous day",
            false,
        ))
        .child(arrow("day-next", Lucide::ChevronRight, "Next day", true))
        .child(
            Button::new("day-calendar")
                .ghost()
                .xsmall()
                .tab_stop(tab_stop)
                .debug_selector(|| "day-calendar".into())
                .icon(
                    crate::icon_themes::library_icon(crate::icon_themes::Slot::Calendar)
                        .text_color(colors.text_secondary),
                )
                .tooltip("Journal calendar")
                .on_click(|_, window, cx| {
                    window.dispatch_action(Box::new(crate::calendar_view::ShowCalendar), cx)
                }),
        )
}

/// What happened elsewhere to the open note while it had unsaved edits.
enum Incoming {
    /// A newer version was saved.
    Changed(Box<Note>),
    /// The note was deleted.
    Deleted,
}

pub struct NoteEditor {
    store: Arc<Store>,
    library: Entity<Library>,
    /// The note as last loaded or saved, with its current `_rev`. A locked
    /// note that is open has its text in `body` here; the store has it
    /// sealed.
    note: Option<Note>,
    /// Whether the open note is locked, and if so whether it is open.
    lock: LockView,
    /// The next write keeps the note's edit time (publishing isn't editing).
    keep_edit_time: bool,
    title: Entity<InputState>,
    body: Entity<EditorState>,
    preview: Entity<Preview>,
    footer: Entity<EditorFooter>,
    pub view_mode: ViewMode,
    pub editor_font: EditorFont,
    pub font_size: f32,
    /// The installed fonts picked for prose and for the monospace face
    /// (settings); `None` for Kanso's own.
    pub font_family: Option<String>,
    pub mono_family: Option<String>,
    /// Whether monospace text may show line numbers (a setting).
    pub allow_line_numbers: bool,
    /// Whether the text keeps to a readable measure (a setting), or fills
    /// the pane.
    pub readable_width: bool,
    /// Whether the body shows line numbers now; see [`EditorFont::face`].
    line_numbers: bool,
    autosave: Option<Task<()>>,
    /// The preview's interval, while typing goes on.
    preview_task: Option<Task<()>>,
    /// Whether the source changed since the preview was last handed it.
    preview_stale: bool,
    /// A change from elsewhere that arrived over unsaved edits.
    incoming: Option<Incoming>,
    /// Notes that link here, with where.
    backlinks: Vec<Backlink>,
    backlinks_task: Option<Task<()>>,
    /// The titles wiki-links resolve against, shared with autocomplete.
    wiki: SharedIndex,
    wiki_task: Option<Task<()>>,
    /// A line to show once note `.0` opens, as a backlink or heading asks.
    pending_reveal: Option<(String, u32)>,
    /// A byte offset of note `.0`'s text to put the caret at once it
    /// opens: where a template's `{{cursor}}` was.
    pending_caret: Option<(String, usize)>,
    /// Where to put the caret and scroll back to once note `.0` opens, on
    /// going back or forward to it.
    pending_place: Option<(String, NotePlace)>,
    /// Where the reader was in the note last switched away from, for the
    /// workspace's back-and-forward history to keep.
    left_place: Option<(String, NotePlace)>,
    /// The note's saved title when its title field got the keyboard, to
    /// tell a rename when it loses it.
    title_on_focus: Option<(String, String)>,
    /// Counts the body's edits and loads, so the outline can tell its text
    /// changed.
    text_version: u64,
    /// Whether the inspector beside the editor is open, for its button.
    pub inspector_open: bool,
    /// Where the empty-notebook block sends its actions from while no note
    /// is open.
    pub(crate) empty_focus: FocusHandle,
    /// The note alone, centred, with its header shown on hover.
    focus_mode: bool,
    /// The source's share of the editor in Split mode, as last dragged.
    split_ratio: f32,
    pub(crate) split: Entity<ResizableState>,
    /// The editor width the divider was last placed for.
    split_placed_for: Pixels,
    /// The source's scroll position the preview last followed, and the
    /// text (by its generation) the preview showed then.
    followed_scroll: Option<(Pixels, u64)>,
    /// The source's line lengths in characters, for the text of the byte
    /// length kept beside them: an edit clears them, and a new text set
    /// without one (opening a note, a reload) has another length or the
    /// same lines.
    source_lines: Option<(usize, Vec<usize>)>,
    /// The writing aids from Settings › Editor.
    settings: EditorSettings,
    /// The pairs typed in the open note that are still open.
    pairs: Pairs,
    /// Words voice input has heard so far, shown at the caret as text
    /// being composed until the transcript replaces them.
    live_voice: Option<LiveVoice>,
    /// Whether the caret's line is kept in the middle now.
    typewriter: bool,
    /// Vim keys, while the setting is on.
    vim: Option<vim_glue::VimState>,
    /// The inline assistant's suggestion, while there is one.
    inline: Option<inline_glue::Suggestion>,
    /// The tints on tasks' due dates in the text.
    due_marks: crate::due::DueMarks,
    /// The underlines under misspelled words in the text.
    spell_marks: SpellMarks,
    /// The shared note open, attached to the engine (`editor_collab.rs`).
    shared: Option<crate::collab_model::OpenShare>,
    /// Effective read-only state for the title and text inputs.
    text_read_only: bool,
    /// Read-only because account unlock and binding consent are incomplete.
    account_read_only: bool,
    /// The shared-note role's read-only state, independent of the account gate.
    collab_read_only: bool,
    /// What related notes say about the open note: suggested tags, and a
    /// note that says nearly the same.
    meaning: Entity<NoteMeaning>,
    /// The text last taken from another editor of the same note
    /// (`editor_peers.rs`), to tell its echo from typing.
    peer_echo: Option<(String, String)>,
    /// The text shown came from another editor, which saves it.
    from_peer: bool,
    /// In a note's own window (`note_window.rs`): the leftmost pane, with
    /// no inspector beside it.
    standalone: bool,
    /// Shows the hidden header in focus mode, as a hover would.
    #[cfg(test)]
    pub(crate) reveal_chrome: bool,
    _subscriptions: Vec<Subscription>,
    _resizes: Task<()>,
}

impl EventEmitter<EditorEvent> for NoteEditor {}

impl NoteEditor {
    pub fn new(
        store: Arc<Store>,
        library: Entity<Library>,
        links: LinkSender,
        appearance: &Appearance,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let Appearance {
            view_mode,
            editor_font,
            editor_font_size: font_size,
            line_numbers: allow_line_numbers,
            readable_width,
            ..
        } = *appearance;
        let font_family = appearance.editor_font_family.clone();
        let mono_family = appearance.editor_mono_family.clone();
        let title = cx.new(|cx| InputState::new(window, cx).placeholder("Untitled"));
        let line_numbers = editor_font
            .face(view_mode)
            .with_line_numbers(allow_line_numbers)
            .line_numbers;
        let body = cx.new(|cx| {
            EditorState::new(window, cx)
                .language("markdown")
                .line_number(line_numbers)
                // Folding comes with line numbers: its column is what keeps
                // the numbers clear of the text.
                .folding(line_numbers)
                .indent_guides(false)
                .soft_wrap(true)
                // Kanso pairs on its own; see `pairing`.
                .auto_close(false)
        });
        let wiki: SharedIndex = Rc::new(RefCell::new(Arc::new(WikiIndex::default())));
        let meaning = cx.new(|cx| NoteMeaning::new(store.clone(), cx));
        body.update(cx, |state, _| {
            // `#tags` beside `[[` wiki-links: the editor takes one provider.
            state.lsp_mut().completion_provider =
                Some(Rc::new(crate::tag_complete::NoteCompletion::new(
                    WikiCompletion::new(wiki.clone()),
                    library.clone(),
                    meaning.clone(),
                )));
        });
        let settings = cx
            .try_global::<AppState>()
            .map_or_else(EditorSettings::default, |state| state.config.editor);
        let (resizes, resized) = async_channel::unbounded::<PreviewEdit>();
        let preview = cx.new(|cx| Preview::new(store.clone(), links, resizes, cx));
        // A size picked for an image in the preview rewrites its Markdown,
        // as does a drawing brought back from Excalidraw.
        let resize_task = cx.spawn_in(window, async move |this, cx| {
            while let Ok(edit) = resized.recv().await {
                if this
                    .update_in(cx, |this, window, cx| match edit {
                        PreviewEdit::Resize(resize) => this.resize_image(resize, window, cx),
                        PreviewEdit::Replace {
                            source,
                            replacement,
                        } => this.replace_block(&source, replacement, window, cx),
                        PreviewEdit::Task(tick) => this.tick_task(&tick, window, cx),
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        let split = cx.new(|_| ResizableState::default());
        let split_ratio = cx
            .try_global::<AppState>()
            .map_or(crate::config::DEFAULT_SPLIT_RATIO, |state| {
                state.config.layout.split_ratio
            });
        let footer = cx.new(|cx| {
            let mut footer = EditorFooter::new(body.clone(), cx);
            footer.set_show_cursor(view_mode != ViewMode::Preview, cx);
            footer.set_goal(GoalSetting::Default, settings.word_goal, cx);
            footer
        });
        let subscriptions = vec![
            cx.subscribe(&footer, |this, _, event: &FooterEvent, cx| match event {
                FooterEvent::Goal(goal) => this.set_word_goal(*goal, cx),
                FooterEvent::CustomGoal => cx.emit(EditorEvent::Pick(Picker::WordGoal)),
            }),
            cx.subscribe_in(&title, window, |this, _, event, window, cx| match event {
                // The breadcrumb shows the title as it's typed.
                InputEvent::Change => {
                    if !this.peer_change(cx) {
                        this.schedule_autosave(cx);
                    }
                    cx.notify();
                }
                InputEvent::PressEnter { .. } => {
                    this.body.update(cx, |body, cx| body.focus(window, cx));
                    this.title_settled(cx);
                }
                InputEvent::Focus => {
                    this.title_on_focus =
                        this.note.as_ref().map(|n| (n.id.clone(), n.title.clone()));
                }
                InputEvent::Blur => this.title_settled(cx),
            }),
            cx.subscribe_in(&body, window, |this, _, event, window, cx| {
                if matches!(event, InputEvent::Change) {
                    if let Some(vim) = this.vim.as_mut() {
                        vim.count_edit();
                    }
                    this.text_version += 1;
                    this.source_lines = None;
                    // Text from another editor of the note is saved there.
                    if !this.peer_change(cx) {
                        this.schedule_autosave(cx);
                    }
                    this.schedule_preview(window, cx);
                    let (body, me) = (this.body.clone(), cx.entity().downgrade());
                    this.due_marks.refresh_soon(
                        body,
                        me,
                        |editor: &mut NoteEditor| &mut editor.due_marks,
                        cx,
                    );
                    this.check_spelling(spellcheck::CHECK_DELAY, cx);
                }
            }),
            // The text editor notifies when the wheel scrolls it (and when
            // its caret blinks): the preview beside it follows at once. Other
            // scrolling is caught as the split is painted.
            cx.observe(&body, |this, body, cx| {
                this.follow_source_scroll(cx);
                this.collab_caret(cx);
                let me = cx.entity().downgrade();
                this.spell_marks.follow_scroll(
                    body,
                    me,
                    |editor: &mut NoteEditor| &mut editor.spell_marks,
                    cx,
                );
            }),
            // Quitting from the Dock, at logout or from the menu.
            cx.on_app_quit(|this, cx| {
                this.flush(cx);
                async {}
            }),
            // With no note open, the pane follows the list: an empty
            // notebook offers its first note.
            cx.observe(&library, |this, _, cx| {
                if this.note.is_none() {
                    cx.notify();
                }
            }),
        ];
        let mut editor = NoteEditor {
            store,
            library,
            note: None,
            lock: LockView::Plain,
            keep_edit_time: false,
            title,
            body,
            preview,
            footer,
            view_mode,
            editor_font,
            font_size,
            font_family,
            mono_family,
            allow_line_numbers,
            readable_width,
            line_numbers,
            autosave: None,
            preview_task: None,
            preview_stale: false,
            incoming: None,
            backlinks: Vec::new(),
            backlinks_task: None,
            wiki,
            wiki_task: None,
            pending_reveal: None,
            pending_caret: None,
            pending_place: None,
            left_place: None,
            title_on_focus: None,
            text_version: 0,
            inspector_open: false,
            empty_focus: cx.focus_handle(),
            focus_mode: false,
            split_ratio,
            split,
            split_placed_for: px(0.),
            followed_scroll: None,
            source_lines: None,
            settings,
            pairs: Pairs::default(),
            live_voice: None,
            typewriter: false,
            vim: None,
            inline: None,
            due_marks: Default::default(),
            spell_marks: Default::default(),
            shared: None,
            text_read_only: false,
            account_read_only: false,
            collab_read_only: false,
            meaning,
            peer_echo: None,
            from_peer: false,
            standalone: false,
            #[cfg(test)]
            reveal_chrome: false,
            _subscriptions: subscriptions,
            _resizes: resize_task,
        };
        editor.set_vim(settings.vim, cx);
        editor.spell_marks.set_enabled(settings.spellcheck, cx);
        peers::register(cx.entity().downgrade(), window.window_handle(), cx);
        editor
    }

    /// Puts the editor in a note's own window: it is the leftmost pane, so
    /// its top band leaves the traffic lights their room, and there is no
    /// inspector to open.
    pub fn set_standalone(&mut self, cx: &mut Context<Self>) {
        self.standalone = true;
        cx.notify();
    }

    /// The writing aids as Settings › Editor has them now.
    pub fn set_settings(&mut self, settings: EditorSettings, cx: &mut Context<Self>) {
        if self.settings == settings {
            return;
        }
        self.settings = settings;
        if !settings.auto_pair {
            self.pairs.clear();
        }
        self.set_vim(settings.vim, cx);
        self.spell_marks.set_enabled(settings.spellcheck, cx);
        self.check_spelling(Duration::ZERO, cx);
        self.update_footer_goal(cx);
        cx.notify();
    }

    /// Checks the spelling of the text in view after `delay`.
    fn check_spelling(&mut self, delay: Duration, cx: &mut Context<Self>) {
        let (body, me) = (self.body.clone(), cx.entity().downgrade());
        self.spell_marks.refresh_soon(
            delay,
            body,
            me,
            |editor: &mut NoteEditor| &mut editor.spell_marks,
            cx,
        );
    }

    /// The open note's word goal: its own, the default, or none.
    pub fn word_goal(&self) -> GoalSetting {
        self.note
            .as_ref()
            .map_or(GoalSetting::Default, word_goal::note_goal)
    }

    /// The default word goal from Settings.
    pub fn default_word_goal(&self) -> Option<u32> {
        self.settings.word_goal
    }

    /// Gives the open note its own word goal (or takes it back to the
    /// default), saved with the note.
    pub fn set_word_goal(&mut self, goal: GoalSetting, cx: &mut Context<Self>) {
        if self.note.is_none() || self.word_goal() == goal {
            return;
        }
        self.write(
            move |note, _| word_goal::set_note_goal(note, goal),
            Origin::Ui,
            cx,
        );
        self.update_footer_goal(cx);
    }

    /// What the Publish sheet works from: the open note's ID, its title and
    /// body as typed, its revision, and the link it keeps, if any.
    pub fn publish_source(&self, cx: &App) -> Option<kanso_engine::publish::Source> {
        let note = self.note.as_ref()?;
        let (title, body) = self.current_text(cx);
        Some(kanso_engine::publish::Source {
            id: note.id.clone(),
            title,
            body,
            rev: note.rev.clone().unwrap_or_default(),
            publish: kanso_sync::share::KansoPublish::from_extra(&note.extra),
        })
    }

    /// Keeps `publish` in note `id` (or takes it out), saved with the note,
    /// when it is the open one. Returns whether it was.
    pub fn set_publish(
        &mut self,
        id: &str,
        publish: Option<kanso_sync::share::KansoPublish>,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.note_id() != Some(id) {
            return false;
        }
        // The sheet saved any edits first, so this write only adds the link.
        self.keep_edit_time = !self.has_unsaved_edits(cx);
        self.write(
            move |note, _| {
                kanso_sync::share::KansoPublish::store(publish.as_ref(), &mut note.extra)
            },
            Origin::Ui,
            cx,
        )
    }

    fn update_footer_goal(&mut self, cx: &mut Context<Self>) {
        let (goal, default) = (self.word_goal(), self.settings.word_goal);
        self.footer
            .update(cx, |footer, cx| footer.set_goal(goal, default, cx));
    }

    /// Focus mode on or off, as the workspace has it.
    pub fn set_focus_mode(&mut self, on: bool, cx: &mut Context<Self>) {
        if self.focus_mode != on {
            self.focus_mode = on;
            self.footer
                .update(cx, |footer, cx| footer.set_quiet(on, cx));
            cx.notify();
        }
    }

    /// Puts the keyboard in the note's text for writing, unless the title
    /// or the text already has it. The preview alone has nothing to type in.
    // no-prop-test: three conditions; the focus mode window test checks the
    // keyboard lands in the text.
    pub fn focus_for_writing(&self, window: &mut Window, cx: &mut Context<Self>) {
        let in_note = self.title.read(cx).focus_handle(cx).is_focused(window)
            || self.body.read(cx).focus_handle(cx).is_focused(window);
        if self.note.is_some() && !in_note && self.view_mode != ViewMode::Preview {
            self.focus_body(window, cx);
        }
    }

    /// Whether focus mode hides the header band until the pointer comes
    /// near the top (the status, tags and toolbar don't show in focus mode
    /// at all). Hidden, its controls leave the Tab order, so the keyboard
    /// never lands on one nobody can see; every one of them is also in the
    /// palette, which focus mode keeps.
    fn chrome_hidden(&self) -> bool {
        self.focus_mode && !self.chrome_revealed()
    }

    /// Whether the header band shows in focus mode without a hover: only in
    /// pictures of the hovered state.
    fn chrome_revealed(&self) -> bool {
        #[cfg(test)]
        {
            self.reveal_chrome
        }
        #[cfg(not(test))]
        {
            false
        }
    }

    /// The divider was dragged: keep where it was left, as a share of the
    /// editor, for this and the next launch.
    // no-prop-test: glue; `split_ratio` has the properties and
    // `a_dragged_divider_is_kept_and_both_sides_keep_room` drags it.
    fn split_dragged(&mut self, left: Option<Pixels>, width: Pixels, cx: &mut Context<Self>) {
        let Some(left) = left else {
            return;
        };
        let ratio = split_ratio(f32::from(left), f32::from(width));
        self.split_placed_for = width;
        if (ratio - self.split_ratio).abs() > 1e-4 {
            self.split_ratio = ratio;
            if cx.has_global::<AppState>() {
                AppState::update_config(cx, |config| config.layout.split_ratio = ratio);
            }
        }
    }

    /// Puts the divider at the kept share of an editor whose width changed
    /// (or was first measured), keeping each side its minimum. While the
    /// width holds, a drag moves it freely.
    // no-prop-test: glue; `split_left` has the properties and the drag test
    // checks kept shares are placed.
    fn place_split(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let state = self.split.read(cx);
        let width = state.container_size();
        if width <= px(0.) || width == self.split_placed_for {
            return;
        }
        self.split_placed_for = width;
        // In the layout's units, as the sides' minimums are.
        let target = split_left(self.split_ratio, crate::zoom::unscaled(width));
        let current = state
            .sizes()
            .first()
            .map(|size| crate::zoom::unscaled(*size));
        if current.is_some_and(|current| (current - target).abs() > 0.5) {
            self.split.update(cx, |state, cx| {
                state.resize_panel(0, px(target), window, cx)
            });
        }
    }

    /// Scrolls the preview beside the source to the same share of its
    /// length as the source shows. The source's full height isn't known,
    /// so it is estimated from its lines and width; a loose match is enough.
    // no-prop-test: glue; `scroll_sync` has the properties and
    // `the_preview_follows_the_source` scrolls the real views.
    fn follow_source_scroll(&mut self, cx: &mut Context<Self>) {
        if self.view_mode != ViewMode::Split || self.note.is_none() {
            self.followed_scroll = None;
            return;
        }
        let offset = self.body.read(cx).scroll_offset().y;
        // A new text moves what is where in the preview, so it follows
        // again even with the source where it was.
        let generation = self.preview.read(cx).generation();
        if self.followed_scroll == Some((offset, generation)) {
            return;
        }
        let src = -f32::from(offset);
        let dst_max = self.preview.read(cx).scroll_max();
        // A preview with no length yet (a long note parses in the background
        // once Split shows it) can't follow a scrolled source: it is tried
        // again on the next frame, rather than kept at the top as followed.
        if dst_max <= 0. && src > 0. {
            return;
        }
        let Some(src_max) = self.source_scroll_estimate(cx) else {
            return;
        };
        self.followed_scroll = Some((offset, generation));
        self.preview.update(cx, |preview, cx| {
            preview.scroll_to(scroll_sync(src, src_max, dst_max), cx);
        });
    }

    /// About how far the source can scroll, from its lines, its width and
    /// its face; `None` before it has been laid out.
    // no-prop-test: reads the editor's layout into `wrapped_rows` and
    // `source_scroll_max`, which have the properties.
    fn source_scroll_estimate(&mut self, cx: &App) -> Option<f32> {
        let body = self.body.read(cx);
        let line_height = body.line_height()?;
        let bounds = body.input_bounds();
        let numbers = if self.line_numbers {
            LINE_NUMBER_COLUMN
        } else {
            0.
        };
        let text_width = crate::zoom::unscaled(bounds.size.width) - numbers - SOURCE_EDGES;
        let em = if self.editor_font.face(self.view_mode).mono {
            MONO_EM
        } else {
            SANS_EM
        };
        let columns = (text_width / (em * self.font_size)).floor().max(1.) as usize;
        let len = body.text().len();
        if self
            .source_lines
            .as_ref()
            .is_none_or(|(kept, _)| *kept != len)
        {
            let lines = line_lengths(body.text().chunks());
            self.source_lines = Some((len, lines));
        }
        let lines = self
            .source_lines
            .as_ref()
            .map(|(_, lines)| lines.as_slice());
        let rows = wrapped_rows(lines.unwrap_or_default().iter().copied(), columns);
        Some(source_scroll_max(
            rows,
            f32::from(line_height),
            f32::from(bounds.size.height),
        ))
    }

    /// What related notes say about the open note.
    pub fn meaning(&self) -> &Entity<NoteMeaning> {
        &self.meaning
    }

    pub fn note_id(&self) -> Option<&str> {
        self.note.as_ref().map(|n| n.id.as_str())
    }

    /// Saves the open note, then opens `id` (or nothing).
    pub fn open(&mut self, id: Option<&str>, window: &mut Window, cx: &mut Context<Self>) {
        if self.note_id() == id {
            return;
        }
        // While a Replace holds writes, edits can't be saved yet: the editor
        // stays on its note rather than drop them (A10).
        if writes_held(cx) && self.has_unsaved_edits(cx) {
            return;
        }
        self.flush(cx);
        self.left_place = self.place(cx);
        let note = match id.map(|id| self.store.note(id)) {
            Some(Ok(note)) => note,
            Some(Err(err)) => {
                cx.emit(EditorEvent::Notice(format!(
                    "Couldn't open the note: {err}"
                )));
                None
            }
            None => None,
        };
        self.show(note, false, window, cx);
        self.catch_up_with_peers(window, cx);
    }

    /// Puts `note` in the editor. With `keep_place`, the cursor and scroll
    /// position survive, for reloading the same note.
    fn show(
        &mut self,
        mut note: Option<Note>,
        keep_place: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.incoming = None;
        self.from_peer = false;
        self.lock = note
            .as_mut()
            .map_or(LockView::Plain, |note| crate::locked::reveal(note, cx));
        // A suggestion belongs to the note it was asked for, in the clear.
        if self.lock != LockView::Plain
            || self.inline.as_ref().is_some_and(|suggestion| {
                note.as_ref().map(|note| note.id.as_str()) != Some(suggestion.note_id())
            })
        {
            self.inline = None;
        }
        let (title, body) = note.as_ref().map_or((String::new(), String::new()), |n| {
            (n.title.clone(), n.body.clone())
        });
        self.note = note;
        // Loading isn't typing: the text set here goes to no other editor.
        self.peer_echo = Some((title.clone(), body.clone()));
        self.title.update(cx, |state, cx| {
            if state.value().as_ref() != title {
                state.set_value(title, window, cx);
            }
        });
        let had_focus = self.body.read(cx).focus_handle(cx).is_focused(window);
        self.body.update(cx, |state, cx| {
            if state.value().as_ref() == body {
                return;
            }
            let cursor = state.cursor_position();
            let scroll = state.scroll_offset();
            state.set_value(body.clone(), window, cx);
            if keep_place {
                if had_focus {
                    let row = cursor
                        .line
                        .min(body.lines().count().saturating_sub(1) as u32);
                    state.set_cursor_position(Position::new(row, cursor.character), window, cx);
                }
                state.set_scroll_offset(scroll, cx);
            }
        });
        self.due_marks.refresh(&self.body, cx);
        self.check_spelling(Duration::ZERO, cx);
        // The kept line lengths describe the old text, which may have had
        // the same length.
        self.source_lines = None;
        self.text_version += 1;
        let text = self.preview_source(body.into());
        self.preview_stale = false;
        let shown = self.note_id().map(str::to_owned);
        self.preview.update(cx, |preview, cx| {
            preview.set_note(shown, cx);
            preview.set_text(text, cx);
            // Another note starts at its top, beside a source that does.
            if !keep_place {
                preview.scroll_to(0., cx);
            }
        });
        if !keep_place {
            self.followed_scroll = None;
        }
        self.pairs.clear();
        // Another note starts in Normal mode. The same one reloaded keeps
        // its mode, though not the undo steps an Insert session would join.
        if keep_place {
            self.vim_forget_session();
        } else {
            self.vim_reset(cx);
        }
        let id = self.note_id().map(str::to_owned);
        self.meaning
            .update(cx, |meaning, cx| meaning.follow(id.as_deref(), cx));
        let has_note = self.note.is_some();
        self.footer
            .update(cx, |footer, cx| footer.note_opened(has_note, cx));
        self.update_footer_goal(cx);
        self.refresh_backlinks(cx);
        if let Some((id, line)) = self.pending_reveal.take()
            && self.note_id() == Some(id.as_str())
        {
            self.reveal_line(line, window, cx);
        }
        if let Some((id, offset)) = self.pending_caret.take()
            && self.note_id() == Some(id.as_str())
        {
            self.body.update(cx, |state, cx| {
                let offset = offset.min(state.text().len());
                let position =
                    gpui_kit::base::input::RopeExt::offset_to_position(state.text(), offset);
                state.set_cursor_position(position, window, cx);
            });
        }
        if let Some((id, place)) = self.pending_place.take()
            && self.note_id() == Some(id.as_str())
        {
            self.restore_place(place, window, cx);
        }
        self.collab_follow(window, cx);
        cx.notify();
    }

    pub fn focus_body(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.body.update(cx, |body, cx| body.focus(window, cx));
    }

    pub fn focus_title(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.title.update(cx, |title, cx| title.focus(window, cx));
    }

    /// The open note's title and body as they are now, for an export.
    pub fn export_text(&self, cx: &App) -> Option<(String, String)> {
        self.note.as_ref().map(|_| self.current_text(cx))
    }

    /// The open note as it is now: the editor's title and text over the
    /// note as last loaded or saved.
    pub fn current_note(&self, cx: &App) -> Option<Note> {
        let mut note = self.note.clone()?;
        (note.title, note.body) = self.current_text(cx);
        Some(note)
    }

    /// Whether the open note is locked, and open.
    pub fn lock_view(&self) -> LockView {
        self.lock.clone()
    }

    /// Locks the open note with `key`: saves it sealed and forgets its
    /// earlier versions. It stays open.
    pub fn lock_with(&mut self, key: &kanso_sync::lock::LockKey, cx: &mut Context<Self>) {
        self.flush(cx);
        if self.incoming.is_some() {
            self.notice("Choose which version to keep before locking the note.", cx);
            return;
        }
        let Some(mut note) = self.note.clone() else {
            return;
        };
        if note.is_locked() {
            return;
        }
        let body = std::mem::take(&mut note.body);
        kanso_sync::lock::Envelope::store(Some(&key.seal(&note.id, &body)), &mut note.extra);
        let result = serde_json::to_value(&note)
            .map_err(StoreError::from)
            .and_then(|value| self.store.lock_note(value));
        match result {
            Ok(result) => {
                note.rev = Some(result.rev);
                note.body = body;
                self.note = Some(note);
                self.lock = LockView::Open;
                self.inline = None;
                cx.notify();
            }
            Err(err) => self.notice(format!("Couldn't lock the note: {err}"), cx),
        }
    }

    /// Takes the open note's lock off: its text is saved as plain text.
    pub fn remove_lock(&mut self, cx: &mut Context<Self>) {
        if self.lock != LockView::Open {
            return;
        }
        if self.write(
            |note, _| kanso_sync::lock::Envelope::store(None, &mut note.extra),
            Origin::Ui,
            cx,
        ) {
            self.lock = LockView::Plain;
            cx.notify();
        }
    }

    /// The keys held changed: a locked note, or a note in a locked
    /// notebook, opens or closes to match.
    pub fn lock_keys_changed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self
            .note
            .as_ref()
            .filter(|n| n.is_locked() || self.store.note_is_locked(&n.id).unwrap_or(false))
            .map(|n| n.id.clone())
        else {
            return;
        };
        if let Ok(note) = self.store.note(&id) {
            self.show(note, true, window, cx);
        }
    }

    /// Something worth telling the user.
    pub fn notice(&self, message: impl Into<String>, cx: &mut Context<Self>) {
        cx.emit(EditorEvent::Notice(message.into()));
    }

    fn current_text(&self, cx: &App) -> (String, String) {
        (
            self.title.read(cx).value().to_string(),
            self.body.read(cx).value().to_string(),
        )
    }

    /// Whether the title or the body differ from the note's. The header
    /// asks on every frame once a save is done, so the body is compared a
    /// piece of its rope at a time rather than copied.
    pub fn has_unsaved_edits(&self, cx: &App) -> bool {
        self.note.as_ref().is_some_and(|note| {
            self.title.read(cx).value() != note.title.as_str()
                || !pieces_eq(self.body.read(cx).text().chunks(), &note.body)
        })
    }

    fn schedule_autosave(&mut self, cx: &mut Context<Self>) {
        // With the question about a change made elsewhere open, nothing is
        // written until it is answered.
        if self.note.is_none() || self.incoming.is_some() {
            return;
        }
        // A shared note's text goes through the engine, never the store.
        if self.collab_attached() {
            self.collab_edited(cx);
            return;
        }
        let was_saved = self.autosave.is_none();
        self.autosave = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(AUTOSAVE_DELAY).await;
            this.update(cx, |this, cx| {
                this.autosave = None;
                this.save_as(true, cx);
                // "Saving…" gives way even when there was nothing to write.
                cx.notify();
            })
            .ok();
        }));
        if was_saved {
            cx.notify();
        }
    }

    /// Hands the preview the source's text: at once for the first change,
    /// then at most every [`PREVIEW_INTERVAL`] while changes keep coming,
    /// and once more after the last, so it never lags a pause.
    fn schedule_preview(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.view_mode == ViewMode::Edit {
            return;
        }
        self.preview_stale = true;
        if self.preview_task.is_some() {
            return;
        }
        self.refresh_preview(cx);
        self.preview_task = Some(cx.spawn_in(window, async move |this, cx| {
            loop {
                cx.background_executor().timer(PREVIEW_INTERVAL).await;
                let more = this
                    .update(cx, |this, cx| {
                        if this.preview_stale {
                            this.refresh_preview(cx);
                            true
                        } else {
                            this.preview_task = None;
                            false
                        }
                    })
                    .unwrap_or(false);
                if !more {
                    break;
                }
            }
        }));
    }

    /// The preview's text size or font changed (a setting): it draws
    /// again, measuring its text anew.
    pub fn refresh_preview_type(&mut self, cx: &mut Context<Self>) {
        self.preview.update(cx, |_, cx| cx.notify());
    }

    fn refresh_preview(&mut self, cx: &mut Context<Self>) {
        self.preview_stale = false;
        let text = self.preview_source(self.body.read(cx).value());
        self.preview
            .update(cx, |preview, cx| preview.set_text(text, cx));
    }

    /// What the preview is handed for `text`: wiki-links written as
    /// ordinary links (see [`crate::wikilinks`]).
    fn preview_source(&self, text: SharedString) -> SharedString {
        crate::due::rewrite_for_preview(&rewrite_for_preview(&text, &self.wiki.borrow()))
    }

    /// Reads the titles wiki-links resolve against again, in the background
    /// once changes settle, and shows the preview again with them.
    pub fn refresh_wiki(&mut self, cx: &mut Context<Self>) {
        let store = self.store.clone();
        self.wiki_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(WIKI_REFRESH_DELAY).await;
            let titles = cx
                .background_executor()
                .spawn(async move { store.note_titles().unwrap_or_default() })
                .await;
            this.update(cx, |this, cx| {
                this.wiki_task = None;
                if this.wiki.borrow().titles == titles {
                    return;
                }
                *this.wiki.borrow_mut() = Arc::new(WikiIndex::new(titles));
                if this.view_mode != ViewMode::Edit && this.note.is_some() {
                    this.refresh_preview(cx);
                }
                cx.notify();
            })
            .ok();
        }));
    }

    /// The open note's ID and a count that changes whenever its text does,
    /// without reading the text: whether the outline is current.
    pub fn outline_version(&self) -> Option<(String, u64)> {
        Some((self.note_id()?.to_owned(), self.text_version))
    }

    /// The source text as it is now.
    pub fn body_text(&self, cx: &App) -> SharedString {
        self.body.read(cx).value()
    }

    /// The open note's ID, its text version and its text: what the outline
    /// is made from.
    pub fn outline_source(&self, cx: &App) -> Option<(String, u64, SharedString)> {
        let (id, version) = self.outline_version()?;
        Some((id, version, self.body_text(cx)))
    }

    /// The line (0-based) the caret is on.
    pub fn caret_line(&self, cx: &App) -> u32 {
        self.body.read(cx).cursor_position().line
    }

    /// Notes that link to the open one, as last found.
    pub fn backlinks(&self) -> &[Backlink] {
        &self.backlinks
    }

    /// Puts the caret at the start of `line` and brings it into view; in
    /// the preview alone, scrolls the preview to about the same place.
    pub fn reveal_line(&mut self, line: u32, window: &mut Window, cx: &mut Context<Self>) {
        if self.note.is_none() {
            return;
        }
        if self.view_mode == ViewMode::Preview {
            let lines = gpui_kit::base::input::RopeExt::lines_len(self.body.read(cx).text()).max(1);
            let share = line as f32 / lines as f32;
            self.preview.update(cx, |preview, cx| {
                let top = preview.scroll_max() * share;
                preview.scroll_to(top, cx);
            });
            return;
        }
        self.body.update(cx, |state, cx| {
            state.set_cursor_position(Position::new(line, 0), window, cx);
        });
    }

    /// The text selected in the note's body, if any.
    pub fn selected_text(&self, cx: &App) -> Option<String> {
        let state = self.body.read(cx);
        let range = state.selected_range();
        let value = state.value();
        (!range.is_empty())
            .then(|| value.get(range.start..range.end).map(str::to_owned))
            .flatten()
    }

    /// The selection a plugin may see: its byte range in the body and its
    /// text. None for a locked or read-only note, or with nothing selected.
    pub fn plugin_selection(&self, cx: &App) -> Option<(usize, usize, String)> {
        if self.note.is_none() || self.text_read_only {
            return None;
        }
        let range = self.body.read(cx).selected_range();
        let text = self.selected_text(cx)?;
        Some((range.start, range.end, text))
    }

    /// Replaces bytes `range` of note `id`'s body with `text` for a plugin,
    /// as one edit that undoes in one step, when the range still holds
    /// `seen` (what the plugin was shown). False, changing nothing, when
    /// another note is open now or the text there changed.
    pub fn replace_for_plugin(
        &mut self,
        id: &str,
        range: std::ops::Range<usize>,
        seen: &str,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.note_id() != Some(id) || self.text_read_only {
            return false;
        }
        self.body.update(cx, |state, cx| {
            let value = state.value();
            if value.get(range.clone()) != Some(seen) {
                return false;
            }
            state.set_selected_range(range.clone(), cx);
            state.replace(text.to_owned(), window, cx);
            let end = range.start + text.len();
            state.set_selected_range(range.start..end, cx);
            true
        })
    }

    /// Puts `body` in place of note `id`'s text for a plugin, as one edit
    /// that undoes in one step, saved as typing is. Only when the editor
    /// holds no edits unsaved since the plugin read the note: false, with
    /// nothing changed, otherwise.
    pub fn replace_body_for_plugin(
        &mut self,
        id: &str,
        body: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.autosave.is_some() || self.has_unsaved_edits(cx) {
            return false;
        }
        let current = self.body.read(cx).value().to_string();
        self.replace_for_plugin(id, 0..current.len(), &current, body, window, cx)
    }

    /// Shows `line` of note `id` once it opens.
    pub fn reveal_after_open(&mut self, id: String, line: u32) {
        self.pending_reveal = Some((id, line));
    }

    /// Puts the caret at byte `offset` of note `id`'s text once it opens.
    pub fn caret_after_open(&mut self, id: String, offset: usize) {
        self.pending_caret = Some((id, offset));
    }

    /// Where the reader is in the open note: its ID, the caret and how far
    /// the text and preview are scrolled.
    pub fn place(&self, cx: &App) -> Option<(String, NotePlace)> {
        let id = self.note_id()?.to_owned();
        let body = self.body.read(cx);
        let caret = body.cursor_position();
        let scroll = body.scroll_offset();
        let place = NotePlace {
            caret_line: caret.line,
            caret_column: caret.character,
            scroll_x: f32::from(scroll.x),
            scroll_y: f32::from(scroll.y),
            preview_top: self.preview.read(cx).scroll_top(),
        };
        Some((id, place))
    }

    /// Puts the caret and scroll back at `place` once note `id` opens.
    pub fn place_after_open(&mut self, id: String, place: NotePlace) {
        self.pending_place = Some((id, place));
    }

    /// Where the reader was in the note last switched away from, once.
    pub fn take_left_place(&mut self) -> Option<(String, NotePlace)> {
        self.left_place.take()
    }

    fn restore_place(&mut self, place: NotePlace, window: &mut Window, cx: &mut Context<Self>) {
        self.body.update(cx, |state, cx| {
            let lines = gpui_kit::base::input::RopeExt::lines_len(state.text()).max(1);
            let line = place.caret_line.min(lines as u32 - 1);
            state.set_cursor_position(Position::new(line, place.caret_column), window, cx);
            // Saved in window pixels, which already have the zoom in them.
            state.set_scroll_offset(
                point(gpui_kit::px(place.scroll_x), gpui_kit::px(place.scroll_y)),
                cx,
            );
        });
        self.preview
            .update(cx, |preview, cx| preview.scroll_to(place.preview_top, cx));
    }

    /// The title field lost the keyboard or took Enter: if that renamed the
    /// note and other notes name it by the old title in wiki-links, the
    /// workspace offers to update them.
    fn title_settled(&mut self, cx: &mut Context<Self>) {
        let Some((id, old)) = self.title_on_focus.take() else {
            return;
        };
        if self.note_id() != Some(id.as_str()) {
            return;
        }
        self.flush(cx);
        let Some(new) = self.note.as_ref().map(|n| n.title.clone()) else {
            return;
        };
        if let Some((links, notes)) =
            kanso_engine::links::relink_offer(&self.store, &id, &old, &new)
        {
            cx.emit(EditorEvent::Renamed {
                id,
                old,
                new,
                links,
                notes,
            });
        }
    }

    /// Saves any pending edits now. With the question about a change made
    /// elsewhere still open, this keeps the editor's text, so that leaving
    /// the note or quitting never loses it; the other version stays in
    /// history.
    pub fn flush(&mut self, cx: &mut Context<Self>) {
        if writes_held(cx) {
            return;
        }
        self.autosave = None;
        // Another writer can win the race between reading and writing; try
        // again on top of its version.
        for _ in 0..3 {
            if self.incoming.is_some() {
                self.keep_mine(cx);
            } else {
                self.save(cx);
            }
            if self.incoming.is_none() {
                return;
            }
        }
        self.stash_draft(cx);
    }

    /// Combines the account's locked-session gate with the current shared
    /// note role, and updates both text inputs when either changes.
    pub(super) fn refresh_text_read_only(&mut self, cx: &mut Context<Self>) {
        let read_only = self.account_read_only || self.collab_read_only;
        if self.text_read_only == read_only {
            return;
        }
        self.text_read_only = read_only;
        self.body
            .update(cx, |state, cx| state.set_readonly(read_only, cx));
        self.title
            .update(cx, |state, cx| state.set_readonly(read_only, cx));
        cx.notify();
    }

    fn set_account_read_only(&mut self, read_only: bool, cx: &mut Context<Self>) {
        if self.account_read_only == read_only {
            return;
        }
        self.account_read_only = read_only;
        self.refresh_text_read_only(cx);
    }

    /// Keeps the editor's text in the open note's history.
    fn stash_draft(&mut self, cx: &mut Context<Self>) {
        if writes_held(cx) {
            return;
        }
        let Some(note) = &self.note else {
            return;
        };
        let (title, body) = self.current_text(cx);
        let mut fields = serde_json::Map::new();
        // A locked note's draft is kept sealed, as its other versions are.
        let (body, lock) = if note.is_locked() && self.lock == LockView::Open {
            let mut sealed = Note {
                body,
                ..note.clone()
            };
            if crate::locked::seal(&mut sealed, cx).is_err() {
                return;
            }
            (
                String::new(),
                sealed.extra.get(kanso_core::model::LOCK_FIELD).cloned(),
            )
        } else if note.is_locked() {
            (String::new(), None)
        } else {
            (body, None)
        };
        fields.insert("title".into(), title.into());
        fields.insert("body".into(), body.into());
        fields.insert("bookId".into(), note.book_id.clone().into());
        if let Some(lock) = lock {
            fields.insert(kanso_core::model::LOCK_FIELD.into(), lock);
        }
        let rev = note.rev.clone().unwrap_or_default();
        if let Err(err) = self
            .store
            .record_revision(&note.id, &rev, &fields, Origin::Ui)
        {
            cx.emit(EditorEvent::Notice(format!(
                "Couldn't keep your draft: {err}"
            )));
        }
    }

    /// Writes the title and body if they changed since the last save. While
    /// a Replace holds writes, the edits wait (A10).
    fn save(&mut self, cx: &mut Context<Self>) {
        self.save_as(false, cx);
    }

    /// [`Self::save`]; `typing` for the autosave while the note is being
    /// written in, which leaves out a `#tag` the caret is still in.
    fn save_as(&mut self, typing: bool, cx: &mut Context<Self>) {
        if self.incoming.is_some() || !self.has_unsaved_edits(cx) || writes_held(cx) {
            return;
        }
        if self.collab_attached() {
            self.collab_sync_quiet(cx);
            return;
        }
        // Settings › Editor: `#tags` in the text become the note's tags.
        // A locked note's `#tags` stay in its text: tags show unlocked.
        let inline = (self.settings.inline_tags && self.lock == LockView::Plain).then(|| {
            let caret = typing.then(|| self.body.read(cx).cursor());
            (self.store.clone(), caret)
        });
        self.write(
            move |note, _| {
                if let Some((store, caret)) = inline {
                    crate::tag_complete::apply_inline_tags(&store, note, caret);
                }
            },
            Origin::Ui,
            cx,
        );
    }

    /// Saves what waited while writes were held. When the Replace took the
    /// note away, its edits are not written into the new library: the
    /// editor asks about them as it does for a note deleted elsewhere,
    /// whichever comes first, this or the store's change.
    pub fn save_held(&mut self, cx: &mut Context<Self>) {
        let Some(note) = &self.note else {
            return;
        };
        if self.incoming.is_none()
            && self.has_unsaved_edits(cx)
            && matches!(self.store.note(&note.id), Ok(None))
        {
            self.autosave = None;
            self.incoming = Some(Incoming::Deleted);
            cx.notify();
            return;
        }
        self.save(cx);
    }

    /// Writes the open note with the editor's text and `change` applied.
    fn write(
        &mut self,
        change: impl FnOnce(&mut Note, &App),
        origin: Origin,
        cx: &mut Context<Self>,
    ) -> bool {
        // A shared note's typing is merged first; the write then carries
        // the library's text, not the editor's.
        if self.collab_attached() {
            self.collab_sync_quiet(cx);
        }
        let Some(note) = self.note.clone() else {
            return false;
        };
        self.write_over(note, change, origin, cx)
    }

    /// Writes `base` with the editor's text and `change` applied. `self.note`
    /// only changes once the write succeeds.
    fn write_over(
        &mut self,
        base: Note,
        change: impl FnOnce(&mut Note, &App),
        origin: Origin,
        cx: &mut Context<Self>,
    ) -> bool {
        if writes_held(cx) {
            return false;
        }
        let mut note = base.clone();
        if !self.collab_attached() {
            let (title, body) = self.current_text(cx);
            note.title = title;
            note.body = body;
        }
        change(&mut note, cx);
        if !std::mem::take(&mut self.keep_edit_time) {
            note.updated_at = Clock::now_ms(cx);
        }
        // A note going into a locked notebook goes sealed, and is open.
        let mut sealed_now = false;
        if !note.is_locked()
            && self
                .store
                .locked_books()
                .is_ok_and(|books| books.contains(&note.book_id))
        {
            if let Err(words) = crate::locked::seal_for_notebook(&mut note, cx) {
                cx.emit(EditorEvent::Notice(format!("Couldn't save: {words}")));
                return false;
            }
            sealed_now = true;
        }
        let open = self.lock == LockView::Open || sealed_now;
        // A sync can move the revision between reading and writing without
        // changing the note: then the write goes on top of the new revision.
        let mut retried = false;
        loop {
            // A locked note goes to the store sealed; the editor keeps its
            // text. One that isn't open can't take any.
            let mut stored = note.clone();
            if stored.is_locked() {
                let sealed = if open {
                    crate::locked::seal(&mut stored, cx)
                } else if stored.body.is_empty() {
                    Ok(())
                } else {
                    Err("Unlock the note to change its text.".to_owned())
                };
                if let Err(words) = sealed {
                    cx.emit(EditorEvent::Notice(format!("Couldn't save: {words}")));
                    return false;
                }
            }
            let value = match serde_json::to_value(&stored) {
                Ok(value) => value,
                Err(err) => {
                    cx.emit(EditorEvent::Notice(format!("Couldn't save: {err}")));
                    return false;
                }
            };
            match self.store.put(DocType::Note, value, Put::Upsert, origin) {
                Ok(result) => {
                    note.rev = Some(result.rev);
                    self.note = Some(note);
                    self.from_peer = false;
                    if sealed_now {
                        self.lock = LockView::Open;
                    }
                    // The write was based on the current version, so there is
                    // nothing left to choose between.
                    self.incoming = None;
                    cx.notify();
                    return true;
                }
                Err(StoreError::Conflict) => {
                    let Ok(Some(mut latest)) = self.store.note(&note.id) else {
                        return false;
                    };
                    crate::locked::reveal(&mut latest, cx);
                    // Another editor of this note saved this very text
                    // first (`editor_peers.rs`): its version is this one.
                    if peers::writes_the_same(&note, &latest) {
                        self.note = Some(latest);
                        self.from_peer = false;
                        self.incoming = None;
                        cx.notify();
                        return true;
                    }
                    if !retried
                        && (same_content(&base, &latest)
                            || peers::text_caught_up(&base, &latest, &note))
                    {
                        retried = true;
                        note.rev = latest.rev;
                        continue;
                    }
                    // Someone else saved first; show their version next to ours.
                    self.incoming = Some(Incoming::Changed(Box::new(latest)));
                    cx.notify();
                    return false;
                }
                Err(err) => {
                    cx.emit(EditorEvent::Notice(format!("Couldn't save: {err}")));
                    return false;
                }
            }
        }
    }

    /// Reacts to a write to the open note made outside this editor.
    pub fn on_store_change(
        &mut self,
        change: &Change,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(note) = &self.note else {
            return;
        };
        if change.id != note.id || note.rev.as_deref() == Some(change.rev.as_str()) {
            return;
        }
        if self.collab_attached() {
            self.collab_store_change(change.deleted, window, cx);
            return;
        }
        let incoming = if change.deleted {
            Incoming::Deleted
        } else {
            let Ok(Some(mut latest)) = self.store.note(&change.id) else {
                return;
            };
            crate::locked::reveal(&mut latest, cx);
            if same_content(note, &latest) {
                // Only the revision moved (a sync): the editor takes it and
                // carries on, unsaved typing and all.
                if let Some(note) = &mut self.note {
                    note.rev = latest.rev;
                }
                return;
            }
            // Another editor of the note saved what this one shows.
            if self.adopt_if_same_text(&latest, cx) {
                return;
            }
            Incoming::Changed(Box::new(latest))
        };
        if self.has_unsaved_edits(cx) {
            self.autosave = None;
            self.incoming = Some(incoming);
            cx.notify();
        } else {
            match incoming {
                Incoming::Changed(latest) => self.show(Some(*latest), true, window, cx),
                Incoming::Deleted => self.show(None, false, window, cx),
            }
        }
    }

    /// Keeps the editor's text over the change made elsewhere. A newer
    /// version stays in history; a deleted note comes back.
    pub fn keep_mine(&mut self, cx: &mut Context<Self>) {
        let Some(incoming) = self.incoming.take() else {
            return;
        };
        let base = match &incoming {
            Incoming::Changed(theirs) => (**theirs).clone(),
            Incoming::Deleted => match self.note.clone() {
                Some(note) => note,
                None => return,
            },
        };
        // The note's notebook may have been deleted along with it.
        let store = self.store.clone();
        let fallback_book = self.library.read(cx).default_book();
        let rehome = move |note: &mut Note, _: &App| {
            if note.book_id != TRASH_BOOK_ID
                && store.get(&note.book_id).ok().flatten().is_none()
                && let Some(book) = fallback_book
            {
                note.book_id = book;
            }
        };
        // An API-origin write always snapshots the version it replaces.
        if !self.write_over(base, rehome, Origin::Api, cx) && self.incoming.is_none() {
            // Not a conflict: leave the question open rather than drop it.
            self.incoming = Some(incoming);
        }
    }

    /// Takes the change made elsewhere; the editor's unsaved text goes to
    /// history.
    pub fn take_theirs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(incoming) = self.incoming.take() else {
            return;
        };
        self.stash_draft(cx);
        match incoming {
            Incoming::Changed(theirs) => self.show(Some(*theirs), true, window, cx),
            Incoming::Deleted => {
                let closed = self.note.as_ref().map(|note| note.id.clone());
                self.show(None, false, window, cx);
                if let Some(id) = closed {
                    cx.emit(EditorEvent::Closed(id));
                }
            }
        }
    }

    pub fn set_book(&mut self, book_id: String, cx: &mut Context<Self>) {
        self.write(move |note, _| note.book_id = book_id, Origin::Ui, cx);
    }

    pub fn set_status(&mut self, status: NoteStatus, cx: &mut Context<Self>) {
        self.write(move |note, _| note.status = status, Origin::Ui, cx);
    }

    pub fn toggle_tag(&mut self, tag_id: String, cx: &mut Context<Self>) {
        self.write(
            move |note, _| match note.tags.iter().position(|t| *t == tag_id) {
                Some(ix) => {
                    note.tags.remove(ix);
                }
                None => note.tags.push(tag_id),
            },
            Origin::Ui,
            cx,
        );
    }

    pub fn toggle_pinned(&mut self, cx: &mut Context<Self>) {
        self.write(|note, _| note.pinned = !note.pinned, Origin::Ui, cx);
    }

    /// Moves the open note to the trash.
    pub fn trash(&mut self, cx: &mut Context<Self>) {
        self.set_book(TRASH_BOOK_ID.into(), cx);
    }

    /// Starts the system's dictation into the note's text, at the cursor.
    /// The text must have the keyboard, and be drawn with it, before the
    /// system starts listening: that is where dictation types. Preview has
    /// no text to type into, so it gives way to Split.
    pub fn dictate(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.note.is_none() || !crate::dictation::AVAILABLE {
            return;
        }
        if self.view_mode == ViewMode::Preview {
            self.set_view_mode(ViewMode::Split, cx);
        }
        self.focus_body(window, cx);
        // Two frames on: the first draws the text with the keyboard, which
        // is when it starts taking typed text.
        window.on_next_frame(|window, _| {
            window.on_next_frame(|_, _| crate::dictation::start());
            window.refresh();
        });
        window.refresh();
    }

    /// Puts what was said at the caret (in place of a selection) as one
    /// step that undoes at once, with the keyboard back in the note.
    /// Whether there was a note to put it in.
    // no-prop-test: glue over `voice_edit`, which has the properties.
    pub fn insert_transcript(
        &mut self,
        transcript: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.note.is_none() {
            return false;
        }
        if self.view_mode == ViewMode::Preview {
            self.set_view_mode(ViewMode::Split, cx);
        }
        let commands = spoken_commands(cx);
        if let Some(live) = self.live_voice.take() {
            // The words shown while speaking become the transcript, in one
            // step that undoes at once.
            let Some(edit) = crate::voice_state::voice_edit(
                &live.base,
                live.selection.clone(),
                transcript,
                commands,
            ) else {
                // Commands took back everything said ("delete that").
                self.live_voice = Some(live);
                self.drop_live_transcript(window, cx);
                self.focus_body(window, cx);
                return true;
            };
            let committed = self.body.update(cx, |state, cx| {
                let Some(marked) = state.marked_text_range(window, cx) else {
                    return false;
                };
                let start = utf8_offset(&state.value(), marked.start);
                let caret = start + (edit.selection.start - edit.range.start);
                state.replace_text_in_range(None, &edit.text, window, cx);
                state.set_selected_range(caret..caret, cx);
                true
            });
            if committed {
                self.focus_body(window, cx);
                return true;
            }
            self.select_live_words(&live, cx);
        }
        self.smart_edit(window, cx, |_, text, selection| {
            crate::voice_state::voice_edit(text, selection, transcript, commands)
        });
        self.focus_body(window, cx);
        true
    }

    /// Shows `words`, heard so far, at the caret as text being composed;
    /// each call replaces the last. They stay out of the undo history
    /// until [`insert_transcript`](Self::insert_transcript) puts the
    /// transcript in their place. Typing or clicking ends the composing;
    /// later words then wait for the transcript.
    pub fn show_live_transcript(
        &mut self,
        words: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.note.is_none() || self.text_read_only || words.trim().is_empty() {
            return;
        }
        if self.live_voice.is_none() {
            if self.view_mode == ViewMode::Preview {
                self.set_view_mode(ViewMode::Split, cx);
            }
            let state = self.body.read(cx);
            self.live_voice = Some(LiveVoice {
                base: state.value().to_string(),
                selection: state.selected_range(),
                shown: String::new(),
                start: 0,
                lost: false,
            });
        }
        let Some(live) = self.live_voice.as_mut() else {
            return;
        };
        if live.lost {
            return;
        }
        let Some(edit) = crate::voice_state::voice_edit(
            &live.base,
            live.selection.clone(),
            words,
            spoken_commands(cx),
        ) else {
            // Commands took back everything said so far: nothing shows,
            // and the next words start the composing again.
            if !live.shown.is_empty() {
                let cleared = self.body.update(cx, |state, cx| {
                    if state.marked_text_range(window, cx).is_none() {
                        return false;
                    }
                    state.replace_and_mark_text_in_range(None, "", None, window, cx);
                    state.unmark_text(window, cx);
                    true
                });
                live.lost = !cleared;
                live.shown.clear();
                // The words had taken the selection's place: the next ones
                // go in at the caret left there.
                let at = live.selection.start.min(live.base.len());
                let end = live.selection.end.clamp(at, live.base.len());
                if live.base.is_char_boundary(at) && live.base.is_char_boundary(end) {
                    live.base.replace_range(at..end, "");
                    live.selection = at..at;
                }
            }
            cx.notify();
            return;
        };
        let first = live.shown.is_empty();
        let range16 = live.base[..edit.range.start].encode_utf16().count()
            ..live.base[..edit.range.end].encode_utf16().count();
        let caret16 = edit.text[..edit.selection.start - edit.range.start]
            .encode_utf16()
            .count();
        let shown = self.body.update(cx, |state, cx| {
            if !first && state.marked_text_range(window, cx).is_none() {
                return false;
            }
            state.replace_and_mark_text_in_range(
                first.then_some(range16),
                &edit.text,
                Some(caret16..caret16),
                window,
                cx,
            );
            true
        });
        if shown {
            live.shown = edit.text;
            live.start = edit.range.start;
        } else {
            live.lost = true;
        }
        cx.notify();
    }

    /// Takes the words shown while speaking back out: voice input was
    /// dropped, or heard nothing it could use.
    pub fn drop_live_transcript(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(live) = self.live_voice.take() else {
            return;
        };
        let composing = self.body.update(cx, |state, cx| {
            state.marked_text_range(window, cx).is_some()
        });
        if composing {
            self.body.update(cx, |state, cx| {
                state.replace_and_mark_text_in_range(None, "", None, window, cx);
                state.unmark_text(window, cx);
            });
        } else if self.select_live_words(&live, cx) {
            self.body
                .update(cx, |state, cx| state.replace("", window, cx));
        }
    }

    /// Selects the live words where they are now, if they can be found
    /// (the composing ended and the text around them moved).
    fn select_live_words(&mut self, live: &LiveVoice, cx: &mut Context<Self>) -> bool {
        let text = self.body.read(cx).value();
        let Some(range) = crate::voice_state::find_live(&text, live.start, &live.shown) else {
            return false;
        };
        self.body
            .update(cx, |state, cx| state.set_selected_range(range, cx));
        true
    }

    pub fn set_view_mode(&mut self, mode: ViewMode, cx: &mut Context<Self>) {
        self.view_mode = mode;
        if mode != ViewMode::Edit {
            self.refresh_preview(cx);
        }
        self.footer.update(cx, |footer, cx| {
            footer.set_show_cursor(mode != ViewMode::Preview, cx)
        });
        cx.notify();
    }

    /// The view mode picked in the header, kept for next time.
    fn choose_view_mode(&mut self, mode: ViewMode, cx: &mut Context<Self>) {
        self.set_view_mode(mode, cx);
        AppState::update_config(cx, |config| config.appearance.view_mode = mode);
    }

    /// Applies a toolbar format to the body at its selection, as one edit
    /// that undoes in one step, and puts the cursor back in the text.
    pub fn apply_format(&mut self, format: Format, window: &mut Window, cx: &mut Context<Self>) {
        if self.note.is_none() || self.text_read_only {
            return;
        }
        self.body.update(cx, |state, cx| {
            let text = state.value();
            let edit = markdown_edit::format(format, &text, state.selected_range());
            state.set_selected_range(edit.range.clone(), cx);
            state.replace(edit.text, window, cx);
            state.set_selected_range(edit.selection, cx);
            state.focus(window, cx);
        });
    }

    /// The text's right-click menu: for a misspelled word, its guesses,
    /// Learn Spelling and Ignore Spelling, then Cut, Copy, Paste and
    /// Select All, as the editor's own menu has them.
    fn text_menu(&self, cx: &App) -> NativeMenu {
        let editable = self.note.is_some() && !self.text_read_only;
        let copyable = !self.body.read(cx).selected_range().is_empty();
        let mut menu = NativeMenu::new();
        let spelling = self.spelling_items(cx);
        if !spelling.is_empty() {
            for (label, action) in spelling {
                menu = menu.menu(label, action);
            }
            menu = menu.separator();
        }
        let mut menu = menu
            .menu_with_disabled("Cut", !(editable && copyable), Box::new(Cut))
            .menu_with_disabled("Copy", !copyable, Box::new(Copy))
            .menu_with_disabled("Paste", !editable, Box::new(Paste))
            .separator()
            .menu("Select All", Box::new(SelectAll));
        // The inline assistant works on a selection, while AI is on.
        let assists = self.inline_menu(cx);
        if !assists.is_empty() {
            menu = menu.separator();
            for action in assists {
                menu = menu.menu(action.label(), Box::new(InlineAssist { action }));
            }
        }
        menu
    }

    /// The spelling items for the misspelled word the right-click landed
    /// on, if it did.
    fn spelling_items(&self, cx: &App) -> Vec<(String, Box<dyn Action>)> {
        let selected = self.body.read(cx).selected_range();
        let Some(misspelling) = self
            .spell_marks
            .at(&self.body, selected.start, cx)
            .filter(|m: &Misspelling| m.range.end >= selected.end)
        else {
            return Vec::new();
        };
        let guesses = spellcheck::speller(cx)
            .map(|speller| speller.guesses(&misspelling.word))
            .unwrap_or_default();
        let editable = self.note.is_some() && !self.text_read_only;
        spellcheck::menu_items(&misspelling, guesses, editable)
    }

    fn correct_spelling(
        &mut self,
        action: &CorrectSpelling,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.note.is_none() || self.text_read_only {
            return;
        }
        let range = action.range.clone();
        if self.body.read(cx).value().get(range.clone()) != Some(action.word.as_str()) {
            return;
        }
        self.spell_marks.forget_range(&self.body, &range, cx);
        self.body.update(cx, |state, cx| {
            state.set_selected_range(range, cx);
            state.replace(action.with.clone(), window, cx);
        });
    }

    fn learn_spelling(&mut self, action: &LearnSpelling, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(speller) = spellcheck::speller(cx) {
            speller.learn(&action.word);
        }
        self.spell_marks.forget(&self.body, &action.word, cx);
    }

    fn ignore_spelling(&mut self, action: &IgnoreSpelling, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(speller) = spellcheck::speller(cx) {
            speller.ignore(&action.word);
        }
        self.spell_marks.forget(&self.body, &action.word, cx);
    }

    /// Whether the note's text has the keyboard and nothing is being
    /// composed in it (an input method's marked text), so a smart edit may
    /// take a key.
    fn body_takes_edits(&self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if self.note.is_none() || self.view_mode == ViewMode::Preview || self.text_read_only {
            return false;
        }
        self.body.update(cx, |state, cx| {
            state.focus_handle(cx).is_focused(window)
                && state.marked_text_range(window, cx).is_none()
        })
    }

    /// Runs `smart` on the body's text and selection; the edit it returns
    /// is applied as one undo step. Whether there was one.
    fn smart_edit(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        smart: impl FnOnce(&mut Self, &str, std::ops::Range<usize>) -> Option<Edit>,
    ) -> bool {
        let (text, selection) = {
            let state = self.body.read(cx);
            (state.value(), state.selected_range())
        };
        let Some(edit) = smart(self, &text, selection) else {
            return false;
        };
        self.body
            .update(cx, |state, cx| apply_edit(state, &text, edit, window, cx));
        true
    }

    /// A key typed in the source: an opener gets its closer, a closer Kanso
    /// put in is stepped over, and a selection is wrapped.
    // no-prop-test: key glue; `pairing` has the properties and
    // `pairing_types_through_the_editor` types through it.
    fn on_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        // Escape while listening drops what was said.
        if event.keystroke.key == "escape"
            && cx
                .try_global::<crate::voice::Voice>()
                .is_some_and(|voice| voice.is_for(&cx.entity()))
        {
            crate::voice::cancel(cx);
            cx.stop_propagation();
            return;
        }
        // Escape drops the inline assistant's suggestion.
        if event.keystroke.key == "escape" && self.inline.is_some() {
            self.discard_inline(cx);
            cx.stop_propagation();
            return;
        }
        if !self.settings.auto_pair {
            return;
        }
        let Some(ch) = typed_char(&event.keystroke) else {
            return;
        };
        if !self.pairs.may_handle(ch) || !self.body_takes_edits(window, cx) {
            return;
        }
        if self.smart_edit(window, cx, |this, text, selection| {
            this.pairs.typed(text, selection, ch)
        }) {
            cx.stop_propagation();
            // The pair goes in past the text editor's own typing, which is
            // what asks for completions: a second `[` opens `[[` ones here.
            if ch == '[' {
                self.body
                    .update(cx, |state, cx| state.show_completions(window, cx));
            }
        }
    }

    /// Backspace in an empty pair Kanso put in takes both halves.
    // no-prop-test: key glue over `Pairs::backspace`.
    fn on_backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.pairs.is_empty() || !self.body_takes_edits(window, cx) {
            return;
        }
        if self.smart_edit(window, cx, |this, text, selection| {
            if !selection.is_empty() {
                return None;
            }
            this.pairs.backspace(text, selection.start)
        }) {
            cx.stop_propagation();
        }
    }

    /// Tab and Shift-Tab: the next or previous table cell, a list item in
    /// or out with what hangs under it, or else the editor's own indent.
    // no-prop-test: key glue over `kanso_engine::edit::tab`, tested there.
    fn on_tab(&mut self, out: bool, window: &mut Window, cx: &mut Context<Self>) {
        if !self.body_takes_edits(window, cx) {
            return;
        }
        let format = self.settings.format_tables;
        if self.smart_edit(window, cx, |_, text, selection| {
            kanso_engine::edit::tab(text, selection, out, format)
        }) {
            cx.stop_propagation();
        }
    }

    /// Enter at the end of a table row adds a row; in an empty last row it
    /// leaves the table.
    // no-prop-test: key glue over `kanso_engine::edit::enter`, tested there.
    fn on_enter(&mut self, action: &Enter, window: &mut Window, cx: &mut Context<Self>) {
        if action.secondary || action.shift || !self.body_takes_edits(window, cx) {
            return;
        }
        // An open completion list takes Enter as "choose", and nothing else
        // may see it: left to bubble, the keystroke also types a newline.
        let chose = self.body.update(cx, |state, cx| {
            state.completion_menu_state().open
                && state.route_overlay_action(Box::new(action.clone()), window, cx)
        });
        if chose {
            cx.stop_propagation();
            return;
        }
        let format = self.settings.format_tables;
        if self.smart_edit(window, cx, |_, text, selection| {
            kanso_engine::edit::enter(text, selection, format)
        }) {
            cx.stop_propagation();
        }
    }

    /// Lines up the table at the cursor ("Format Table").
    pub fn format_table(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.note.is_none() || self.view_mode == ViewMode::Preview {
            return;
        }
        let formatted = self.smart_edit(window, cx, |_, text, selection| {
            table_format::format_table(text, selection.start)
        });
        if formatted {
            self.focus_body(window, cx);
        } else {
            cx.emit(EditorEvent::Notice(
                "Put the cursor in a table to format it.".into(),
            ));
        }
    }

    /// ⌘V in the source: images become attachments; with smart paste on, a
    /// web address over selected words links them and formatted text comes
    /// in as Markdown. Anything else pastes as usual.
    // no-prop-test: clipboard glue; `smart_paste` has the properties and
    // `pasting_links_and_images` pastes through it.
    fn on_paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        if !self.body_takes_edits(window, cx) {
            return;
        }
        let Some(item) = cx.read_from_clipboard() else {
            return;
        };
        let mut plain = None;
        let mut images = Vec::new();
        for entry in item.entries() {
            match entry {
                ClipboardEntry::String(string) => {
                    plain.get_or_insert_with(|| string.text().clone());
                }
                ClipboardEntry::Image(image) => {
                    images.push(Pasted::Bytes(image.bytes.clone(), image.format.extension()))
                }
                ClipboardEntry::ExternalPaths(paths) => images.extend(
                    paths
                        .paths()
                        .iter()
                        .filter(|path| image_path(path))
                        .map(|path| Pasted::File(path.clone())),
                ),
            }
        }
        // Copied image files, or a copied image with no text.
        let files = images.iter().any(|p| matches!(p, Pasted::File(_)));
        if !images.is_empty() && (files || plain.is_none()) {
            cx.stop_propagation();
            self.attach_images(images, None, window, cx);
            return;
        }
        if !self.settings.smart_paste {
            return;
        }
        let Some(plain) = plain else {
            return;
        };
        if self.smart_edit(window, cx, |_, text, selection| {
            if let Some(link) = smart_paste::link_over_selection(text, selection.clone(), &plain) {
                return Some(link);
            }
            let markdown = clipboard_html()
                .filter(|html| smart_paste::worth_converting(html))
                .and_then(|html| smart_paste::html_to_markdown(&html))?;
            let end = selection.start + markdown.len();
            Some(Edit {
                range: selection,
                text: markdown,
                selection: end..end,
            })
        }) {
            cx.stop_propagation();
        }
    }

    /// ⇧⌘V: the clipboard's plain text, as it is, over the selection.
    pub fn paste_plain(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.body_takes_edits(window, cx) {
            // The title pastes plain text anyway.
            window.dispatch_action(Box::new(Paste), cx);
            return;
        }
        let Some(plain) = cx.read_from_clipboard().and_then(|item| item.text()) else {
            return;
        };
        self.smart_edit(window, cx, |_, _, selection| {
            let end = selection.start + plain.len();
            Some(Edit {
                range: selection,
                text: plain,
                selection: end..end,
            })
        });
    }

    /// Files dropped on the source: images become attachments where they
    /// fall.
    // no-prop-test: drop glue over `attach_images`.
    fn on_drop(&mut self, paths: &ExternalPaths, window: &mut Window, cx: &mut Context<Self>) {
        if self.note.is_none() || self.view_mode == ViewMode::Preview {
            return;
        }
        let images: Vec<Pasted> = paths
            .paths()
            .iter()
            .filter(|path| image_path(path) || is_drawing(path))
            .map(|path| Pasted::File(path.clone()))
            .collect();
        let audio: Vec<std::path::PathBuf> = paths
            .paths()
            .iter()
            .filter(|path| crate::transcribe::is_audio(path))
            .cloned()
            .collect();
        if images.is_empty() && audio.is_empty() {
            if !paths.paths().is_empty() {
                cx.emit(EditorEvent::Notice(
                    "Only images, recordings and Excalidraw drawings can be dropped into a note."
                        .into(),
                ));
            }
            return;
        }
        let point = window.mouse_position();
        let at = self.body.update(cx, |state, cx| {
            let utf16 = state.character_index_for_point(point, window, cx)?;
            Some(utf8_offset(&state.value(), utf16))
        });
        self.focus_body(window, cx);
        if !audio.is_empty() {
            self.attach_recordings(audio, at, window, cx);
        }
        if !images.is_empty() {
            self.attach_images(images, at, window, cx);
        }
    }

    /// Attaches recordings dropped at `at` (those up to the attachment
    /// limit), puts their links there, and transcribes the first: its words
    /// go under its link, or at the end of the note for one too long to
    /// attach.
    // no-prop-test: glue; `transcribe::transcript_edit` and `attach_audio`
    // have the tests.
    fn attach_recordings(
        &mut self,
        paths: Vec<std::path::PathBuf>,
        at: Option<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let store = self.store.clone();
        let length = self.body.read(cx).text().len();
        let work = cx.background_executor().spawn(async move {
            paths
                .into_iter()
                .map(|path| {
                    let attached = crate::transcribe::attach_audio(&store, &path);
                    (path, attached)
                })
                .collect::<Vec<_>>()
        });
        cx.spawn_in(window, async move |this, cx| {
            let results = work.await;
            this.update_in(cx, |this, window, cx| {
                if this.note.is_none() {
                    return;
                }
                let mut links = Vec::new();
                let mut first = None;
                for (path, attached) in results {
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("recording")
                        .to_owned();
                    match attached {
                        Ok(Some((file_id, link))) => {
                            links.push(link);
                            first.get_or_insert(crate::transcribe::Source::Path {
                                path,
                                file_id: Some(file_id),
                                name,
                            });
                        }
                        Ok(None) => {
                            first.get_or_insert(crate::transcribe::Source::Path {
                                path,
                                file_id: None,
                                name,
                            });
                        }
                        Err(message) => cx.emit(EditorEvent::Notice(message)),
                    }
                }
                if !links.is_empty() {
                    let block = links.join("\n\n");
                    this.smart_edit(window, cx, |_, text, selection| {
                        let range = match at {
                            Some(at) if text.len() == length && text.is_char_boundary(at) => at..at,
                            _ => selection,
                        };
                        Some(smart_paste::block_insert(text, range, &block))
                    });
                    this.save(cx);
                }
                if let Some(source) = first {
                    let editor = cx.entity();
                    crate::transcribe::start(editor, source, window, cx);
                }
            })
            .ok();
        })
        .detach();
    }

    /// Puts a transcript in the note as a paragraph of its own, under the
    /// line linking to `file_id` (or at the end), as one undo step, leaving
    /// the caret where it was. False when the note's text can't take it.
    // no-prop-test: glue over `transcribe::transcript_edit`, which has the properties.
    pub fn insert_transcript_below(
        &mut self,
        file_id: Option<&str>,
        transcript: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.note.is_none() || matches!(self.lock, LockView::Closed(_)) {
            return false;
        }
        self.smart_edit(window, cx, |_, text, selection| {
            crate::transcribe::transcript_edit(text, file_id, transcript, selection)
        });
        self.schedule_autosave(cx);
        true
    }

    /// The audio attachment linked on the caret's line: its file ID and
    /// name.
    pub fn audio_on_caret_line(&self, cx: &App) -> Option<(String, String)> {
        self.note.as_ref()?;
        let state = self.body.read(cx);
        let text = state.value();
        let caret = state.selected_range().start.min(text.len());
        let start = text[..caret].rfind('\n').map_or(0, |i| i + 1);
        let end = text[caret..].find('\n').map_or(text.len(), |i| caret + i);
        crate::transcribe::linked_files(&text[start..end])
            .into_iter()
            .find_map(|id| {
                let doc = self.store.get(&id).ok()??;
                let content_type = doc.str_field("contentType")?;
                kanso_voice::file::extension(content_type)?;
                let name = doc.str_field("name").unwrap_or("recording").to_owned();
                Some((id, name))
            })
    }

    /// Stores `images` as attachments in the background, then puts them in
    /// the text at `at` (or over the selection), each a paragraph of its
    /// own. What can't be attached is said in a notice.
    fn attach_images(
        &mut self,
        images: Vec<Pasted>,
        at: Option<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let store = self.store.clone();
        let length = self.body.read(cx).text().len();
        let work = cx.background_executor().spawn(async move {
            images
                .into_iter()
                .map(|image| attach(&store, image))
                .collect::<Vec<_>>()
        });
        cx.spawn_in(window, async move |this, cx| {
            let results = work.await;
            this.update_in(cx, |this, window, cx| {
                let mut blocks = Vec::new();
                for result in results {
                    match result {
                        Ok(block) => blocks.push(block),
                        Err(message) => cx.emit(EditorEvent::Notice(message)),
                    }
                }
                if blocks.is_empty() || this.note.is_none() {
                    return;
                }
                let block = blocks.join("\n\n");
                this.smart_edit(window, cx, |_, text, selection| {
                    // Where it was dropped, unless the text changed since.
                    let range = match at {
                        Some(at) if text.len() == length && text.is_char_boundary(at) => at..at,
                        _ => selection,
                    };
                    Some(smart_paste::block_insert(text, range, &block))
                });
            })
            .ok();
        })
        .detach();
    }

    /// A size picked for an image in the preview: its Markdown is
    /// rewritten, as one undo step, with the cursor kept where it was.
    // no-prop-test: glue; `smart_paste::sized_image` has the properties and
    // `image_sizes_rewrite_the_markdown` resizes through it.
    fn resize_image(&mut self, resize: ImageResize, window: &mut Window, cx: &mut Context<Self>) {
        if self.note.is_none() {
            return;
        }
        let replacement = match crate::image_size::line_image(&resize.source) {
            Some(image) => crate::image_size::written(&image, resize.width),
            None => smart_paste::sized_image(&resize.src, &resize.alt, resize.width),
        };
        self.replace_block(&resize.source, replacement, window, cx);
    }

    /// A task box ticked in the preview: its `[ ]` becomes `[x]` (or back) in
    /// the note's text, as one undo step with the cursor kept where it was,
    /// saved like typing. Nothing is written where the editor takes no
    /// typing, or when the note no longer has that task where the click
    /// found it.
    // no-prop-test: glue; `preview_tasks` has the properties and
    // `ticking_a_task_in_the_preview_rewrites_the_markdown` goes through it.
    fn tick_task(&mut self, tick: &TaskTick, window: &mut Window, cx: &mut Context<Self>) {
        if !self.takes_preview_edits(cx) {
            return;
        }
        self.smart_edit(window, cx, |_, text, selection| {
            body_edit(text, &preview_tasks::toggled(text, tick)?, selection)
        });
    }

    /// Whether the preview may rewrite the note's text: it is open for
    /// editing, not in the trash, not locked shut, not shared to view only,
    /// not held by an account lock or an unanswered change made elsewhere,
    /// and the preview shows the text as it is now, so a box clicked is the
    /// task it was drawn for.
    fn takes_preview_edits(&self, cx: &App) -> bool {
        let editable = self
            .note
            .as_ref()
            .is_some_and(|note| note.book_id != TRASH_BOOK_ID);
        editable
            && !self.lock.is_closed()
            && !self.text_read_only
            && !self.collab_read_only()
            && self.incoming.is_none()
            && !self.preview_stale
            && !writes_held(cx)
    }

    /// The image on the cursor's line in the note's text, if that line is
    /// only one: for "Resize image…".
    pub fn image_on_caret_line(&self, cx: &App) -> Option<crate::image_size::LineImage> {
        if self.note.is_none() || self.view_mode == ViewMode::Preview {
            return None;
        }
        let state = self.body.read(cx);
        let text = state.value();
        crate::image_size::image_at(&text, state.selected_range().start).map(|(_, image)| image)
    }

    /// Gives the image on the cursor's line `width` (`None`: its own size),
    /// as one undo step.
    // no-prop-test: glue; `image_size::resize_at` has the tests and
    // `resize_image_from_the_palette` goes through it.
    pub fn resize_caret_image(
        &mut self,
        width: Option<u32>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.note.is_none() {
            return;
        }
        let resized = self.smart_edit(window, cx, |_, text, selection| {
            crate::image_size::resize_at(text, selection.start, width)
        });
        if resized {
            self.focus_body(window, cx);
        } else {
            cx.emit(EditorEvent::Notice(
                "Put the cursor on an image's line to resize it.".into(),
            ));
        }
    }

    /// "New drawing": an empty Excalidraw drawing at the cursor, on lines of
    /// its own, as one undo step; then an offer to draw in it.
    // no-prop-test: glue; `image_size::insert_block` has the properties and
    // `new_drawing_goes_in_at_the_cursor` goes through it.
    pub fn insert_drawing(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self
            .note
            .as_ref()
            .is_none_or(|n| n.book_id == TRASH_BOOK_ID)
        {
            return;
        }
        if self.view_mode == ViewMode::Preview {
            self.set_view_mode(ViewMode::Split, cx);
        }
        self.smart_edit(window, cx, |_, text, selection| {
            Some(crate::image_size::insert_block(
                text,
                selection.start,
                crate::image_size::EMPTY_DRAWING,
            ))
        });
        self.focus_body(window, cx);
        let scene = crate::image_size::EMPTY_DRAWING
            .trim_start_matches("```excalidraw\n")
            .trim_end_matches("\n```")
            .to_owned();
        window.push_notification(
            Notification::new()
                .title("New drawing")
                .message("Draw in Excalidraw, then Paste back from the drawing in the preview.")
                .icon(Icon::new(Lucide::PenTool))
                .action(move |_, _, _| {
                    let scene = scene.clone();
                    Button::new("edit-drawing")
                        .small()
                        .primary()
                        .label("Edit in Excalidraw")
                        .on_click(move |_, window, cx| {
                            crate::preview_rich::edit_in_excalidraw(&scene, window, cx)
                        })
                }),
            cx,
        );
    }

    /// Puts `replacement` over the first `source` in the note, as one undo
    /// step, with the cursor kept where it was (or after the replacement,
    /// if it was inside it).
    // no-prop-test: glue; `image_sizes_rewrite_the_markdown` and
    // `drawings_brought_back_replace_their_block` go through it.
    pub(crate) fn replace_block(
        &mut self,
        source: &str,
        replacement: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.note.is_none() || source.is_empty() {
            return;
        }
        self.smart_edit(window, cx, |_, text, selection| {
            let start = text.find(source)?;
            let end = start + source.len();
            let shift = |at: usize| {
                if at >= end {
                    at - source.len() + replacement.len()
                } else {
                    at.min(start)
                }
            };
            let selection = if selection.end <= start || selection.start >= end {
                shift(selection.start)..shift(selection.end)
            } else {
                let after = start + replacement.len();
                after..after
            };
            Some(Edit {
                range: start..end,
                text: replacement.clone(),
                selection,
            })
        });
    }

    /// Replaces the open note's whole text as one undoable edit, keeping the
    /// caret where it was when it isn't in the part replaced.
    pub fn set_body(&mut self, body: String, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if self.note.is_none() || self.text_read_only || self.collab_read_only() {
            return false;
        }
        self.smart_edit(window, cx, |_, text, selection| {
            body_edit(text, &body, selection)
        })
    }

    /// A change to the notes has settled: query views in the preview read
    /// them again.
    pub fn queries_changed(&mut self, cx: &mut Context<Self>) {
        crate::query_view_ui::notes_changed(&self.store);
        self.preview
            .update(cx, |preview, cx| preview.queries_changed(cx));
    }

    /// Finds the notes that link to the open one, in the background.
    pub fn refresh_backlinks(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.note_id().map(str::to_owned) else {
            self.backlinks.clear();
            self.backlinks_task = None;
            return;
        };
        let store = self.store.clone();
        let search = cx
            .background_executor()
            .spawn(async move { backlinks(&store, &id) });
        self.backlinks_task = Some(cx.spawn(async move |this, cx| {
            let found = search.await;
            this.update(cx, |this, cx| {
                if this.backlinks != found {
                    this.backlinks = found;
                    cx.notify();
                }
            })
            .ok();
        }));
    }

    /// Replaces the editor's text with an earlier version, as a normal edit.
    /// The text it replaces is kept in history first.
    pub fn restore(
        &mut self,
        title: String,
        body: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.collab_read_only() {
            self.notice(
                "The note was shared with you to read: its text can't be changed here.",
                cx,
            );
            return;
        }
        self.flush(cx);
        self.stash_draft(cx);
        self.title
            .update(cx, |state, cx| state.set_value(title, window, cx));
        self.body
            .update(cx, |state, cx| state.set_value(body.clone(), window, cx));
        self.source_lines = None;
        let text = self.preview_source(body.into());
        self.preview
            .update(cx, |preview, cx| preview.set_text(text, cx));
        self.save(cx);
    }

    /// Whether the open note's text is saved, as the header says it.
    pub(crate) fn save_state(&self, cx: &App) -> SaveState {
        let pending = self.autosave.is_some() || self.collab_pending();
        match save_state(self.incoming.is_some(), pending, || {
            self.has_unsaved_edits(cx)
        }) {
            // Another editor of the note has the text, and saves it.
            SaveState::NotSaved if self.from_peer && self.incoming.is_none() => SaveState::Saving,
            state => state,
        }
    }

    /// The top band: where the note lives (click it to move the note),
    /// whether it is saved, linked-from notes, the view mode and the note's
    /// actions. Its empty middle moves the window.
    /// While this note listens: a level meter, the time, and a cancel; then
    /// "Writing…" while what was said becomes text.
    // no-prop-test: drawing; the voice snapshots show each state.
    fn voice_pill(&self, colors: &Colors, cx: &mut Context<Self>) -> Option<AnyElement> {
        let voice = cx.try_global::<crate::voice::Voice>()?;
        if !voice.is_for(&cx.entity()) {
            return None;
        }
        let pill = h_flex()
            .id("voice-pill")
            .debug_selector(|| "voice-pill".into())
            .flex_none()
            .h(px(24.))
            .mr(px(4.))
            .pl(px(10.))
            .gap(px(8.))
            .rounded_full()
            .bg(colors.fill)
            .text_size(px(12.))
            .text_color(colors.text_secondary);
        Some(match voice.level_and_elapsed() {
            Some((level, elapsed)) => pill
                .child(level_meter(level, colors))
                .child(
                    div()
                        .min_w(px(30.))
                        .child(crate::voice::elapsed_label(elapsed)),
                )
                .child(
                    Button::new("voice-cancel")
                        .ghost()
                        .xsmall()
                        .icon(Icon::new(Lucide::Close).text_color(colors.text_secondary))
                        .tooltip("Cancel (Esc)")
                        .on_click(|_, _, cx| crate::voice::cancel(cx)),
                )
                .into_any_element(),
            None => pill
                .pr(px(10.))
                .child(
                    Icon::new(Lucide::LoaderCircle)
                        .xsmall()
                        .text_color(colors.text_secondary),
                )
                .child("Writing…")
                .into_any_element(),
        })
    }

    fn render_band(&self, note: &Note, inset: f32, cx: &mut Context<Self>) -> impl IntoElement {
        let library = self.library.read(cx);
        let path: Vec<String> = if note.book_id == TRASH_BOOK_ID {
            vec!["Trash".to_owned()]
        } else {
            let names: Vec<String> = library
                .book_path(&note.book_id)
                .iter()
                .map(|b| b.name.clone())
                .collect();
            if names.is_empty() {
                vec!["No notebook".to_owned()]
            } else {
                names
            }
        };
        let title = self.title.read(cx).value();
        let title: SharedString = if title.trim().is_empty() {
            "Untitled".into()
        } else {
            title
        };
        let colors = Colors::get(cx);
        let (saved, saved_color) = match self.save_state(cx) {
            SaveState::Saving => ("Saving…".to_owned(), colors.text_muted),
            SaveState::NotSaved => ("Not saved".to_owned(), colors.warning),
            SaveState::Saved => (
                format!("Edited {}", Clock::edited_label(cx, note.updated_at)),
                colors.text_muted,
            ),
        };
        let view = cx.entity();
        // One separator for the notebook path and before the title, the
        // same size as the words and the same room on either side.
        let separator = || {
            div()
                .flex_none()
                .text_size(px(13.))
                .text_color(colors.text_muted)
                .child("/")
        };
        let mode = self.view_mode;
        let pinned = note.pinned;
        let locked = self.lock != LockView::Plain;
        let status = note.status;

        let segments = path.into_iter().enumerate().flat_map(|(ix, name)| {
            let name = div()
                .min_w_0()
                .truncate()
                .text_size(px(13.))
                .text_color(colors.text_secondary)
                .child(name)
                .into_any_element();
            let before = (ix > 0).then(|| separator().mx(px(8.)).into_any_element());
            before.into_iter().chain([name])
        });
        // Out of the Tab order while focus mode hides them.
        let tab_stop = !self.chrome_hidden();
        let listening = cx.try_global::<crate::voice::Voice>().is_some_and(|voice| {
            voice.is_for(&cx.entity())
                && matches!(voice.phase, crate::voice_state::Phase::Listening { .. })
        });
        let move_menu = Button::new("breadcrumb")
            .ghost()
            .small()
            .tab_stop(tab_stop)
            // Buttons don't shrink by default; a long path truncates.
            .flex_shrink(1.)
            .min_w_0()
            .child(
                h_flex()
                    .min_w_0()
                    .max_w(px(240.))
                    .overflow_hidden()
                    .children(segments),
            )
            .tooltip("Move to notebook")
            .debug_selector(|| "breadcrumb".into())
            // The palette's list rather than a menu: a library can have more
            // notebooks than a menu shows, and typing finds one.
            .on_click(cx.listener(|_, _, _, cx| cx.emit(EditorEvent::Pick(Picker::Notebook))));

        let backlinks = (!self.backlinks.is_empty()).then(|| {
            let count = self.backlinks.len();
            Button::new("backlinks")
                .ghost()
                .small()
                .tab_stop(tab_stop)
                .icon(Icon::new(Lucide::Link).text_color(colors.text_secondary))
                .child(
                    div()
                        .text_size(px(12.5))
                        .text_color(colors.text_secondary)
                        .child(count.to_string()),
                )
                .tooltip(if count == 1 {
                    "Linked from 1 note".to_owned()
                } else {
                    format!("Linked from {count} notes")
                })
                .on_click(cx.listener(|_, _, _, cx| cx.emit(EditorEvent::ShowBacklinks)))
        });
        let inspector_open = self.inspector_open;

        let modes = [
            ("mode-edit", "Edit", ViewMode::Edit),
            ("mode-preview", "Preview", ViewMode::Preview),
            ("mode-split", "Split", ViewMode::Split),
        ];
        let mode_group = ButtonGroup::new("view-mode")
            .outline()
            .small()
            .children(modes.map(|(id, label, option)| {
                let selected = mode == option;
                Button::new(id).selected(selected).tab_stop(tab_stop).child(
                    div()
                        .px_0p5()
                        .text_size(px(12.5))
                        // The mode in use reads heavier as well as brighter:
                        // the segment's fill alone is faint on dark themes.
                        .when(selected, |el| el.font_medium())
                        .text_color(if selected {
                            colors.text
                        } else {
                            colors.text_secondary
                        })
                        .child(label),
                )
            }))
            .on_click(cx.listener(move |this, clicked: &Vec<usize>, _, cx| {
                if let Some(&ix) = clicked.first() {
                    this.choose_view_mode(modes[ix].2, cx);
                }
            }));

        h_flex()
            .flex_none()
            .h(px(TOP_BAND_HEIGHT))
            // The ghost button's padding lines its text up with the title.
            .pl(px(inset + TEXT_INSET - 8.))
            .pr(px(12.))
            .gap(px(4.))
            // When the pane is narrow the save state gives way first, then
            // the path and the title down to a few letters. The button's
            // own padding is the room before the separator.
            .child(
                h_flex()
                    .min_w_0()
                    .child(move_menu)
                    .child(separator().mr(px(8.)))
                    .child(
                        div()
                            .min_w(px(64.))
                            .truncate()
                            .text_size(px(13.))
                            .child(title),
                    )
                    .when(kanso_core::roles::daily_date(&note.id).is_some(), |el| {
                        el.child(day_arrows(tab_stop, cx))
                    }),
            )
            // The save state shows whole or not at all: when it doesn't fit
            // beside the title it wraps onto a second line the band clips.
            // Shrinking it instead left a stub such as "Edi…" while the
            // title was cut short.
            .child(
                h_flex()
                    .flex_1()
                    .min_w(px(12.))
                    .h(px(SAVE_LABEL_HEIGHT))
                    .flex_wrap()
                    .overflow_hidden()
                    .child(div().flex_1().h(px(SAVE_LABEL_HEIGHT)))
                    .child(
                        div()
                            .flex_none()
                            .mr(px(8.))
                            .whitespace_nowrap()
                            .line_height(px(SAVE_LABEL_HEIGHT))
                            .text_size(px(12.))
                            .text_color(saved_color)
                            .child(saved),
                    ),
            )
            .children(backlinks)
            .child(div().flex_none().mx_1().child(mode_group))
            // An open locked note shows its lock open, and a click locks
            // every note again. Not a listener: `lock_now` updates this
            // editor.
            .when(self.lock == LockView::Open, |el| {
                let editor = cx.entity();
                el.child(
                    Button::new("lock-now")
                        .debug_selector(|| "lock-now".into())
                        .ghost()
                        .small()
                        .tab_stop(tab_stop)
                        .icon(Icon::new(Lucide::LockOpen).text_color(colors.text_secondary))
                        .tooltip("Unlocked: click to lock notes now")
                        .on_click(move |_, window, cx| {
                            crate::locked::lock_now(&editor, window, cx)
                        }),
                )
            })
            .child(
                Button::new("pin")
                    .ghost()
                    .small()
                    .tab_stop(tab_stop)
                    .icon(Icon::new(Lucide::Pin).text_color(if pinned {
                        colors.text
                    } else {
                        colors.text_secondary
                    }))
                    .selected(pinned)
                    .tooltip(if pinned { "Unpin note" } else { "Pin note" })
                    .on_click(cx.listener(|this, _, _, cx| this.toggle_pinned(cx))),
            )
            .children(self.voice_pill(colors, cx))
            .child(
                Button::new("dictate")
                    .debug_selector(|| "dictate".into())
                    .ghost()
                    .small()
                    .tab_stop(tab_stop)
                    .icon(Icon::new(Lucide::Mic).text_color(if listening {
                        colors.danger
                    } else {
                        colors.text_secondary
                    }))
                    .selected(listening)
                    .tooltip(if listening {
                        "Stop and write it down"
                    } else {
                        "Voice input"
                    })
                    // Not a listener: `press` updates this editor (System
                    // Dictation puts the keyboard in the note), which can't be
                    // done while a listener of its own is updating it.
                    .on_click({
                        let editor = cx.entity();
                        move |_, window, cx| crate::voice::press(editor.clone(), window, cx)
                    }),
            )
            .child(
                Button::new("history")
                    .ghost()
                    .small()
                    .tab_stop(tab_stop)
                    .icon(Icon::new(Lucide::Clock).text_color(colors.text_secondary))
                    .tooltip("Show history")
                    .on_click(cx.listener(|_, _, _, cx| cx.emit(EditorEvent::ShowHistory))),
            )
            .child(
                Button::new("note-menu")
                    .ghost()
                    .small()
                    .tab_stop(tab_stop)
                    .icon(Icon::new(IconName::Ellipsis).text_color(colors.text_secondary))
                    .tooltip("More")
                    .dropdown_menu(move |menu, window, cx| {
                        let view = view.clone();
                        let run: crate::note_menu::Run = Rc::new(move |command, _, cx| {
                            view.update(cx, |_, cx| cx.emit(EditorEvent::Command(command)));
                        });
                        crate::note_menu::items(menu, pinned, locked, status, run, window, cx)
                    }),
            )
            .when(!self.standalone, |el| {
                el.child(
                    Button::new("inspector")
                        .ghost()
                        .small()
                        .tab_stop(tab_stop)
                        .icon(Icon::new(Lucide::PanelRight).text_color(if inspector_open {
                            colors.text
                        } else {
                            colors.text_secondary
                        }))
                        .selected(inspector_open)
                        .tooltip(if inspector_open {
                            "Hide outline and backlinks"
                        } else {
                            "Show outline and backlinks"
                        })
                        .debug_selector(|| "inspector-toggle".into())
                        .on_click(cx.listener(|_, _, _, cx| cx.emit(EditorEvent::ToggleInspector))),
                )
            })
    }

    /// The note's title: the field while it has the keyboard, and otherwise
    /// the title cut short with an ellipsis where the pane ends, as the list
    /// and the header cut it. A click puts the keyboard in the field.
    // no-prop-test: layout; `the_title_is_cut_short_until_it_is_edited`
    // clicks it and the focus snapshots show it.
    fn render_title(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let style = |el: Div| {
            el.w_full()
                .h(px(40.))
                .text_size(px(28.))
                .line_height(px(36.))
                .font_semibold()
        };
        if self.title.read(cx).focus_handle(cx).is_focused(window) {
            return Input::new(&self.title)
                .appearance(false)
                .text_size(px(28.))
                .line_height(px(36.))
                .h(px(40.))
                .p_0()
                .font_semibold()
                .into_any_element();
        }
        let colors = Colors::get(cx);
        let title = self.title.read(cx).value().clone();
        let (text, color) = if title.trim().is_empty() {
            (SharedString::from("Untitled"), colors.text_muted)
        } else {
            (title, colors.text)
        };
        style(div())
            .id("note-title")
            .debug_selector(|| "note-title".into())
            .min_w_0()
            .overflow_hidden()
            .whitespace_nowrap()
            .text_ellipsis()
            .text_color(color)
            .cursor_text()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    this.focus_title(window, cx);
                    cx.notify();
                }),
            )
            .child(text)
            .into_any_element()
    }

    /// The status chip, the note's tags and "Add tag".
    fn render_meta(&self, note: &Note, cx: &mut Context<Self>) -> impl IntoElement {
        let library = self.library.read(cx);
        let tags: Vec<(String, String, u32)> = library
            .tags
            .iter()
            .map(|t| (t.id.clone(), t.name.clone(), t.color.rgb()))
            .collect();
        let status = note.status;
        let view = cx.entity();
        let colors = Colors::get(cx);
        let dark = cx.theme().is_dark();

        // Out of the Tab order while focus mode hides them.
        let tab_stop = !self.chrome_hidden();
        let status_chip =
            status_glyph(status, colors, colors.text_secondary).map(|(icon, tint)| {
                let view = view.clone();
                Button::new("status")
                    .ghost()
                    .xsmall()
                    .tab_stop(tab_stop)
                    .icon(Icon::new(icon).text_color(tint))
                    .child(
                        div()
                            .text_size(px(12.5))
                            .text_color(colors.text_secondary)
                            .child(status.label()),
                    )
                    .bg(colors.fill)
                    .tooltip("Status")
                    .dropdown_menu(move |mut menu, _, _| {
                        for option in NoteStatus::ALL {
                            let view = view.clone();
                            menu = menu.item(
                                PopupMenuItem::new(option.label())
                                    .checked(option == status)
                                    .on_click(move |_, _, cx| {
                                        view.update(cx, |this, cx| this.set_status(option, cx));
                                    }),
                            );
                        }
                        menu
                    })
            });

        let chips = note
            .tags
            .iter()
            .filter_map(|id| {
                let (_, name, rgb) = tags.iter().find(|(tid, _, _)| tid == id)?;
                let library = self.library.clone();
                let view = view.clone();
                let filter = Filter::Tag(id.clone());
                let tag_id = id.clone();
                Some(
                    div()
                        .id(SharedString::from(format!("note-tag-{id}")))
                        .cursor_pointer()
                        .child(
                            Tag::custom(
                                colors.fill.into(),
                                colors.text_secondary.into(),
                                colors.fill.into(),
                            )
                            .text_size(px(12.5))
                            .px(px(8.))
                            .py(px(2.))
                            .child(
                                div()
                                    .text_color(rgb_color(muted_tag_color(*rgb, dark)))
                                    .child("#"),
                            )
                            .child(name.clone()),
                        )
                        .on_click(move |_, _, cx| {
                            library
                                .update(cx, |library, cx| library.set_filter(filter.clone(), cx));
                        })
                        .context_menu(move |menu, _, _| {
                            let view = view.clone();
                            let tag_id = tag_id.clone();
                            menu.item(PopupMenuItem::new("Remove Tag").on_click(move |_, _, cx| {
                                view.update(cx, |this, cx| this.toggle_tag(tag_id.clone(), cx));
                            }))
                        }),
                )
            })
            .collect::<Vec<_>>();

        let add_tag = Button::new("tags")
            .ghost()
            .xsmall()
            .tab_stop(tab_stop)
            .icon(Icon::new(IconName::Plus).text_color(colors.text_muted))
            .child(
                div()
                    .text_size(px(12.5))
                    .text_color(colors.text_muted)
                    .child("Add tag"),
            )
            .debug_selector(|| "add-tag".into())
            // The palette's list: it takes a tag off or puts one on, finds
            // one as you type, and makes a new one from what was typed, even
            // in a library with no tags yet.
            .on_click(cx.listener(|_, _, _, cx| cx.emit(EditorEvent::Pick(Picker::Tag))));

        h_flex()
            .flex_none()
            .flex_wrap()
            .gap(px(6.))
            .px(px(TEXT_INSET))
            .pb(px(6.))
            .children(status_chip)
            .children(chips)
            .child(add_tag)
    }

    /// The formatting toolbar: headings, lists, code, links, images, tables
    /// and quotes, each a toggle or a template at the cursor.
    fn render_format_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        // Out of the Tab order while focus mode hides them.
        let tab_stop = !self.chrome_hidden();
        let button = |id: &'static str, icon: Icon, tooltip: &'static str, format: Format| {
            let button = Button::new(id)
                .ghost()
                .small()
                .tab_stop(tab_stop)
                .icon(icon.text_color(colors.text_secondary));
            // The tooltip names the shortcut when the format has one.
            let button = match format_keys::format_action(format) {
                Some(action) => button.tooltip_with_action(
                    tooltip,
                    action.as_ref(),
                    Some(format_keys::FORMAT_CONTEXT),
                ),
                None => button.tooltip(tooltip),
            };
            button.on_click(
                cx.listener(move |this, _, window, cx| this.apply_format(format, window, cx)),
            )
        };
        let separator = || {
            div()
                .flex_none()
                .w(px(1.))
                .h(px(16.))
                .mx(px(6.))
                .bg(colors.divider)
        };
        let view = cx.entity();
        // The menu looks up its shortcuts from the text, where they apply.
        let body_focus = self.body.read(cx).focus_handle(cx);
        h_flex()
            .id("format-bar")
            .flex_none()
            .h(px(34.))
            .pl(px(TEXT_INSET - 8.))
            .gap(px(2.))
            .child(button(
                "h1",
                Lucide::Heading1.into(),
                "Heading 1",
                Format::Heading(1),
            ))
            .child(button(
                "h2",
                Lucide::Heading2.into(),
                "Heading 2",
                Format::Heading(2),
            ))
            .child(button(
                "h3",
                Lucide::Heading3.into(),
                "Heading 3",
                Format::Heading(3),
            ))
            .child(separator())
            .child(button(
                "bulleted",
                Lucide::List.into(),
                "Bulleted list",
                Format::Bulleted,
            ))
            .child(button(
                "numbered",
                Lucide::ListOrdered.into(),
                "Numbered list",
                Format::Numbered,
            ))
            .child(button(
                "task",
                Lucide::ListTodo.into(),
                "Task",
                Format::Task,
            ))
            .child(button("code", Lucide::Code.into(), "Code", Format::Code))
            .child(button("link", Lucide::Link.into(), "Link", Format::Link))
            .child(button(
                "image",
                Lucide::Image.into(),
                "Image",
                Format::Image,
            ))
            .child(button(
                "table",
                Lucide::Table.into(),
                "Table",
                Format::Table,
            ))
            .child(button(
                "quote",
                Lucide::TextQuote.into(),
                "Quote",
                Format::Quote,
            ))
            .child(separator())
            .child(
                Button::new("format-more")
                    .ghost()
                    .small()
                    .tab_stop(tab_stop)
                    .icon(Icon::new(IconName::Ellipsis).text_color(colors.text_secondary))
                    .tooltip("More formatting")
                    .dropdown_menu(move |menu, _, _| {
                        let mut menu = menu.action_context(body_focus.clone());
                        for (label, format) in [
                            ("Bold", Format::Bold),
                            ("Italic", Format::Italic),
                            ("Strikethrough", Format::Strikethrough),
                            ("Code Block", Format::CodeBlock),
                            ("Heading 4", Format::Heading(4)),
                            ("Heading 5", Format::Heading(5)),
                            ("Heading 6", Format::Heading(6)),
                            ("Horizontal Rule", Format::Rule),
                        ] {
                            let view = view.clone();
                            let mut item =
                                PopupMenuItem::new(label).on_click(move |_, window, cx| {
                                    view.update(cx, |this, cx| {
                                        this.apply_format(format, window, cx)
                                    });
                                });
                            if let Some(action) = format_keys::format_action(format) {
                                item = item.action(action);
                            }
                            menu = menu.item(item);
                        }
                        menu
                    }),
            )
    }

    /// A quiet strip above the text with a message and its choices.
    fn banner(&self, tint: Hsla, message: &'static str, cx: &mut Context<Self>) -> Div {
        let colors = Colors::get(cx);
        h_flex()
            .flex_none()
            .gap_2()
            .mx(px(TEXT_INSET - 12.))
            .my_1()
            .px(px(12.))
            .py(px(8.))
            .rounded(px(6.))
            .bg(tint)
            .text_size(px(13.))
            .text_color(colors.text)
            .child(div().flex_1().child(message))
    }

    fn render_incoming_banner(&self, deleted: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let (message, keep, take) = if deleted {
            (
                "This note was deleted elsewhere while you were editing.",
                "Keep it",
                "Discard",
            )
        } else {
            (
                "This note was changed elsewhere while you were editing. The version you don't keep stays in its history.",
                "Keep mine",
                "Use theirs",
            )
        };
        let tint = Hsla::from(Colors::get(cx).warning).opacity(0.16);
        self.banner(tint, message, cx)
            .child(
                Button::new("keep-mine")
                    .small()
                    .primary()
                    .label(keep)
                    .on_click(cx.listener(|this, _, _, cx| this.keep_mine(cx))),
            )
            .child(
                Button::new("take-theirs")
                    .small()
                    .label(take)
                    .on_click(cx.listener(|this, _, window, cx| this.take_theirs(window, cx))),
            )
    }

    /// A quiet line when another note says nearly the same as this one,
    /// with Open, Merge and a way to say they're different.
    fn render_twin_banner(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let twin = self.meaning.read(cx).duplicate()?.clone();
        let colors = Colors::get(cx);
        let title = if twin.title.trim().is_empty() {
            "Untitled".to_owned()
        } else {
            twin.title.clone()
        };
        let percent = (twin.score.clamp(0., 1.) * 100.).floor() as u32;
        let (open_id, merge_id) = (twin.id.clone(), twin.id.clone());
        let merge_title = title.clone();
        Some(
            h_flex()
                .flex_none()
                .gap_2()
                .mx(px(TEXT_INSET - 12.))
                .my_1()
                .px(px(12.))
                .py(px(5.))
                .rounded(px(6.))
                .bg(colors.fill)
                .text_size(px(12.5))
                .text_color(colors.text_secondary)
                .debug_selector(|| "twin-banner".into())
                .child(
                    Icon::new(Lucide::Copy)
                        .size(px(13.))
                        .text_color(colors.text_muted),
                )
                .child(div().flex_1().min_w_0().truncate().child(format!(
                    "Says nearly the same as “{title}” ({percent}% alike)"
                )))
                .child(
                    Button::new("twin-open")
                        .ghost()
                        .xsmall()
                        .label("Open")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            let id = open_id.clone();
                            this.library
                                .update(cx, |library, cx| library.select(Some(id), cx));
                        })),
                )
                .child(
                    Button::new("twin-merge")
                        .ghost()
                        .xsmall()
                        .label("Merge…")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.ask_to_merge(merge_id.clone(), merge_title.clone(), window, cx)
                        })),
                )
                .child(
                    Button::new("twin-dismiss")
                        .ghost()
                        .xsmall()
                        .label("Not the same")
                        .tooltip("Don't mention these two again while Kanso is open")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.meaning
                                .update(cx, |meaning, cx| meaning.dismiss_duplicate(cx));
                            cx.notify();
                        })),
                )
                .into_any_element(),
        )
    }

    /// While a recording is transcribed into this note: its name, how far
    /// along it is, and Cancel.
    fn render_transcribing(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let shown = crate::transcribe::shown_for(self.note_id()?, cx)?;
        let colors = Colors::get(cx);
        let words = match (shown.waiting, shown.share) {
            (true, _) => format!(
                "“{}” is transcribed once the speech model has downloaded",
                shown.name
            ),
            (false, Some(share)) => format!(
                "Transcribing “{}”… {}%",
                shown.name,
                (share * 100.).floor() as u32
            ),
            (false, None) => format!("Transcribing “{}”…", shown.name),
        };
        let share = shown.share.unwrap_or(0.);
        Some(
            h_flex()
                .flex_none()
                .gap_2()
                .mx(px(TEXT_INSET - 12.))
                .my_1()
                .px(px(12.))
                .py(px(5.))
                .rounded(px(6.))
                .bg(colors.fill)
                .text_size(px(12.5))
                .text_color(colors.text_secondary)
                .debug_selector(|| "transcribing".into())
                .child(
                    Icon::new(Lucide::Mic)
                        .size(px(13.))
                        .text_color(colors.text_muted),
                )
                .child(div().flex_1().min_w_0().truncate().child(words))
                .child(
                    div()
                        .flex_none()
                        .w(px(80.))
                        .h(px(4.))
                        .rounded_full()
                        .bg(colors.divider)
                        .child(
                            div()
                                .h_full()
                                .w(px(80. * share.clamp(0., 1.)))
                                .rounded_full()
                                .bg(colors.accent),
                        ),
                )
                .child(
                    Button::new("transcribe-cancel")
                        .ghost()
                        .xsmall()
                        .label("Cancel")
                        .on_click(|_, _, cx| crate::transcribe::cancel(cx)),
                )
                .into_any_element(),
        )
    }

    /// Asks before merging note `other` into the open note.
    fn ask_to_merge(
        &mut self,
        other: String,
        title: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let editor = cx.entity().downgrade();
        crate::dialogs::confirm(
            format!("Merge “{title}” into this note?"),
            "Its text goes at the end of this note, under its title, with its tags, and it moves to the trash. Both notes' versions from before stay in their history.",
            "Merge",
            window,
            cx,
            move |window, cx| {
                editor
                    .update(cx, |editor, cx| editor.merge_from(&other, window, cx))
                    .ok();
            },
        );
    }

    /// Merges note `other` into the open note, after saving what was typed
    /// (`Store::merge_notes`), and shows the result.
    pub fn merge_from(&mut self, other: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.flush(cx);
        let Some(note) = self.note.clone() else {
            return;
        };
        if self.incoming.is_some() || note.is_locked() {
            cx.emit(EditorEvent::Notice(
                "This note can't be merged right now.".into(),
            ));
            return;
        }
        let other_rev = match self.store.note(other) {
            Ok(Some(other)) => other.rev.unwrap_or_default(),
            _ => {
                cx.emit(EditorEvent::Notice("The other note is gone.".into()));
                return;
            }
        };
        let rev = note.rev.clone().unwrap_or_default();
        match self
            .store
            .merge_notes(&note.id, &rev, other, &other_rev, Origin::Ui)
        {
            Ok(_) => {
                self.meaning
                    .update(cx, |meaning, cx| meaning.dismiss_duplicate(cx));
                if let Ok(Some(merged)) = self.store.note(&note.id) {
                    self.show(Some(merged), true, window, cx);
                }
            }
            Err(err) => cx.emit(EditorEvent::Notice(format!(
                "Couldn't merge the notes: {err}"
            ))),
        }
    }

    /// What shows in place of a locked note's text until it is unlocked.
    fn render_locked(&self, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let problem = match &self.lock {
            LockView::Closed(problem) => problem.clone(),
            _ => None,
        };
        v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .gap_3()
            .px(px(24.))
            .pb(px(TOP_BAND_HEIGHT))
            .debug_selector(|| "locked-note".into())
            .child(
                Icon::new(Lucide::Lock)
                    .size(px(28.))
                    .text_color(colors.text_muted),
            )
            .child(div().text_color(colors.text).child("This note is locked."))
            .child(
                div()
                    .text_size(px(13.))
                    .text_color(colors.text_muted)
                    .child(
                        problem
                            .clone()
                            .unwrap_or_else(|| "Its text opens with your lock password.".into()),
                    ),
            )
            .when(problem.is_none(), |el| {
                el.child(
                    Button::new("unlock-note")
                        .small()
                        .primary()
                        .label("Unlock…")
                        .debug_selector(|| "unlock-note".into())
                        .on_click(|_, window, cx| {
                            window.dispatch_action(Box::new(crate::locked::UnlockNote), cx)
                        }),
                )
            })
            .into_any_element()
    }

    fn render_trash_banner(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let tint = Colors::get(cx).fill.into();
        self.banner(tint, "This note is in the trash.", cx).child(
            Button::new("restore-from-trash")
                .small()
                .label("Restore")
                .on_click(cx.listener(|this, _, _, cx| {
                    let book = this.library.read(cx).default_book();
                    if let Some(book) = book {
                        this.set_book(book, cx);
                    }
                })),
        )
    }
}

/// What the header says about saving the open note.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SaveState {
    /// Typing paused; the save is about to run.
    Saving,
    /// The editor holds text the note doesn't: a change made elsewhere is
    /// waiting for an answer, or a save failed.
    NotSaved,
    Saved,
}

/// Whether a Replace or a locked account holds editor writes.
fn writes_held(cx: &App) -> bool {
    cx.try_global::<crate::services::Services>()
        .is_some_and(|services| {
            services.writes_held
                || (services.has_locked_session() && !services.locked_replace_write_consent())
        })
}

/// What the header says, from whether a change made elsewhere is waiting,
/// whether a save is pending, and (asked only when neither) whether the
/// editor's text differs from the note's.
// no-prop-test: three flags; `save_states` checks all of them.
fn save_state(conflict: bool, pending: bool, unsaved: impl FnOnce() -> bool) -> SaveState {
    if conflict {
        SaveState::NotSaved
    } else if pending {
        SaveState::Saving
    } else if unsaved() {
        SaveState::NotSaved
    } else {
        SaveState::Saved
    }
}

pub fn rgb_color(rgb: u32) -> Hsla {
    gpui_kit::rgb(rgb).into()
}

/// An image to attach: bytes from the clipboard with their format's file
/// extension, or a file.
enum Pasted {
    Bytes(Vec<u8>, &'static str),
    File(std::path::PathBuf),
}

/// Stores one image as an attachment: its ID, or what to tell the user. A
/// file over the limit isn't read at all.
// no-prop-test: file reading around `smart_paste::attach_image`, which is
// tested with a store.
/// Whether a dropped file is an Excalidraw drawing.
fn is_drawing(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("excalidraw"))
}

/// What a pasted or dropped file becomes in the note: an attached image's
/// Markdown, or a drawing's fenced block (kept in the note itself, so it
/// syncs and versions with it).
fn attach(store: &Store, image: Pasted) -> Result<String, String> {
    use kanso_core::model::MAX_ATTACHMENT_SIZE;
    match image {
        Pasted::Bytes(bytes, ext) => smart_paste::attach_image(store, &bytes, ext, None)
            .map(|id| smart_paste::image_markdown(&id)),
        Pasted::File(path) if is_drawing(&path) => {
            let text = std::fs::read_to_string(&path)
                .map_err(|err| format!("Couldn't read the drawing: {err}"))?;
            if !kanso_render::excalidraw::is_scene(&text) {
                return Err("That file isn't an Excalidraw drawing.".into());
            }
            crate::preview_rich::scene_block(&text).map_err(|failure| failure.badge())
        }
        Pasted::File(path) => {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or_default();
            let name = path.file_name().and_then(|n| n.to_str());
            if let Ok(meta) = std::fs::metadata(&path)
                && meta.len() > MAX_ATTACHMENT_SIZE as u64
            {
                return Err(smart_paste::too_large(
                    meta.len() as usize,
                    MAX_ATTACHMENT_SIZE,
                ));
            }
            let bytes =
                std::fs::read(&path).map_err(|err| format!("Couldn't read the image: {err}"))?;
            smart_paste::attach_image(store, &bytes, ext, name)
                .map(|id| smart_paste::image_markdown(&id))
        }
    }
}

/// The HTML on the clipboard, for a paste. Tests set their own rather than
/// read the computer's.
#[cfg(not(test))]
fn clipboard_html() -> Option<String> {
    crate::clipboard::read_html()
}

#[cfg(test)]
thread_local! {
    static TEST_HTML: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
fn clipboard_html() -> Option<String> {
    TEST_HTML.with(|html| html.borrow().clone())
}

/// Whether a file looks like an image Kanso can attach, by its extension.
fn image_path(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| smart_paste::stored_as(ext).is_some())
}

/// The character a key types, when it types one and no command modifier is
/// held.
// no-prop-test: reads three flags and a string; the pairing window test
// types through it.
fn typed_char(keystroke: &Keystroke) -> Option<char> {
    let modifiers = keystroke.modifiers;
    if modifiers.control || modifiers.platform || modifiers.function {
        return None;
    }
    let typed = keystroke.key_char.as_deref()?;
    let mut chars = typed.chars();
    let ch = chars.next()?;
    chars.next().is_none().then_some(ch)
}

/// The UTF-8 offset in `text` of UTF-16 offset `utf16`, clamped to the end.
fn utf8_offset(text: &str, utf16: usize) -> usize {
    let mut units = 0;
    for (ix, ch) in text.char_indices() {
        if units >= utf16 {
            return ix;
        }
        units += ch.len_utf16();
    }
    text.len()
}

/// Five bars that rise with `level` (0 to 1), the middle one most, in the
/// danger color: the microphone is on.
// no-prop-test: drawing; the voice snapshots show it.
fn level_meter(level: f32, colors: &Colors) -> impl IntoElement {
    let level = level.clamp(0., 1.).sqrt();
    h_flex()
        .h(px(14.))
        .gap(px(2.))
        .items_center()
        .children([0.5f32, 0.8, 1., 0.8, 0.5].map(|weight| {
            div()
                .w(px(2.5))
                .h(px(3. + 11. * level * weight))
                .rounded_full()
                .bg(colors.danger)
        }))
}

/// Applies `edit`, made against `text`, to the body as one step that undoes
/// at once. An edit that changes nothing only moves the selection, and
/// leaves nothing to undo.
// no-prop-test: three calls into the editor; the window tests undo
// through it.
/// Whether spoken punctuation and commands act on what is said
/// (Settings › Editor › Voice input).
fn spoken_commands(cx: &App) -> bool {
    cx.try_global::<AppState>()
        .is_none_or(|state| state.config.voice.spoken_commands)
}

/// The words voice input has shown so far (`NoteEditor::live_voice`).
struct LiveVoice {
    /// The text, and its selection, when the first words came: each
    /// update is worked out against them, as the transcript would be.
    base: String,
    selection: std::ops::Range<usize>,
    /// The words being composed, and where they start.
    shown: String,
    start: usize,
    /// The composing ended (a key, a click) before the transcript came.
    lost: bool,
}

/// The one edit that turns `text` into `body`, keeping the caret where it
/// was when it isn't in the part replaced. `None` when they are the same.
fn body_edit(text: &str, body: &str, selection: std::ops::Range<usize>) -> Option<Edit> {
    if text == body {
        return None;
    }
    let mut start = common_prefix(text, body);
    while start > 0 && (!text.is_char_boundary(start) || !body.is_char_boundary(start)) {
        start -= 1;
    }
    let mut suffix = common_suffix(&text[start..], &body[start..]);
    while suffix > 0
        && (!text.is_char_boundary(text.len() - suffix)
            || !body.is_char_boundary(body.len() - suffix))
    {
        suffix -= 1;
    }
    let end = text.len() - suffix;
    let after = body.len() - suffix;
    let selection = if selection.end <= start {
        selection
    } else if selection.start >= end {
        let shift = body.len() as isize - text.len() as isize;
        let shift = |at: usize| (at as isize + shift).max(0) as usize;
        shift(selection.start)..shift(selection.end)
    } else {
        after..after
    };
    Some(Edit {
        range: start..end,
        text: body[start..body.len() - suffix].to_owned(),
        selection,
    })
}

/// The length of the bytes `a` and `b` start with in common.
fn common_prefix(a: &str, b: &str) -> usize {
    a.bytes().zip(b.bytes()).take_while(|(a, b)| a == b).count()
}

/// The length of the bytes `a` and `b` end with in common.
fn common_suffix(a: &str, b: &str) -> usize {
    a.bytes()
        .rev()
        .zip(b.bytes().rev())
        .take_while(|(a, b)| a == b)
        .count()
}

fn apply_edit(
    state: &mut EditorState,
    text: &str,
    edit: Edit,
    window: &mut Window,
    cx: &mut Context<EditorState>,
) {
    if text.get(edit.range.clone()) != Some(edit.text.as_str()) {
        if state.selected_range() != edit.range {
            state.set_selected_range(edit.range, cx);
        }
        state.replace(edit.text, window, cx);
    }
    if state.selected_range() != edit.selection {
        state.set_selected_range(edit.selection, cx);
    }
}

impl Render for NoteEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let account_locked = cx
            .try_global::<crate::services::Services>()
            .is_some_and(crate::services::Services::has_locked_session);
        self.set_account_read_only(account_locked, cx);
        // The face follows the view mode and the font setting, however they
        // changed.
        let face = self
            .editor_font
            .face(self.view_mode)
            .with_line_numbers(self.allow_line_numbers);
        if face.line_numbers != self.line_numbers {
            self.line_numbers = face.line_numbers;
            self.body.update(cx, |body, cx| {
                body.set_line_number(face.line_numbers, window, cx);
                body.set_folding(face.line_numbers, window, cx);
            });
        }
        let colors = Colors::get(cx);
        let focus_mode = self.focus_mode;
        let Some(note) = self.note.clone() else {
            let pane = v_flex()
                .flex_1()
                .min_h_0()
                .w_full()
                .items_center()
                .justify_center()
                .track_focus(&self.empty_focus);
            let library = self.library.read(cx);
            let first_run = library.is_empty_library();
            let place = library.empty_place();
            // An empty notebook offers its first note; an empty library,
            // whatever shows, its very first.
            let content = if first_run || place.is_some() {
                let title = place.unwrap_or_else(|| "No notes yet".into());
                let block = EmptyNotebook::titled(title, self.empty_focus.clone());
                // Centred a little above the middle, as the list's is: the
                // top band's room less the bottom band's.
                pane.px(px(24.))
                    .pb(px(TOP_BAND_HEIGHT - BOTTOM_BAND_HEIGHT))
                    .child(if first_run { block.first_run() } else { block })
            } else {
                // The shortcut as bound, so the hint never shows a stale one.
                let new_note = crate::keys::shortcut_chip(&NewNote, window, cx);
                pane.gap_2()
                    .text_color(colors.text_muted)
                    .child("No note selected")
                    .children(new_note.map(|key| {
                        h_flex()
                            .gap_1p5()
                            .text_xs()
                            .child(key)
                            .child("creates a note")
                    }))
            };
            // The bottom band runs on under the editor with no note too, so
            // the bands line up across the panes.
            return v_flex()
                .size_full()
                .bg(colors.editor)
                .child(content)
                .child(
                    AnyView::from(self.footer.clone()).cached(
                        StyleRefinement::default()
                            .w_full()
                            .flex_none()
                            .h(px(BOTTOM_BAND_HEIGHT)),
                    ),
                )
                .into_any_element();
        };
        let trashed = note.is_trashed();
        let mode = self.view_mode;
        let installed = crate::fonts::installed(cx);
        let family = if face.mono {
            crate::fonts::family(
                self.mono_family.as_deref(),
                &installed,
                cx.theme().mono_font_family.clone(),
            )
        } else {
            crate::fonts::family(
                self.font_family.as_deref(),
                &installed,
                cx.theme().font_family.clone(),
            )
        };
        let size = self.font_size;
        // Alone, the source keeps to a readable measure for its face and
        // size, starting under the title, unless the setting lets it fill the
        // pane; beside the preview it takes its side of the divider.
        let measure = (mode == ViewMode::Edit && self.readable_width).then(|| {
            let numbers = if face.line_numbers {
                LINE_NUMBER_COLUMN
            } else {
                0.
            };
            px(readable_width(size, face.mono) + SOURCE_EDGES + numbers)
        });
        // In focus mode the header band waits, invisible but in place so
        // nothing moves, until the pointer comes near the top. The status,
        // tags and toolbar go: kept in place, invisible, they left a wide
        // empty band between the title and the text.
        let hide_chrome = self.chrome_hidden();
        // Alone in its window, the band starts after the traffic lights.
        let band_inset = if self.standalone && !focus_mode {
            leading_inset(true, window.is_fullscreen())
        } else {
            0.
        };
        let chrome = |el: Div| {
            el.flex_none().when(hide_chrome, |el| {
                el.opacity(0.)
                    .group_hover(TOP_GROUP, |style| style.opacity(1.))
            })
        };
        let top = v_flex()
            .flex_none()
            .group(TOP_GROUP)
            .child(chrome(v_flex()).child(self.render_band(&note, band_inset, cx)))
            .child(
                div()
                    .flex_none()
                    .pl(px(TEXT_INSET))
                    .pr_4()
                    .pb(px(8.))
                    .child(self.render_title(window, cx)),
            )
            .when(!focus_mode, |el| {
                el.child(v_flex().flex_none().child(self.render_meta(&note, cx)))
            })
            .when(
                mode == ViewMode::Edit && !trashed && !focus_mode && !self.lock.is_closed(),
                |el| el.child(v_flex().flex_none().child(self.render_format_bar(cx))),
            );

        // Typewriter scrolling: the caret's line stays in the middle.
        let typewriter = self.settings.typewriter.centres(focus_mode);
        if typewriter != self.typewriter {
            self.typewriter = typewriter;
            self.body.update(cx, |body, cx| {
                body.set_cursor_surrounding_lines(
                    typewriter.then_some(TYPEWRITER_LINES),
                    window,
                    cx,
                )
            });
        }
        // The writing aids take their keys before the text editor does.
        let source = Self::with_format_shortcuts(div(), cx)
            .flex_1()
            .h_full()
            .min_w_0()
            .capture_key_down(cx.listener(Self::on_key_down))
            .capture_action(cx.listener(Self::on_backspace))
            .capture_action(
                cx.listener(|this, _: &IndentInline, window, cx| this.on_tab(false, window, cx)),
            )
            .capture_action(
                cx.listener(|this, _: &OutdentInline, window, cx| this.on_tab(true, window, cx)),
            )
            .capture_action(cx.listener(Self::on_enter))
            .capture_action(cx.listener(Self::on_paste))
            .on_action(cx.listener(Self::correct_spelling))
            .on_action(cx.listener(Self::inline_assist))
            .on_action(cx.listener(Self::learn_spelling))
            .on_action(cx.listener(Self::ignore_spelling))
            .on_drop(cx.listener(Self::on_drop))
            .pl(px(TEXT_INSET - SOURCE_TEXT_LEFT))
            .pr_4()
            .py_2()
            // The margin beside a narrowed source scrolls it too; over the
            // text, the editor takes the wheel first.
            .when(measure.is_some(), |el| {
                el.on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, _, cx| {
                    this.body.update(cx, |body, cx| {
                        let line = body.line_height().unwrap_or(px(24.));
                        let offset = body.scroll_offset() + event.delta.pixel_delta(line);
                        body.set_scroll_offset(offset, cx);
                    });
                }))
            })
            .child(
                div()
                    .h_full()
                    .w_full()
                    .relative()
                    .when_some(measure, |el, width| el.max_w(width))
                    .child(
                        Editor::new(&self.body)
                            .context_menu({
                                let view = cx.entity().downgrade();
                                move |menu, _, cx| {
                                    view.upgrade()
                                        .map_or(menu, |view| view.read(cx).text_menu(cx))
                                }
                            })
                            .bordered(false)
                            .h_full()
                            .font_family(family)
                            .text_size(px(size))
                            .line_height(relative(BODY_LINE_HEIGHT)),
                    )
                    // Vim's block caret, over the text.
                    .children(self.vim_caret(window, cx))
                    // Other people's carets in a shared note.
                    .children(self.remote_carets(window, cx)),
            );
        let readable = self.readable_width;
        self.preview
            .update(cx, |preview, cx| preview.set_readable(readable, cx));
        let preview = div()
            .flex_1()
            .h_full()
            .min_w_0()
            .debug_selector(|| "preview-pane".into())
            .child(
                self.preview
                    .clone()
                    .cached(StyleRefinement::default().size_full()),
            );
        let body = match mode {
            _ if self.lock.is_closed() => self.render_locked(cx),
            ViewMode::Edit => h_flex().size_full().child(source).into_any_element(),
            ViewMode::Preview => h_flex().size_full().child(preview).into_any_element(),
            ViewMode::Split => {
                self.place_split(window, cx);
                let editor = cx.entity().downgrade();
                // Until the divider is placed for a measured width, the
                // sides grow in the kept proportion from nothing. Neither
                // may be dragged narrower than the editor has room for, so
                // the preview never runs past the window.
                let ratio = self.split_ratio.clamp(0., 1.);
                let min_side =
                    split_min_side(crate::zoom::unscaled(self.split.read(cx).container_size()));
                let side = |share: f32| {
                    let mut panel = resizable_panel()
                        .size_range(px(min_side)..gpui_kit::px(f32::MAX))
                        .min_w_0()
                        .flex_basis(px(0.));
                    panel.style().flex_grow = Some(share);
                    panel
                };
                // Painted after the source, which by then has taken any
                // scroll set for this frame (from its scrollbar, or the
                // margin's wheel), which it doesn't announce. The preview
                // follows once the frame is drawn, so moving it asks for the
                // next one.
                let follow = {
                    let editor = editor.clone();
                    canvas(
                        |_, _, _| {},
                        move |_, _, window, cx| {
                            let editor = editor.clone();
                            window.defer(cx, move |_, cx| {
                                editor
                                    .update(cx, |this, cx| this.follow_source_scroll(cx))
                                    .ok();
                            });
                        },
                    )
                    .absolute()
                    .size_0()
                };
                h_resizable("split")
                    .with_state(&self.split)
                    .on_resize(move |state, _, cx| {
                        let (left, width) = {
                            let state = state.read(cx);
                            (state.sizes().first().copied(), state.container_size())
                        };
                        editor
                            .update(cx, |this, cx| this.split_dragged(left, width, cx))
                            .ok();
                    })
                    .child(side(ratio).child(source).child(follow))
                    .child(side(1. - ratio).child(preview))
                    .into_any_element()
            }
        };

        let column = v_flex()
            .size_full()
            .bg(colors.editor)
            .child(top)
            .when(trashed, |el| el.child(self.render_trash_banner(cx)))
            .when_some(self.incoming.as_ref(), |el, incoming| {
                let deleted = matches!(incoming, Incoming::Deleted);
                el.child(self.render_incoming_banner(deleted, cx))
            })
            .when(!trashed && self.incoming.is_none(), |el| {
                el.children(self.render_twin_banner(cx))
            })
            .children(self.render_transcribing(cx))
            .children(self.render_suggestion(cx))
            .when(self.shared.as_ref().is_some_and(|s| s.ended), |el| {
                el.child(self.banner(
                    colors.fill.into(),
                    "This note is no longer shared. It's your private copy now.",
                    cx,
                ))
            })
            .children(self.render_problems(cx))
            .child(div().flex_1().min_h_0().w_full().child(body))
            // Redrawn only when the cursor or the counts change.
            .child(
                AnyView::from(self.footer.clone()).cached(
                    StyleRefinement::default()
                        .w_full()
                        .flex_none()
                        .h(px(BOTTOM_BAND_HEIGHT)),
                ),
            );
        if !focus_mode {
            return column.into_any_element();
        }
        // Focus mode: one centred column. Alone, the text keeps its
        // readable measure with the same room either side of it, so the
        // text itself is centred; beside the preview each side gets as much.
        let width = match mode {
            ViewMode::Split => 2. * FOCUS_COLUMN,
            _ if !self.readable_width => FOCUS_COLUMN,
            _ => {
                let numbers = if face.line_numbers {
                    LINE_NUMBER_COLUMN
                } else {
                    0.
                };
                (readable_width(size, face.mono) + numbers + 2. * TEXT_INSET).min(FOCUS_COLUMN)
            }
        };
        // The note is leftmost now: a column as wide as the window leaves
        // the traffic lights their room on both sides.
        let lights = leading_inset(true, window.is_fullscreen());
        let width = width.min(crate::zoom::unscaled(window.viewport_size().width) - 2. * lights);
        h_flex()
            .size_full()
            .justify_center()
            .bg(colors.editor)
            .child(column.w_full().max_w(px(width)))
            .into_any_element()
    }
}

#[cfg(test)]
impl NoteEditor {
    /// The source editor's state, to type into.
    pub fn body_state(&self) -> Entity<EditorState> {
        self.body.clone()
    }

    /// How many misspelled words the text has underlined.
    pub fn spelling_marks_for_test(&self) -> usize {
        self.spell_marks.count()
    }

    /// The spelling items of the context menu, where the selection is.
    pub fn spelling_menu_for_test(&self, cx: &App) -> Vec<(String, Box<dyn Action>)> {
        self.spelling_items(cx)
    }

    /// How many due dates the text has tinted.
    pub fn due_marks_for_test(&self) -> usize {
        self.due_marks.count()
    }

    /// What the open completion list offers, by label; empty while closed.
    pub fn completion_labels_for_test(&self, cx: &App) -> Vec<String> {
        let menu = self.body.read(cx).completion_menu_state();
        if !menu.open {
            return Vec::new();
        }
        menu.items.iter().map(|item| item.label.clone()).collect()
    }

    /// Types `title` into the title field, as if typed.
    pub fn set_title_for_test(&mut self, title: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.title.update(cx, |state, cx| {
            state.set_value(title.to_owned(), window, cx)
        });
        self.schedule_autosave(cx);
    }

    pub fn set_preview_for_test(&self, text: String, cx: &mut App) {
        self.preview.update(cx, |p, cx| p.set_text(text.into(), cx));
    }

    /// The Markdown the preview shows.
    pub fn preview_shown_for_test(&self, cx: &App) -> String {
        self.preview.read(cx).shown().to_string()
    }

    /// How many chunks the preview shows the note in.
    pub fn preview_chunks_for_test(&self, cx: &App) -> usize {
        self.preview.read(cx).chunk_count()
    }

    /// Replaces the title and body as if typed, scheduling the autosave.
    pub fn edit_for_test(&mut self, body: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.body
            .update(cx, |state, cx| state.set_value(body.to_owned(), window, cx));
        self.schedule_autosave(cx);
    }

    pub fn body_for_test(&self, cx: &App) -> String {
        self.current_text(cx).1
    }

    /// How far the body is scrolled.
    pub fn body_scroll_for_test(&self, cx: &App) -> Point<Pixels> {
        self.body.read(cx).scroll_offset()
    }

    pub fn scroll_body_for_test(&self, offset: Point<Pixels>, cx: &mut App) {
        self.body
            .update(cx, |body, cx| body.set_scroll_offset(offset, cx));
    }

    pub fn has_incoming(&self) -> bool {
        self.incoming.is_some()
    }

    /// The divider's share as a drag in an earlier session left it; it is
    /// placed on the next draw.
    pub fn keep_split_ratio(&mut self, ratio: f32, cx: &mut Context<Self>) {
        self.split_ratio = ratio;
        self.split_placed_for = px(0.);
        cx.notify();
    }

    /// Scrolls the source `share` of the way down, as the wheel would.
    pub fn scroll_source_for_test(&mut self, share: f32, cx: &mut Context<Self>) {
        let top = self.source_scroll_estimate(cx).unwrap_or(0.) * share;
        self.body.update(cx, |body, cx| {
            body.set_scroll_offset(point(px(0.), px(-top)), cx)
        });
    }

    pub fn split_ratio(&self) -> f32 {
        self.split_ratio
    }

    pub fn preview_scroll(&self, cx: &App) -> (f32, f32) {
        let preview = self.preview.read(cx);
        (preview.scroll_top(), preview.scroll_max())
    }

    pub fn source_scroll(&self, cx: &App) -> f32 {
        -f32::from(self.body.read(cx).scroll_offset().y)
    }

    pub fn title_focused(&self, window: &Window, cx: &App) -> bool {
        self.title.read(cx).focus_handle(cx).is_focused(window)
    }

    pub fn body_focused(&self, window: &Window, cx: &App) -> bool {
        self.body.read(cx).focus_handle(cx).is_focused(window)
    }

    pub fn split_sizes(&self, cx: &App) -> Vec<f32> {
        self.split
            .read(cx)
            .sizes()
            .iter()
            .map(|s| f32::from(*s))
            .collect()
    }

    /// Shows what focus mode hides, as the pointer near the top and over
    /// the footer would.
    pub fn reveal_chrome_for_test(&mut self, cx: &mut Context<Self>) {
        self.reveal_chrome = true;
        self.footer.update(cx, |footer, cx| {
            footer.reveal = true;
            cx.notify();
        });
        cx.notify();
    }

    /// Scrolls the preview `top` points down, as its wheel would.
    pub fn scroll_preview_for_test(&mut self, top: f32, cx: &mut Context<Self>) {
        self.preview
            .update(cx, |preview, cx| preview.scroll_to(top, cx));
    }

    /// The cached source line lengths, as (text length, lines).
    pub fn source_lines_for_test(&self) -> Option<(usize, Vec<usize>)> {
        self.source_lines.clone()
    }

    /// Fills the source line cache, as a paint in Split does.
    pub fn estimate_source_for_test(&mut self, cx: &mut Context<Self>) -> Option<f32> {
        self.source_scroll_estimate(cx)
    }
}

/// Notes outside the trash that link to note `id`, from the store's index
/// of links: by ID link, or by a wiki-link to its title.
pub fn backlinks(store: &Store, id: &str) -> Vec<Backlink> {
    store.backlinks(id).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{NoteEditor, SaveState, backlinks, body_edit, same_content, save_state};
    use crate::config::Appearance;
    use crate::library::Library;
    use crate::markdown_edit::Format;
    use gpui_kit::{AppContext as _, TestAppContext};
    use kanso_core::DocType;
    use kanso_store::{Origin, Put, Store};
    use proptest::prelude::*;
    use serde_json::json;
    use std::sync::Arc;
    use std::time::Duration;

    type Opened = (
        gpui_kit::AnyWindowHandle,
        gpui_kit::Entity<NoteEditor>,
        Arc<Store>,
        String,
    );

    /// An editor in a window, on a new note with `body`.
    fn editor_on(cx: &mut TestAppContext, dir: &std::path::Path, body: &str) -> Opened {
        let store = Arc::new(Store::open(dir.join("k.db")).unwrap());
        let book = store
            .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
            .unwrap()
            .id;
        let id = store
            .put(
                DocType::Note,
                json!({"title": "t", "body": body, "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let editor_store = store.clone();
        let (window, editor) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(Default::default(), cx, |window, cx| {
                let library = cx.new(|cx| Library::new(editor_store.clone(), cx));
                let (links, _) = async_channel::unbounded();
                cx.new(|cx| {
                    NoteEditor::new(
                        editor_store,
                        library,
                        links,
                        &Appearance::default(),
                        window,
                        cx,
                    )
                })
            })
            .unwrap()
        });
        (window, editor, store, id)
    }

    #[test]
    fn a_body_edit_replaces_only_what_changed() {
        let edit = body_edit("one\ntwo\nthree", "one\nTWO\nthree", 0..0).unwrap();
        assert_eq!(edit.apply("one\ntwo\nthree"), "one\nTWO\nthree");
        assert_eq!(edit.range, 4..7);
        assert!(body_edit("same", "same", 0..0).is_none());
    }

    proptest! {
        /// Making `body` from `text` is one replacement that leaves the text
        /// as `body`, and its caret sits in it.
        #[test]
        fn prop_body_edit_makes_the_body(
            text in "\\PC{0,80}",
            body in "\\PC{0,80}",
            end in any::<bool>(),
        ) {
            let selection = if end {
                text.len()..text.len()
            } else {
                0..text.len()
            };
            match body_edit(&text, &body, selection.clone()) {
                None => prop_assert_eq!(text, body),
                Some(edit) => {
                    prop_assert_eq!(edit.apply(&text), body.clone());
                    prop_assert!(edit.selection.start <= edit.selection.end);
                    prop_assert!(edit.selection.end <= body.len());
                    prop_assert!(body.is_char_boundary(edit.selection.start));
                    prop_assert!(body.is_char_boundary(edit.selection.end));
                }
            }
        }
    }

    /// The toolbar edits the body the way typing does: at the selection,
    /// saved after the pause, and counted in the footer once typing rests.
    // no-prop-test: glue between the toolbar, the text editor and autosave;
    // the edits themselves are property-tested in `kanso_engine::edit`.
    #[gpui_kit::test]
    fn formatting_edits_the_body_and_saves(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let (window, editor, store, id) = editor_on(cx, dir.path(), "Goals\nnext");
        let edit = |cx: &mut TestAppContext,
                    f: &dyn Fn(
            &mut NoteEditor,
            &mut gpui_kit::Window,
            &mut gpui_kit::Context<NoteEditor>,
        )| {
            window
                .update(cx, |_, window, cx| {
                    editor.update(cx, |editor, cx| f(editor, window, cx))
                })
                .unwrap();
            cx.executor().advance_clock(Duration::from_secs(1));
            cx.run_until_parked();
        };
        edit(cx, &|editor, window, cx| editor.open(Some(&id), window, cx));
        edit(cx, &|editor, window, cx| {
            editor.apply_format(Format::Heading(2), window, cx)
        });
        assert_eq!(store.note(&id).unwrap().unwrap().body, "## Goals\nnext");
        edit(cx, &|editor, window, cx| {
            editor
                .body
                .update(cx, |body, cx| body.set_selected_range(9..13, cx));
            editor.apply_format(Format::Bold, window, cx);
        });
        assert_eq!(store.note(&id).unwrap().unwrap().body, "## Goals\n**next**");
        let label = cx.read(|cx| editor.read(cx).footer.read(cx).label());
        assert!(label.ends_with("2 words · 17 chars"), "{label}");
    }

    /// A recording linked on the caret's line is found for Transcribe
    /// Recording, and its transcript goes in under it as one edit, saved,
    /// with the caret left where it was.
    // no-prop-test: glue; `transcribe::transcript_edit` has the properties.
    #[gpui_kit::test]
    fn a_transcript_goes_under_its_recording(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let (window, editor, store, id) = editor_on(cx, dir.path(), "");
        let wav = dir.path().join("memo.wav");
        std::fs::write(&wav, b"RIFF").unwrap();
        let (file_id, link) = crate::transcribe::attach_audio(&store, &wav)
            .unwrap()
            .unwrap();
        let rev = store.note(&id).unwrap().unwrap().rev.unwrap();
        let body = format!("Notes\n{link}\nLater");
        store
            .put(
                DocType::Note,
                json!({"_id": id, "_rev": rev, "body": body}),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap();
        let run = |cx: &mut TestAppContext,
                   f: &dyn Fn(
            &mut NoteEditor,
            &mut gpui_kit::Window,
            &mut gpui_kit::Context<NoteEditor>,
        )| {
            window
                .update(cx, |_, window, cx| {
                    editor.update(cx, |editor, cx| f(editor, window, cx))
                })
                .unwrap();
            cx.executor().advance_clock(Duration::from_secs(2));
            cx.run_until_parked();
        };
        run(cx, &|editor, window, cx| editor.open(Some(&id), window, cx));
        // On the first line: no recording there.
        run(cx, &|editor, _, cx| {
            editor
                .body
                .update(cx, |body, cx| body.set_selected_range(2..2, cx))
        });
        assert_eq!(cx.read(|cx| editor.read(cx).audio_on_caret_line(cx)), None);
        run(cx, &|editor, _, cx| {
            editor
                .body
                .update(cx, |body, cx| body.set_selected_range(8..8, cx))
        });
        let found = cx.read(|cx| editor.read(cx).audio_on_caret_line(cx));
        assert_eq!(found, Some((file_id.clone(), "memo.wav".to_owned())));
        run(cx, &|editor, window, cx| {
            editor.insert_transcript_below(Some(&file_id), "Spoken words.", window, cx);
        });
        assert_eq!(
            store.note(&id).unwrap().unwrap().body,
            format!("Notes\n{link}\n\nSpoken words.\n\nLater")
        );
        let caret = cx.read(|cx| editor.read(cx).body.read(cx).selected_range());
        assert_eq!(caret, 8..8);
    }

    /// A note that says nearly the same as another is mentioned once both
    /// are embedded; merging saves what was typed first, keeps both texts
    /// and tags, and trashes the other. Dismissing stops the mention.
    // no-prop-test: glue; `merged_body` and `Store::merge_notes` have the
    // properties.
    #[gpui_kit::test]
    fn a_near_twin_is_mentioned_and_merges_without_loss(cx: &mut TestAppContext) {
        use crate::related::{MODEL_ID, Related};
        let dir = tempfile::tempdir().unwrap();
        let (window, editor, store, id) =
            editor_on(cx, dir.path(), "milk eggs bread butter cheese");
        let book = store.note(&id).unwrap().unwrap().book_id;
        let shop = store.tag_named("shop", Origin::Ui).unwrap();
        let twin = store
            .put(
                DocType::Note,
                json!({"title": "t", "body": "milk eggs bread butter cheese jam", "bookId": book, "tags": [shop]}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let letters = |text: &str| {
            let mut v = vec![0f32; 26];
            for c in text.bytes().filter(u8::is_ascii_lowercase) {
                v[(c - b'a') as usize] += 1.;
            }
            v
        };
        let embed = |store: &Store| {
            let queue = store.embedding_queue(MODEL_ID).unwrap();
            let rows: Vec<_> = queue
                .pending
                .into_iter()
                .map(|p| {
                    let v = letters(&p.text);
                    (p.id, p.hash, v)
                })
                .collect();
            store.put_embeddings(MODEL_ID, &rows).unwrap();
        };
        store.set_embedding_model(Some(MODEL_ID)).unwrap();
        embed(&store);
        let run = |cx: &mut TestAppContext,
                   f: &dyn Fn(
            &mut NoteEditor,
            &mut gpui_kit::Window,
            &mut gpui_kit::Context<NoteEditor>,
        )| {
            window
                .update(cx, |_, window, cx| {
                    editor.update(cx, |editor, cx| f(editor, window, cx))
                })
                .unwrap();
            cx.run_until_parked();
        };
        let new_embeddings = |cx: &mut TestAppContext, generation: u64| {
            cx.update(|cx| {
                if !cx.has_global::<Related>() {
                    cx.set_global(Related::default());
                }
                gpui_kit::BorrowAppContext::update_global::<Related, _>(cx, |related, _| {
                    related.generation = generation
                });
            });
            cx.run_until_parked();
        };
        run(cx, &|editor, window, cx| editor.open(Some(&id), window, cx));
        new_embeddings(cx, 1);
        let duplicate = |cx: &mut TestAppContext| {
            cx.read(|cx| {
                let meaning = editor.read(cx).meaning().clone();
                meaning.read(cx).duplicate().map(|d| d.id.clone())
            })
        };
        assert_eq!(duplicate(cx), Some(twin.clone()));
        // Dismissed: not mentioned again, from either note.
        run(cx, &|editor, _, cx| {
            editor
                .meaning
                .update(cx, |meaning, cx| meaning.dismiss_duplicate(cx))
        });
        assert_eq!(duplicate(cx), None);

        // Typed and not yet saved, then merged.
        run(cx, &|editor, window, cx| {
            editor.edit_for_test("milk eggs bread\nbutter", window, cx)
        });
        run(cx, &|editor, window, cx| {
            editor.merge_from(&twin, window, cx)
        });
        let kept = store.note(&id).unwrap().unwrap();
        assert_eq!(
            kept.body,
            "milk eggs bread\nbutter\n\n## t\n\nmilk eggs bread butter cheese jam\n"
        );
        assert_eq!(kept.tags, [shop]);
        assert_eq!(
            store.note(&twin).unwrap().unwrap().book_id,
            kanso_core::model::TRASH_BOOK_ID
        );
        let shown = cx.read(|cx| editor.read(cx).body_for_test(cx));
        assert_eq!(shown, kept.body);
        // Nothing waits to be saved over it.
        assert!(!cx.read(|cx| editor.read(cx).has_unsaved_edits(cx)));
    }

    /// Dictation, as macOS sends it: marked text that grows while you
    /// speak, committed in place, then a correction of an earlier word by
    /// its range. The text comes out as spoken, smart edits keep out while
    /// it is marked, an autosave in the middle doesn't disturb it, and the
    /// note is saved as committed.
    // no-prop-test: glue between the platform's input calls, the text editor
    // and autosave; there is no rule here beyond "what was spoken lands".
    #[gpui_kit::test]
    fn dictation_lands_as_spoken(cx: &mut TestAppContext) {
        use gpui_kit::EntityInputHandler as _;
        let dir = tempfile::tempdir().unwrap();
        let start = "Shopping\n- eggs";
        let (window, editor, store, id) = editor_on(cx, dir.path(), start);
        let step = |cx: &mut TestAppContext,
                    f: &dyn Fn(
            &mut NoteEditor,
            &mut gpui_kit::Window,
            &mut gpui_kit::Context<NoteEditor>,
        )| {
            window
                .update(cx, |_, window, cx| {
                    editor.update(cx, |editor, cx| f(editor, window, cx))
                })
                .unwrap();
            cx.run_until_parked();
        };
        step(cx, &|editor, window, cx| {
            editor.open(Some(&id), window, cx);
            editor.focus_body(window, cx);
            editor.body.update(cx, |body, cx| {
                body.set_selected_range(start.len()..start.len(), cx)
            });
        });
        // Words arrive as marked text, replacing the marked text before.
        for heard in [" and", " and milk"] {
            step(cx, &|editor, window, cx| {
                editor.body.update(cx, |body, cx| {
                    let end = heard.encode_utf16().count();
                    body.replace_and_mark_text_in_range(None, heard, Some(end..end), window, cx)
                });
            });
        }
        // Still speaking: no smart edit takes a key, and an autosave
        // leaves the marked text as it is.
        step(cx, &|editor, window, cx| {
            assert!(!editor.body_takes_edits(window, cx))
        });
        cx.executor().advance_clock(Duration::from_secs(30));
        cx.run_until_parked();
        step(cx, &|editor, window, cx| {
            let marked = editor
                .body
                .update(cx, |body, cx| body.marked_text_range(window, cx));
            assert_eq!(marked, Some(15..24));
            assert_eq!(editor.body.read(cx).value(), "Shopping\n- eggs and milk");
        });
        // Done speaking: committed in place, and the editor's own again.
        step(cx, &|editor, window, cx| {
            editor.body.update(cx, |body, cx| {
                body.replace_text_in_range(None, " and milk", window, cx)
            });
            assert!(editor.body_takes_edits(window, cx));
        });
        // "Replace milk with bread": a correction by range.
        step(cx, &|editor, window, cx| {
            editor.body.update(cx, |body, cx| {
                body.replace_text_in_range(Some(20..24), "bread", window, cx)
            });
        });
        cx.executor().advance_clock(Duration::from_secs(30));
        cx.run_until_parked();
        assert_eq!(
            store.note(&id).unwrap().unwrap().body,
            "Shopping\n- eggs and bread"
        );
    }

    /// Voice input's words show at the caret while someone speaks, each
    /// update in place of the last; the transcript replaces them as one
    /// step that undoes at once, and dropping them leaves the note as it
    /// was, even after a key ended the composing.
    // no-prop-test: drives the editor; `voice_edit`, `find_live` and
    // `LiveText` have the properties.
    #[gpui_kit::test]
    fn live_words_show_then_become_the_transcript(cx: &mut TestAppContext) {
        use gpui_kit::EntityInputHandler as _;
        let open = super::writing_tests::Open::new(cx, "Groceries:");
        open.select(cx, 10..10);
        for heard in ["Milk", "Milk and", "Milk and egg"] {
            open.with(cx, |editor, window, cx| {
                editor.show_live_transcript(heard, window, cx)
            });
        }
        assert_eq!(open.text(cx), ("Groceries: Milk and egg".into(), 23..23));
        open.with(cx, |editor, window, cx| {
            assert!(editor.insert_transcript("Milk and eggs.", window, cx));
        });
        assert_eq!(open.text(cx), ("Groceries: Milk and eggs.".into(), 25..25));
        open.keys(cx, "cmd-z");
        assert_eq!(open.text(cx).0, "Groceries:");

        // Dropped: nothing stays.
        open.select(cx, 10..10);
        open.with(cx, |editor, window, cx| {
            editor.show_live_transcript("Bread", window, cx)
        });
        open.with(cx, |editor, window, cx| {
            editor.drop_live_transcript(window, cx)
        });
        assert_eq!(open.text(cx).0, "Groceries:");

        // A click moved the caret mid-speech: the composing ended where the
        // words were, and the transcript still goes in their place.
        open.select(cx, 10..10);
        open.with(cx, |editor, window, cx| {
            editor.show_live_transcript("Tea", window, cx)
        });
        open.with(cx, |editor, window, cx| {
            editor
                .body
                .update(cx, |body, cx| body.unmark_text(window, cx))
        });
        open.select(cx, 0..0);
        open.with(cx, |editor, window, cx| {
            editor.show_live_transcript("Tea and", window, cx)
        });
        assert_eq!(open.text(cx).0, "Groceries: Tea");
        open.with(cx, |editor, window, cx| {
            assert!(editor.insert_transcript("Tea and cake.", window, cx));
        });
        assert_eq!(open.text(cx).0, "Groceries: Tea and cake.");
        open.with(cx, |editor, window, cx| {
            editor.show_live_transcript("Jam", window, cx);
        });
        open.with(cx, |editor, window, cx| {
            editor
                .body
                .update(cx, |body, cx| body.unmark_text(window, cx))
        });
        open.with(cx, |editor, window, cx| {
            editor.drop_live_transcript(window, cx)
        });
        assert_eq!(open.text(cx).0, "Groceries: Tea and cake.");
    }

    /// Spoken punctuation and commands act on the live words as on the
    /// transcript, so what shows while speaking is what lands; "delete
    /// that" takes back only what was said, never the note's own text.
    // no-prop-test: drives the editor; `voice_commands` has the properties.
    #[gpui_kit::test]
    fn spoken_commands_act_live_and_in_the_transcript(cx: &mut TestAppContext) {
        let open = super::writing_tests::Open::new(cx, "Groceries");
        open.select(cx, 9..9);
        for heard in [
            "Bullet point.",
            "Bullet point. milk,",
            "Bullet point. milk, comma",
            "Bullet point. milk, comma, eggs.",
        ] {
            open.with(cx, |editor, window, cx| {
                editor.show_live_transcript(heard, window, cx)
            });
        }
        assert_eq!(open.text(cx).0, "Groceries\n- Milk, eggs.");
        open.with(cx, |editor, window, cx| {
            assert!(editor.insert_transcript("Bullet point. milk, comma, eggs.", window, cx));
        });
        assert_eq!(open.text(cx), ("Groceries\n- Milk, eggs.".into(), 23..23));
        open.keys(cx, "cmd-z");
        assert_eq!(open.text(cx).0, "Groceries");

        // Everything said taken back, live and then in the transcript: the
        // note keeps its own words.
        open.select(cx, 9..9);
        open.with(cx, |editor, window, cx| {
            editor.show_live_transcript("Bread.", window, cx)
        });
        assert_eq!(open.text(cx).0, "Groceries bread.");
        open.with(cx, |editor, window, cx| {
            editor.show_live_transcript("Bread. Delete that.", window, cx)
        });
        assert_eq!(open.text(cx).0, "Groceries");
        open.with(cx, |editor, window, cx| {
            editor.show_live_transcript("Bread. Delete that. Delete that. Jam.", window, cx)
        });
        assert_eq!(open.text(cx).0, "Groceries jam.");
        open.with(cx, |editor, window, cx| {
            assert!(editor.insert_transcript("Bread. Delete that. Delete that.", window, cx));
        });
        assert_eq!(open.text(cx).0, "Groceries");

        // With nothing shown live, the transcript's commands act alone.
        open.select(cx, 9..9);
        open.with(cx, |editor, window, cx| {
            assert!(editor.insert_transcript("Colon. Tea. New line. Cake. Undo that.", window, cx));
        });
        assert_eq!(open.text(cx).0, "Groceries: Tea.\n");
        open.keys(cx, "cmd-z");
        assert_eq!(open.text(cx).0, "Groceries");
    }

    #[test]
    fn save_states() {
        for unsaved in [false, true] {
            for pending in [false, true] {
                assert_eq!(save_state(true, pending, || unsaved), SaveState::NotSaved);
            }
            assert_eq!(save_state(false, true, || unsaved), SaveState::Saving);
        }
        assert_eq!(save_state(false, false, || true), SaveState::NotSaved);
        assert_eq!(save_state(false, false, || false), SaveState::Saved);
    }

    #[gpui_kit::test]
    fn a_locked_account_holds_editor_writes(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let (window, editor, store, id) = editor_on(cx, dir.path(), "before");
        window
            .update(cx, |_, window, cx| {
                editor.update(cx, |editor, cx| editor.open(Some(&id), window, cx))
            })
            .unwrap();
        cx.update(|cx| {
            let mut services = crate::services::Services::default();
            services.account = crate::services::AccountState::SignedInLocked {
                username: "ngoni".into(),
            };
            cx.set_global(services);
            cx.refresh_windows();
        });
        cx.run_until_parked();
        assert!(cx.read(|cx| editor.read(cx).text_read_only));

        window
            .update(cx, |_, window, cx| {
                editor.update(cx, |editor, cx| {
                    editor.edit_for_test("after", window, cx);
                    editor.flush(cx);
                    editor.toggle_pinned(cx);
                })
            })
            .unwrap();

        let saved = store.note(&id).unwrap().unwrap();
        assert_eq!(saved.body, "before");
        assert!(!saved.pinned);

        cx.update(|cx| {
            cx.global_mut::<crate::services::Services>()
                .begin_locked_replace_write_consent();
        });
        window
            .update(cx, |_, _, cx| {
                editor.update(cx, |editor, cx| editor.flush(cx))
            })
            .unwrap();
        cx.update(|cx| {
            cx.global_mut::<crate::services::Services>()
                .end_locked_replace_write_consent();
        });
        assert_eq!(store.note(&id).unwrap().unwrap().body, "after");
    }

    /// While a change made elsewhere waits for an answer the header says
    /// the typing isn't saved, and no autosave pretends otherwise; keeping
    /// the editor's text saves it.
    #[gpui_kit::test]
    fn a_waiting_conflict_reads_not_saved(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::open(dir.path().join("k.db")).unwrap());
        let book = store
            .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
            .unwrap()
            .id;
        let id = store
            .put(
                DocType::Note,
                json!({"title": "t", "body": "one", "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let editor_store = store.clone();
        let (window, editor) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(Default::default(), cx, |window, cx| {
                let library = cx.new(|cx| Library::new(editor_store.clone(), cx));
                let (links, _) = async_channel::unbounded();
                cx.new(|cx| {
                    NoteEditor::new(
                        editor_store,
                        library,
                        links,
                        &Appearance::default(),
                        window,
                        cx,
                    )
                })
            })
            .unwrap()
        });
        let state = |cx: &mut TestAppContext| cx.read(|cx| editor.read(cx).save_state(cx));
        window
            .update(cx, |_, window, cx| {
                editor.update(cx, |editor, cx| {
                    editor.open(Some(&id), window, cx);
                    editor.edit_for_test("one, typed", window, cx);
                })
            })
            .unwrap();
        assert_eq!(state(cx), SaveState::Saving);
        // Saved elsewhere before the pause ends.
        store
            .put(
                DocType::Note,
                json!({"_id": id, "body": "theirs"}),
                Put::Merge,
                Origin::Api,
            )
            .unwrap();
        window
            .update(cx, |_, window, cx| {
                let change = kanso_store::Change {
                    id: id.clone(),
                    rev: store.note(&id).unwrap().unwrap().rev.unwrap(),
                    seq: 0,
                    deleted: false,
                    origin: Origin::Api,
                };
                editor.update(cx, |editor, cx| editor.on_store_change(&change, window, cx));
            })
            .unwrap();
        assert_eq!(state(cx), SaveState::NotSaved);
        window
            .update(cx, |_, window, cx| {
                editor.update(cx, |editor, cx| {
                    editor.edit_for_test("one, typed more", window, cx)
                })
            })
            .unwrap();
        cx.executor().advance_clock(Duration::from_secs(2));
        cx.run_until_parked();
        assert_eq!(state(cx), SaveState::NotSaved);
        assert_eq!(store.note(&id).unwrap().unwrap().body, "theirs");
        editor.update(cx, |editor, cx| editor.keep_mine(cx));
        assert_eq!(state(cx), SaveState::Saved);
        assert_eq!(store.note(&id).unwrap().unwrap().body, "one, typed more");
    }

    /// A note with these fields, as the store returns one.
    fn note(title: &str, body: &str, rev: &str, tasks: u32) -> kanso_core::Note {
        serde_json::from_value(json!({
            "_id": "note:n1",
            "_rev": rev,
            "title": title,
            "body": body,
            "bookId": "book:b1",
            "tags": ["tag:t1"],
            "status": "active",
            "pinned": false,
            "createdAt": 1,
            "updatedAt": 2,
            "timestamp": 2,
            "numOfTasks": tasks,
            "numOfCheckedTasks": 0,
        }))
        .unwrap()
    }

    #[test]
    fn same_content_examples() {
        let a = note("t", "- [ ] one", "1-a", 1);
        assert!(same_content(&a, &a));
        // A sync's new revision, and task counts the editor hadn't derived.
        assert!(same_content(&a, &note("t", "- [ ] one", "2-b", 0)));
        assert!(!same_content(&a, &note("t", "- [ ] two", "1-a", 1)));
        assert!(!same_content(&a, &note("T", "- [ ] one", "1-a", 1)));
        let mut moved = a.clone();
        moved.book_id = "book:b2".into();
        assert!(!same_content(&a, &moved));
        let mut pinned = a.clone();
        pinned.pinned = true;
        assert!(!same_content(&a, &pinned));
    }

    /// Over unsaved typing, a sync that only gave the open note a new
    /// revision asks nothing: the editor takes the revision and its autosave
    /// writes on top of it. A save that meets such a revision before the
    /// editor heard of it goes on top of it too.
    #[gpui_kit::test]
    fn a_sync_revision_keeps_the_typing(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::open(dir.path().join("k.db")).unwrap());
        let book = store
            .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
            .unwrap()
            .id;
        let id = store
            .put(
                DocType::Note,
                json!({"title": "t", "body": "- [ ] one", "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let editor_store = store.clone();
        let (window, editor) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(Default::default(), cx, |window, cx| {
                let library = cx.new(|cx| Library::new(editor_store.clone(), cx));
                let (links, _) = async_channel::unbounded();
                cx.new(|cx| {
                    NoteEditor::new(
                        editor_store,
                        library,
                        links,
                        &Appearance::default(),
                        window,
                        cx,
                    )
                })
            })
            .unwrap()
        });
        let state = |cx: &mut TestAppContext| cx.read(|cx| editor.read(cx).save_state(cx));
        let type_text = |cx: &mut TestAppContext, text: &str| {
            window
                .update(cx, |_, window, cx| {
                    editor.update(cx, |editor, cx| editor.edit_for_test(text, window, cx))
                })
                .unwrap();
        };
        window
            .update(cx, |_, window, cx| {
                editor.update(cx, |editor, cx| editor.open(Some(&id), window, cx))
            })
            .unwrap();
        type_text(cx, "- [ ] one\n- [x] two");
        assert_eq!(state(cx), SaveState::Saving);

        // The sync's new revision arrives before the pause ends.
        let rev = store.bump_revision(&id, None).unwrap().unwrap();
        window
            .update(cx, |_, window, cx| {
                let change = kanso_store::Change {
                    id: id.clone(),
                    rev: rev.clone(),
                    seq: 0,
                    deleted: false,
                    origin: Origin::Sync,
                };
                editor.update(cx, |editor, cx| editor.on_store_change(&change, window, cx));
            })
            .unwrap();
        assert!(!cx.read(|cx| editor.read(cx).has_incoming()));
        assert_eq!(
            cx.read(|cx| editor.read(cx).body_for_test(cx)),
            "- [ ] one\n- [x] two"
        );
        cx.executor().advance_clock(Duration::from_secs(2));
        cx.run_until_parked();
        assert_eq!(state(cx), SaveState::Saved);
        assert_eq!(
            store.note(&id).unwrap().unwrap().body,
            "- [ ] one\n- [x] two"
        );

        // Again, but the save comes first: it goes on top of the new revision.
        type_text(cx, "- [ ] one\n- [x] two\nthree");
        store.bump_revision(&id, None).unwrap().unwrap();
        cx.executor().advance_clock(Duration::from_secs(2));
        cx.run_until_parked();
        assert!(!cx.read(|cx| editor.read(cx).has_incoming()));
        assert_eq!(state(cx), SaveState::Saved);
        assert_eq!(
            store.note(&id).unwrap().unwrap().body,
            "- [ ] one\n- [x] two\nthree"
        );
    }

    fn gen_note() -> impl Strategy<Value = kanso_core::Note> {
        (
            "[a-z ]{0,8}",
            "(- \\[[ x]\\] [a-z]{1,4}\n){0,3}[a-z ]{0,8}",
            "[0-9]-[a-f]{4}",
            prop::sample::select(vec!["book:b1", "book:b2", "trash"]),
            prop::collection::vec(prop::sample::select(vec!["tag:t1", "tag:t2"]), 0..2),
            prop::sample::select(kanso_core::NoteStatus::ALL.to_vec()),
            any::<bool>(),
            0i64..1_000,
            0u32..4,
        )
            .prop_map(
                |(title, body, rev, book, tags, status, pinned, updated, tasks)| {
                    let mut n = note(&title, &body, &rev, tasks);
                    n.book_id = book.to_owned();
                    n.tags = tags.into_iter().map(str::to_owned).collect();
                    n.status = status;
                    n.pinned = pinned;
                    n.updated_at = updated;
                    n
                },
            )
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Invariant: a new revision and other task counts (what a sync's
        /// revision bump and the store's derivation change) keep the content
        /// the same; the relation is symmetric.
        #[test]
        fn prop_revisions_and_derived_counts_dont_change_content(
            n in gen_note(), rev in "[0-9]-[a-f]{4}", tasks in 0u32..4, checked in 0u32..4,
        ) {
            let mut bumped = n.clone();
            bumped.rev = Some(rev);
            bumped.num_of_tasks = tasks;
            bumped.num_of_checked_tasks = checked;
            prop_assert!(same_content(&n, &bumped));
            prop_assert!(same_content(&bumped, &n));
        }

        /// Metamorphic: two notes have the same content exactly when every
        /// field the user can change is equal.
        #[test]
        fn prop_same_content_compares_what_the_user_changes(a in gen_note(), b in gen_note()) {
            let visible = |n: &kanso_core::Note| (
                n.title.clone(), n.body.clone(), n.book_id.clone(), n.tags.clone(),
                n.status, n.pinned, n.updated_at,
            );
            prop_assert_eq!(same_content(&a, &b), visible(&a) == visible(&b));
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]

        /// Model-based: backlinks are exactly the other live notes whose
        /// links include the note, even when one ID is a prefix of another.
        #[test]
        fn prop_backlinks_match_the_link_graph(links in prop::collection::vec((prop::collection::vec(0usize..6, 0..4), any::<bool>()), 1..6)) {
            let dir = tempfile::tempdir().unwrap();
            let store = Store::open(dir.path().join("kanso.db")).unwrap();
            let book = store.put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui).unwrap().id;
            // IDs n, nn, nnn… so shorter IDs are prefixes of longer ones.
            let ids: Vec<String> = (0..6).map(|i| format!("note:{}", "n".repeat(i + 1))).collect();
            for (ix, (targets, trashed)) in links.iter().enumerate() {
                let body: String = targets.iter().map(|t| format!("[x](inkdrop://note/{}) ", &ids[*t][5..])).collect();
                let book_id = if *trashed { "trash".to_owned() } else { book.clone() };
                store.put(DocType::Note, json!({"_id": ids[ix], "title": format!("t{ix}"), "body": body, "bookId": book_id}), Put::Upsert, Origin::Ui).unwrap();
            }
            for (target, id) in ids.iter().enumerate() {
                let mut expected: Vec<String> = links.iter().enumerate()
                    .filter(|(ix, (targets, trashed))| *ix != target && !*trashed && targets.contains(&target))
                    .map(|(ix, _)| ids[ix].clone())
                    .collect();
                expected.sort();
                let mut found: Vec<String> = backlinks(&store, id).into_iter().map(|b| b.id).collect();
                found.sort();
                prop_assert_eq!(found, expected);
            }
        }
    }
}

/// The writing aids through the real text editor: keys, paste and undo.
#[cfg(test)]
mod writing_tests {
    use super::{NoteEditor, TEST_HTML};
    use crate::config::{Appearance, EditorSettings};
    use crate::library::Library;
    use crate::preview::ImageResize;
    use crate::word_goal::{GOAL_KEY, GoalSetting};
    use gpui_kit::{
        AnyWindowHandle, AppContext as _, ClipboardItem, Entity, Image, ImageFormat, TestAppContext,
    };
    use kanso_core::DocType;
    use kanso_store::{Origin, Put, Store};
    use serde_json::json;
    use std::ops::Range;
    use std::sync::Arc;

    pub(super) struct Open {
        pub(super) window: AnyWindowHandle,
        pub(super) editor: Entity<NoteEditor>,
        pub(super) store: Arc<Store>,
        pub(super) id: String,
        _dir: tempfile::TempDir,
    }

    impl Open {
        pub(super) fn new(cx: &mut TestAppContext, body: &str) -> Open {
            let dir = tempfile::tempdir().unwrap();
            let store = Arc::new(Store::open(dir.path().join("k.db")).unwrap());
            let book = store
                .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
                .unwrap()
                .id;
            let id = store
                .put(
                    DocType::Note,
                    json!({"title": "t", "body": body, "bookId": book}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap()
                .id;
            let editor_store = store.clone();
            let (window, editor) = cx.update(|cx| {
                gpui_kit::init(cx);
                gpui_kit::open_window(Default::default(), cx, |window, cx| {
                    let library = cx.new(|cx| Library::new(editor_store.clone(), cx));
                    let (links, _) = async_channel::unbounded();
                    cx.new(|cx| {
                        NoteEditor::new(
                            editor_store,
                            library,
                            links,
                            &Appearance::default(),
                            window,
                            cx,
                        )
                    })
                })
                .unwrap()
            });
            let open = Open {
                window,
                editor,
                store,
                id: id.clone(),
                _dir: dir,
            };
            open.with(cx, move |editor, window, cx| {
                editor.open(Some(&id), window, cx);
                editor.focus_body(window, cx);
            });
            open
        }

        pub(super) fn with<R>(
            &self,
            cx: &mut TestAppContext,
            f: impl FnOnce(
                &mut NoteEditor,
                &mut gpui_kit::Window,
                &mut gpui_kit::Context<NoteEditor>,
            ) -> R,
        ) -> R {
            let editor = self.editor.clone();
            let out = cx
                .update_window(self.window, |_, window, cx| {
                    editor.update(cx, |editor, cx| f(editor, window, cx))
                })
                .unwrap();
            cx.run_until_parked();
            out
        }

        pub(super) fn select(&self, cx: &mut TestAppContext, range: Range<usize>) {
            self.with(cx, |editor, _, cx| {
                editor
                    .body
                    .update(cx, |body, cx| body.set_selected_range(range, cx))
            });
        }

        pub(super) fn text(&self, cx: &mut TestAppContext) -> (String, Range<usize>) {
            self.with(cx, |editor, _, cx| {
                let body = editor.body.read(cx);
                (body.value().to_string(), body.selected_range())
            })
        }

        pub(super) fn input(&self, cx: &mut TestAppContext, text: &str) {
            cx.simulate_input(self.window, text);
        }

        pub(super) fn keys(&self, cx: &mut TestAppContext, keys: &str) {
            cx.simulate_keystrokes(self.window, keys);
        }
    }

    /// Pairs close as they're typed, a closer Kanso put in is stepped over,
    /// Backspace takes an empty pair, a selection is wrapped, and each of
    /// these undoes in one step. Turned off, typing is plain.
    // no-prop-test: drives the editor with keys; `pairing` has the
    // properties.
    #[gpui_kit::test]
    fn pairing_types_through_the_editor(cx: &mut TestAppContext) {
        let open = Open::new(cx, "");
        open.input(cx, "(");
        assert_eq!(open.text(cx), ("()".into(), 1..1));
        open.input(cx, "a)");
        assert_eq!(open.text(cx), ("(a)".into(), 3..3));
        open.input(cx, " [");
        assert_eq!(open.text(cx), ("(a) []".into(), 5..5));
        open.keys(cx, "backspace");
        assert_eq!(open.text(cx), ("(a) ".into(), 4..4));
        open.input(cx, "**b");
        assert_eq!(open.text(cx).0, "(a) **b**");
        open.input(cx, "**");
        assert_eq!(open.text(cx), ("(a) **b**".into(), 9..9));

        // A selection is wrapped, and stays selected; undo unwraps it.
        open.select(cx, 1..2);
        open.input(cx, "`");
        assert_eq!(open.text(cx), ("(`a`) **b**".into(), 2..3));
        open.keys(cx, "cmd-z");
        assert_eq!(open.text(cx).0, "(a) **b**");

        // Off: typed as it is.
        open.with(cx, |editor, _, cx| {
            editor.set_settings(
                EditorSettings {
                    auto_pair: false,
                    ..EditorSettings::default()
                },
                cx,
            )
        });
        open.select(cx, 9..9);
        open.input(cx, " (");
        assert_eq!(open.text(cx).0, "(a) **b** (");
    }

    /// Tab moves list items in and out, and through a table's cells,
    /// lining the table up; Enter at the end of a row adds one.
    // no-prop-test: drives the editor with keys; `kanso_engine::edit` has
    // the properties.
    #[gpui_kit::test]
    fn tab_and_enter_in_lists_and_tables(cx: &mut TestAppContext) {
        let open = Open::new(cx, "- a\n- b");
        open.select(cx, 7..7);
        open.keys(cx, "tab");
        assert_eq!(open.text(cx), ("- a\n  - b".into(), 9..9));
        open.keys(cx, "shift-tab");
        assert_eq!(open.text(cx), ("- a\n- b".into(), 7..7));
        // Undo takes the indent back in one step.
        open.keys(cx, "tab");
        open.keys(cx, "cmd-z");
        assert_eq!(open.text(cx).0, "- a\n- b");

        let table = "| Name | Qty |\n|---|---|\n| Apples | 3 |";
        open.with(cx, |editor, window, cx| {
            editor.edit_for_test(table, window, cx)
        });
        open.select(cx, 3..3);
        open.keys(cx, "tab");
        let (text, selection) = open.text(cx);
        assert_eq!(text, "| Name   | Qty |\n| ------ | --- |\n| Apples | 3   |");
        assert_eq!(&text[selection], "Qty");
        // The last cell: Enter at the end of the row adds one.
        let end = text.len();
        open.select(cx, end..end);
        open.keys(cx, "enter");
        let (text, selection) = open.text(cx);
        assert!(
            text.starts_with("| Name   | Qty |\n| ------ | --- |\n| Apples | 3   |\n|"),
            "{text:?}"
        );
        assert!(selection.start > end);

        // Format Table, from the palette or the menu.
        open.with(cx, |editor, window, cx| {
            editor.edit_for_test("| a | bb |\n|-|:-:|\n| ccc | d |", window, cx)
        });
        open.select(cx, 2..2);
        open.with(cx, |editor, window, cx| editor.format_table(window, cx));
        assert_eq!(
            open.text(cx).0,
            "| a   | bb  |\n| --- | :-: |\n| ccc |  d  |"
        );
    }

    /// A link pasted over words links them; formatted HTML pastes as
    /// Markdown; ⇧⌘V pastes it plain; an image becomes an attachment.
    // no-prop-test: drives the editor's paste; `smart_paste` has the
    // properties.
    #[gpui_kit::test]
    fn pasting_links_and_images(cx: &mut TestAppContext) {
        let open = Open::new(cx, "read the docs");
        cx.write_to_clipboard(ClipboardItem::new_string("https://kanso.app".into()));
        open.select(cx, 9..13);
        open.keys(cx, "cmd-v");
        assert_eq!(open.text(cx).0, "read the [docs](https://kanso.app)");

        let end = open.text(cx).0.len();
        TEST_HTML.with(|html| *html.borrow_mut() = Some("<p><b>Bold</b> move</p>".into()));
        cx.write_to_clipboard(ClipboardItem::new_string("Bold move".into()));
        open.select(cx, end..end);
        open.keys(cx, "cmd-v");
        assert_eq!(
            open.text(cx).0,
            "read the [docs](https://kanso.app)**Bold** move"
        );
        open.with(cx, |editor, window, cx| editor.paste_plain(window, cx));
        assert!(open.text(cx).0.ends_with("**Bold** moveBold move"));
        TEST_HTML.with(|html| *html.borrow_mut() = None);

        // An image on its own: stored, and put in as a paragraph.
        let mut png = std::io::Cursor::new(Vec::new());
        image::RgbaImage::from_pixel(2, 2, image::Rgba([9, 8, 7, 255]))
            .write_to(&mut png, image::ImageFormat::Png)
            .unwrap();
        let png = png.into_inner();
        cx.write_to_clipboard(ClipboardItem::new_image(&Image::from_bytes(
            ImageFormat::Png,
            png.clone(),
        )));
        open.keys(cx, "cmd-v");
        let text = open.text(cx).0;
        let (_, tail) = text.rsplit_once("\n\n![](inkdrop://").expect(&text);
        let id = tail.strip_suffix(')').unwrap();
        let (content_type, blob) = open.store.attachment(id).unwrap().unwrap();
        assert_eq!((content_type.as_str(), blob.data), ("image/png", png));
    }

    /// A size picked in the preview rewrites the image's Markdown as one
    /// undo step, and the cursor stays put.
    // no-prop-test: glue; `smart_paste::sized_image` has the properties.
    #[gpui_kit::test]
    fn image_sizes_rewrite_the_markdown(cx: &mut TestAppContext) {
        let source = "![](inkdrop://file:abc)";
        let open = Open::new(cx, &format!("Intro\n\n{source}\n\nEnd"));
        let end = open.text(cx).0.len();
        open.select(cx, end..end);
        let resize = |width| ImageResize {
            source: source.into(),
            src: "inkdrop://file:abc".into(),
            alt: String::new(),
            width,
        };
        open.with(cx, |editor, window, cx| {
            editor.resize_image(resize(Some(240)), window, cx)
        });
        let (text, selection) = open.text(cx);
        assert_eq!(
            text,
            "Intro\n\n<img src=\"inkdrop://file:abc\" width=\"240\">\n\nEnd"
        );
        assert_eq!(selection, text.len()..text.len());
        open.keys(cx, "cmd-z");
        assert_eq!(open.text(cx).0, format!("Intro\n\n{source}\n\nEnd"));
    }

    /// A drawing brought back from Excalidraw replaces its block in one
    /// undo step; the cursor before it stays put.
    // no-prop-test: glue; `scene_block` has the round-trip property.
    #[gpui_kit::test]
    fn drawings_brought_back_replace_their_block(cx: &mut TestAppContext) {
        let block = "```excalidraw\n{\"elements\":[]}\n```";
        let open = Open::new(cx, &format!("Intro\n\n{block}\n\nEnd"));
        open.select(cx, 2..2);
        let scene = r#"{"type":"excalidraw/clipboard","elements":[{"type":"ellipse","x":0,"y":0,"width":9,"height":9}]}"#;
        let replacement = crate::preview_rich::scene_block(scene).unwrap();
        open.with(cx, |editor, window, cx| {
            editor.replace_block(block, replacement.clone(), window, cx)
        });
        let (text, selection) = open.text(cx);
        assert_eq!(text, format!("Intro\n\n{replacement}\n\nEnd"));
        assert_eq!(selection, 2..2);
        open.keys(cx, "cmd-z");
        assert_eq!(open.text(cx).0, format!("Intro\n\n{block}\n\nEnd"));
    }

    /// An image written `![alt|300](src)` keeps that form when the preview
    /// resizes it, as one undo step.
    // no-prop-test: glue; `image_size` has the properties.
    #[gpui_kit::test]
    fn bar_widths_stay_bars(cx: &mut TestAppContext) {
        let source = "![Cat|300](inkdrop://file:abc)";
        let open = Open::new(cx, &format!("Intro\n\n{source}\n\nEnd"));
        open.with(cx, |editor, window, cx| {
            editor.resize_image(
                ImageResize {
                    source: source.into(),
                    src: "inkdrop://file:abc".into(),
                    alt: "Cat".into(),
                    width: Some(412),
                },
                window,
                cx,
            )
        });
        assert_eq!(
            open.text(cx).0,
            "Intro\n\n![Cat|412](inkdrop://file:abc)\n\nEnd"
        );
        open.keys(cx, "cmd-z");
        assert_eq!(open.text(cx).0, format!("Intro\n\n{source}\n\nEnd"));
    }

    /// A note for ticking tasks in the preview. The task list counts the
    /// YAML line as its first task, though the preview draws no box there,
    /// and skips the fence's line: the three boxes drawn are the list's
    /// second to fourth, which a click must not mix up.
    const TICK_NOTE: &str = "---\ntags:\n  - [ ] yaml\n---\n\n- [ ] one\n  - [x] two\n\n```\n- [ ] code\n```\n\n- [ ] three\n";

    /// Draws the open note's window, so the preview's boxes have bounds,
    /// after the preview has caught up with the text.
    fn draw_window(open: &Open, cx: &mut TestAppContext) {
        for _ in 0..4 {
            cx.run_until_parked();
            cx.executor()
                .advance_clock(std::time::Duration::from_millis(200));
            cx.update_window(open.window, |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .unwrap();
        }
        cx.run_until_parked();
    }

    /// Shows the note beside its source, drawn.
    fn show_preview(open: &Open, cx: &mut TestAppContext) {
        open.with(cx, |editor, _, cx| {
            editor.set_view_mode(crate::config::ViewMode::Split, cx)
        });
        draw_window(open, cx);
    }

    /// Clicks the box of the task item that starts with `item` (the note is
    /// one chunk, so a box's place in the chunk is its place in the note).
    /// Whether that box was drawn to click.
    fn click_task_box(open: &Open, cx: &mut TestAppContext, item: &str) -> bool {
        let at = open.text(cx).0.find(item).unwrap();
        let selector: &'static str = Box::leak(format!("task-box-{at}").into_boxed_str());
        let mut vcx = gpui_kit::VisualTestContext::from_window(open.window, cx);
        let Some(bounds) = vcx.debug_bounds(selector) else {
            return false;
        };
        vcx.simulate_click(bounds.center(), gpui_kit::Modifiers::none());
        draw_window(open, cx);
        true
    }

    /// A click on a task's box in the preview toggles that task in the
    /// note's Markdown, as one undo step, saved like typing, with the task
    /// counts and the preview following. A task in a fence, in front matter
    /// or in indented code is no box, and the boxes after them are the
    /// ones clicked.
    // no-prop-test: drives the preview through clicks; `preview_tasks` has
    // the properties.
    #[gpui_kit::test]
    fn ticking_a_task_in_the_preview_rewrites_the_markdown(cx: &mut TestAppContext) {
        let open = Open::new(cx, TICK_NOTE);
        let end = TICK_NOTE.len();
        open.select(cx, end..end);
        show_preview(&open, cx);
        assert_eq!(
            open.store
                .note(&open.id)
                .unwrap()
                .unwrap()
                .num_of_checked_tasks,
            1
        );

        assert!(click_task_box(&open, cx, "- [ ] three"));
        let ticked = TICK_NOTE.replace("- [ ] three", "- [x] three");
        let (text, selection) = open.text(cx);
        assert_eq!(text, ticked);
        assert_eq!(selection, end..end);
        // The preview shows the new text, and the save carries the counts.
        assert_eq!(
            open.with(cx, |editor, _, cx| editor
                .preview
                .read(cx)
                .shown()
                .to_string()),
            ticked
        );
        open.with(cx, |editor, _, cx| editor.flush(cx));
        let saved = open.store.note(&open.id).unwrap().unwrap();
        assert_eq!(
            (saved.body.as_str(), saved.num_of_checked_tasks),
            (ticked.as_str(), 2)
        );

        // One undo step puts it back.
        open.with(cx, |editor, window, cx| editor.focus_body(window, cx));
        open.keys(cx, "cmd-z");
        assert_eq!(open.text(cx).0, TICK_NOTE);

        // A ticked box clears, and a nested one is its own task.
        assert!(click_task_box(&open, cx, "- [x] two"));
        assert_eq!(open.text(cx).0, TICK_NOTE.replace("- [x] two", "- [ ] two"));
        assert!(click_task_box(&open, cx, "- [ ] two"));
        assert_eq!(open.text(cx).0, TICK_NOTE);
        // The fence's line has no box at all.
        assert!(!click_task_box(&open, cx, "- [ ] code"));
        assert_eq!(open.text(cx).0, TICK_NOTE);
    }

    /// Nothing is written when the editor takes no typing.
    fn assert_preview_leaves_note_alone(open: &Open, cx: &mut TestAppContext) {
        let before = open.store.note(&open.id).unwrap().unwrap();
        assert!(click_task_box(open, cx, "- [ ] three"));
        assert_eq!(open.text(cx).0, TICK_NOTE);
        open.with(cx, |editor, _, cx| editor.flush(cx));
        assert_eq!(open.store.note(&open.id).unwrap().unwrap(), before);
    }

    /// A note shared to view only, and one held by a locked account, can't
    /// be changed from the preview.
    // no-prop-test: drives the preview through clicks.
    #[gpui_kit::test]
    fn a_read_only_note_is_not_ticked_in_the_preview(cx: &mut TestAppContext) {
        let open = Open::new(cx, TICK_NOTE);
        show_preview(&open, cx);
        open.with(cx, |editor, _, cx| {
            editor.collab_read_only = true;
            editor.refresh_text_read_only(cx);
        });
        assert_preview_leaves_note_alone(&open, cx);
        // Editable again: the same click now writes.
        open.with(cx, |editor, _, cx| {
            editor.collab_read_only = false;
            editor.refresh_text_read_only(cx);
        });
        draw_window(&open, cx);
        assert!(click_task_box(&open, cx, "- [ ] three"));
        assert_ne!(open.text(cx).0, TICK_NOTE);
    }

    #[gpui_kit::test]
    fn a_locked_account_does_not_tick_in_the_preview(cx: &mut TestAppContext) {
        let open = Open::new(cx, TICK_NOTE);
        show_preview(&open, cx);
        cx.update(|cx| {
            let mut services = crate::services::Services::default();
            services.account = crate::services::AccountState::SignedInLocked {
                username: "ngoni".into(),
            };
            cx.set_global(services);
            cx.refresh_windows();
        });
        draw_window(&open, cx);
        assert!(cx.read(|cx| open.editor.read(cx).text_read_only));
        assert_preview_leaves_note_alone(&open, cx);
    }

    /// A trashed note is not written from the preview.
    // no-prop-test: drives the preview through clicks.
    #[gpui_kit::test]
    fn a_trashed_note_is_not_ticked_in_the_preview(cx: &mut TestAppContext) {
        let open = Open::new(cx, TICK_NOTE);
        show_preview(&open, cx);
        open.with(cx, |editor, _, _| {
            editor.note.as_mut().unwrap().book_id = kanso_core::model::TRASH_BOOK_ID.into();
        });
        assert_preview_leaves_note_alone(&open, cx);
    }

    /// A note waiting on a change made elsewhere is not written from the
    /// preview: the editor's text stays as it was typed.
    #[gpui_kit::test]
    fn a_conflicted_note_is_not_ticked_in_the_preview(cx: &mut TestAppContext) {
        let open = Open::new(cx, TICK_NOTE);
        show_preview(&open, cx);
        let typed = format!("{TICK_NOTE}typed\n");
        open.with(cx, |editor, window, cx| {
            editor.edit_for_test(&typed, window, cx)
        });
        // Saved elsewhere while the editor holds a change of its own.
        open.store
            .put(
                DocType::Note,
                json!({"_id": open.id, "body": "theirs"}),
                Put::Merge,
                Origin::Api,
            )
            .unwrap();
        let rev = open.store.note(&open.id).unwrap().unwrap().rev.unwrap();
        let id = open.id.clone();
        open.with(cx, |editor, window, cx| {
            let change = kanso_store::Change {
                id,
                rev,
                seq: 0,
                deleted: false,
                origin: Origin::Api,
            };
            editor.on_store_change(&change, window, cx)
        });
        assert!(open.with(cx, |editor, _, _| editor.has_incoming()));
        draw_window(&open, cx);
        assert!(click_task_box(&open, cx, "- [ ] three"));
        assert_eq!(open.text(cx).0, typed);
    }

    /// A note locked shut is never written from the preview, whatever
    /// reaches the editor.
    #[gpui_kit::test]
    fn a_locked_note_is_not_ticked(cx: &mut TestAppContext) {
        use crate::preview_tasks::tick_at;
        let open = Open::new(cx, TICK_NOTE);
        let tick = tick_at(TICK_NOTE, TICK_NOTE.find("- [ ] three").unwrap()).unwrap();
        open.with(cx, |editor, window, cx| {
            editor.lock = crate::locked::LockView::Closed(None);
            editor.tick_task(&tick, window, cx);
        });
        assert_eq!(open.text(cx).0, TICK_NOTE);
        open.with(cx, |editor, window, cx| {
            editor.lock = crate::locked::LockView::Plain;
            editor.tick_task(&tick, window, cx);
        });
        assert_eq!(
            open.text(cx).0,
            TICK_NOTE.replace("- [ ] three", "- [x] three")
        );
    }

    /// A tick that no longer matches the note (typed above it since the
    /// click was read) writes nothing.
    #[gpui_kit::test]
    fn a_tick_for_a_task_that_moved_writes_nothing(cx: &mut TestAppContext) {
        use crate::preview_tasks::tick_at;
        let open = Open::new(cx, TICK_NOTE);
        let tick = tick_at(TICK_NOTE, TICK_NOTE.find("- [ ] three").unwrap()).unwrap();
        let moved =
            format!("{TICK_NOTE}- [ ] four\n").replace("- [ ] one", "- [ ] zero\n- [ ] one");
        open.with(cx, |editor, window, cx| {
            editor.edit_for_test(&moved, window, cx)
        });
        open.with(cx, |editor, window, cx| editor.tick_task(&tick, window, cx));
        assert_eq!(open.text(cx).0, moved);
    }

    /// "Resize image…" gives the image on the cursor's line a width, as one
    /// undo step; on any other line it says where to put the cursor.
    // no-prop-test: glue; `image_size::resize_at` has the tests.
    #[gpui_kit::test]
    fn resize_image_from_the_palette(cx: &mut TestAppContext) {
        let text = "Intro\n\n![](https://example.com/a.png)\n\nEnd";
        let open = Open::new(cx, text);
        let on_image = text.find("https").unwrap();
        open.select(cx, on_image..on_image);
        assert!(open.with(cx, |editor, _, cx| editor.image_on_caret_line(cx).is_some()));
        open.with(cx, |editor, window, cx| {
            editor.resize_caret_image(Some(320), window, cx)
        });
        assert_eq!(
            open.text(cx).0,
            "Intro\n\n<img src=\"https://example.com/a.png\" width=\"320\">\n\nEnd"
        );
        open.keys(cx, "cmd-z");
        assert_eq!(open.text(cx).0, text);
        open.select(cx, 0..0);
        assert!(open.with(cx, |editor, _, cx| editor.image_on_caret_line(cx).is_none()));
        open.with(cx, |editor, window, cx| {
            editor.resize_caret_image(Some(320), window, cx)
        });
        assert_eq!(open.text(cx).0, text);
    }

    /// "New drawing" puts an empty drawing on lines of its own at the
    /// cursor, as one undo step.
    // no-prop-test: glue; `image_size::insert_block` has the properties.
    #[gpui_kit::test]
    fn new_drawing_goes_in_at_the_cursor(cx: &mut TestAppContext) {
        let open = Open::new(cx, "Intro\n\nEnd");
        open.select(cx, 2..2);
        open.with(cx, |editor, window, cx| editor.insert_drawing(window, cx));
        let drawing = crate::image_size::EMPTY_DRAWING;
        assert_eq!(open.text(cx).0, format!("Intro\n\n{drawing}\n\nEnd"));
        open.keys(cx, "cmd-z");
        assert_eq!(open.text(cx).0, "Intro\n\nEnd");
    }

    /// A note's own word goal is saved with it; without one the default
    /// from Settings applies, and the footer shows whichever.
    // no-prop-test: glue; `word_goal` has the properties.
    #[gpui_kit::test]
    fn word_goals_are_kept_with_the_note(cx: &mut TestAppContext) {
        let open = Open::new(cx, "one two three");
        open.with(cx, |editor, _, cx| {
            editor.set_settings(
                EditorSettings {
                    word_goal: Some(1000),
                    ..EditorSettings::default()
                },
                cx,
            )
        });
        let footer_goal =
            |cx: &mut TestAppContext| open.with(cx, |editor, _, cx| editor.footer.read(cx).goal());
        assert_eq!(footer_goal(cx), Some(1000));
        open.with(cx, |editor, _, cx| {
            editor.set_word_goal(GoalSetting::Words(500), cx)
        });
        assert_eq!(footer_goal(cx), Some(500));
        let note = open.store.note(&open.id).unwrap().unwrap();
        assert_eq!(note.extra.get(GOAL_KEY), Some(&json!(500)));
        open.with(cx, |editor, _, cx| {
            editor.set_word_goal(GoalSetting::Off, cx)
        });
        assert_eq!(footer_goal(cx), None);
        open.with(cx, |editor, _, cx| {
            editor.set_word_goal(GoalSetting::Default, cx)
        });
        assert_eq!(footer_goal(cx), Some(1000));
        let note = open.store.note(&open.id).unwrap().unwrap();
        assert!(!note.extra.contains_key(GOAL_KEY));
    }
}
