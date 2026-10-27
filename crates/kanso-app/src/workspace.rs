//! The main window: sidebar, note list and editor, plus the operations that
//! need more than one of them (dialogs, moving notes, deleting notebooks).

use kanso_engine::list::{FALLBACK_BOOK_NAME, Narrowing, NewNoteTarget, new_note_target};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::base::{GlobalState, resize_handle};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::{ActiveTheme as _, WindowExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_core::{DocType, TagColor};
use kanso_store::{Origin, Put, Store};
use serde_json::{Value, json};

use crate::account_page::SignedOutForm;
use crate::commands::Picker;
use crate::config::{AppConfig, InspectorTab, ViewMode};
use crate::dialogs::{confirm, prompt_text};
use crate::editor::inline_glue::InlineAssist;
use crate::editor::{EditorEvent, NoteEditor};
use crate::inspector::{Inspector, InspectorEvent};
use crate::layout::{
    BandAction, BandDrag, PaneEdge, PaneWidths, inset_after, inspector_dragged, inspector_fitted,
    rail_width,
};
use crate::library::{Filter, Library, LibraryEvent};
use crate::note_batch::Change as BatchChange;
use crate::note_list::{NoteList, NoteListEvent};
use crate::note_menu::NoteCommand;
use crate::palette::{Mode, Palette};
use crate::paths;
use crate::settings::{SettingsPage, SettingsReturn, SettingsView};
use crate::sidebar::{Sidebar, SidebarEvent};
use crate::tab_bar::{TabBar, TabBarEvent};
use crate::tabs::{TabMeta, Tabs};
use crate::theme::Colors;
use crate::zoom::px;

actions!(
    kanso,
    [
        NewNote,
        NewNotebook,
        FocusSearch,
        SearchAllNotes,
        FocusNoteList,
        FocusEditor,
        ToggleSidebar,
        ShowEditor,
        ShowSplit,
        ShowPreview,
        TogglePinned,
        TrashNote,
        SaveNote,
        NextNote,
        PreviousNote,
        ShowHistory,
        Dictate,
        OpenSettings,
        ExportNote,
        ExportHtml,
        ExportPdf,
        ExportLibrary,
        SyncNow,
        ShowSignIn,
        ShowCreateAccount,
        ChangePassword,
        NewRecoveryKey,
        SignOut,
        ImportPouchLibrary,
        ImportLibrary,
        OpenFile,
        Quit,
        OpenCommandPalette,
        OpenNotePicker,
        PastePlainText,
        FormatTable,
        ToggleInspector,
        ShowOutline,
        ShowBacklinks,
        AskYourNotes,
        SummarizeNote,
    ]
);

// Focus mode: ⌘⇧D in and out, and Escape out once nothing inside wants it.
actions!(kanso, [ToggleFocusMode, ExitFocusMode]);

// An empty Excalidraw drawing at the cursor.
actions!(kanso, [NewDrawing]);

// Back and forward between the notes a window opened (`nav_history`), and
// the open note's link to paste elsewhere (`note_link`).
actions!(kanso, [GoBack, GoForward, CopyNoteLink, CopyKansoLink]);

/// Process-wide state: the store, the data folder and the settings.
pub struct AppState {
    pub store: Arc<Store>,
    pub data_dir: PathBuf,
    pub config: AppConfig,
}

impl Global for AppState {}

/// How long settings that change often wait before they are written.
const CONFIG_SAVE_DELAY: Duration = Duration::from_secs(2);

/// A settings write waiting for [`CONFIG_SAVE_DELAY`].
#[derive(Default)]
struct PendingConfigSave(Option<Task<()>>);

impl Global for PendingConfigSave {}

impl AppState {
    /// Changes the settings and writes them to disk.
    pub fn update_config(cx: &mut App, change: impl FnOnce(&mut AppConfig)) {
        // This write includes anything still waiting.
        cx.default_global::<PendingConfigSave>().0 = None;
        let state = cx.global_mut::<AppState>();
        change(&mut state.config);
        let path = paths::config_path(&state.data_dir);
        if let Err(err) = state.config.save(&path) {
            eprintln!("kanso: couldn't save settings: {err:#}");
        }
    }

    /// Changes the settings now and writes them once they stop changing for
    /// a moment, for things that change all the time, like the selected note.
    pub fn update_config_soon(cx: &mut App, change: impl FnOnce(&mut AppConfig)) {
        change(&mut cx.global_mut::<AppState>().config);
        let save = cx.spawn(async move |cx| {
            cx.background_executor().timer(CONFIG_SAVE_DELAY).await;
            cx.update(AppState::save_pending_config);
        });
        cx.default_global::<PendingConfigSave>().0 = Some(save);
    }

    /// Writes a waiting settings change now, as when quitting.
    pub fn save_pending_config(cx: &mut App) {
        if cx.default_global::<PendingConfigSave>().0.take().is_some() {
            AppState::update_config(cx, |_| {});
        }
    }
}

pub fn bind_keys(cx: &mut App) {
    // `secondary` is ⌘ on macOS and Ctrl on Windows and Linux.
    cx.bind_keys([
        KeyBinding::new("secondary-n", NewNote, None),
        KeyBinding::new("secondary-shift-n", NewNotebook, None),
        // ⌘L searches the list as it is; ⇧⌘F, global search, all notes.
        KeyBinding::new("secondary-l", FocusSearch, None),
        KeyBinding::new("secondary-shift-f", SearchAllNotes, None),
        // ⌘L goes to the list's search box; ⇧⌘L to the list itself.
        KeyBinding::new("secondary-shift-l", FocusNoteList, None),
        KeyBinding::new("secondary-\\", ToggleSidebar, None),
        KeyBinding::new("secondary-alt-i", ToggleInspector, None),
        KeyBinding::new("secondary-shift-d", ToggleFocusMode, None),
        // After anything inside that takes Escape (an open menu, extra
        // cursors, the palette), so the first Escape left over leaves.
        KeyBinding::new("escape", ExitFocusMode, Some("FocusMode")),
        KeyBinding::new("secondary-1", ShowEditor, None),
        KeyBinding::new("secondary-2", ShowSplit, None),
        KeyBinding::new("secondary-3", ShowPreview, None),
        KeyBinding::new("secondary-s", SaveNote, None),
        KeyBinding::new("secondary-shift-p", TogglePinned, None),
        KeyBinding::new("secondary-shift-a", AskYourNotes, None),
        KeyBinding::new("secondary-shift-m", Dictate, None),
        KeyBinding::new("secondary-alt-down", NextNote, None),
        KeyBinding::new("secondary-alt-up", PreviousNote, None),
        KeyBinding::new("secondary-,", OpenSettings, None),
        KeyBinding::new("secondary-q", Quit, None),
        KeyBinding::new("enter", FocusEditor, Some("NoteList")),
        KeyBinding::new("secondary-k", OpenCommandPalette, None),
        KeyBinding::new("secondary-p", OpenNotePicker, None),
        // In the note's text: the clipboard's plain text, as it is.
        KeyBinding::new("secondary-shift-v", PastePlainText, Some("Input")),
    ]);
    crate::nav_history::bind_keys(cx);
    crate::palette::bind_keys(cx);
    crate::journal::bind_keys(cx);
    crate::calendar_view::bind_keys(cx);
    crate::tasks_view::bind_keys(cx);
    crate::replace::bind_keys(cx);
    crate::settings::bind_keys(cx);
    crate::tabs::bind_keys(cx);
    crate::zoom::bind_keys(cx);
    crate::editor::format_keys::bind_format_keys(cx);
}

/// The Format submenu of the Note menu: the formatting shortcuts, which act
/// on the note's text while it has the keyboard.
pub(crate) fn format_menu() -> Vec<MenuItem> {
    use crate::editor::format_keys::*;
    vec![
        MenuItem::action("Bold", FormatBold),
        MenuItem::action("Italic", FormatItalic),
        MenuItem::action("Strikethrough", FormatStrikethrough),
        MenuItem::action("Code", FormatCode),
        MenuItem::action("Link", FormatLink),
        MenuItem::separator(),
        MenuItem::action("Heading 1", FormatHeading1),
        MenuItem::action("Heading 2", FormatHeading2),
        MenuItem::action("Heading 3", FormatHeading3),
        MenuItem::action("Heading 4", FormatHeading4),
        MenuItem::action("Heading 5", FormatHeading5),
        MenuItem::action("Heading 6", FormatHeading6),
        MenuItem::separator(),
        MenuItem::action("Task", FormatTask),
        MenuItem::action("Bulleted List", FormatBulleted),
        MenuItem::action("Numbered List", FormatNumbered),
        MenuItem::action("Quote", FormatQuote),
        MenuItem::action("Code Block", FormatCodeBlock),
    ]
}

/// The View menu. Ask Your Notes is there only while AI is on: until then
/// nothing about it shows.
fn view_menu(cx: &App) -> Vec<MenuItem> {
    let ai = cx
        .try_global::<AppState>()
        .is_some_and(|state| state.config.ai.enabled);
    let mut items = vec![
        MenuItem::action("Editor Only", ShowEditor),
        MenuItem::action("Editor and Preview", ShowSplit),
        MenuItem::action("Preview Only", ShowPreview),
        MenuItem::separator(),
        MenuItem::action("Toggle Sidebar", ToggleSidebar),
        MenuItem::action("Outline and Backlinks", ToggleInspector),
    ];
    if ai {
        items.push(MenuItem::action("Ask Your Notes…", AskYourNotes));
    }
    items.extend([
        MenuItem::action("Graph", crate::graph_view::ShowGraph),
        MenuItem::action("Board", crate::board_view::ShowBoard),
        MenuItem::action("Tasks Board", crate::board_view::ShowTaskBoard),
        MenuItem::action("Focus Mode", ToggleFocusMode),
        MenuItem::action("Search All Notes", SearchAllNotes),
        MenuItem::action("Replace in All Notes…", crate::replace::ReplaceInNotes),
        MenuItem::action("Search This List", FocusSearch),
        MenuItem::action("Go to Note List", FocusNoteList),
        MenuItem::separator(),
        MenuItem::action("Zoom In", crate::zoom::ZoomIn),
        MenuItem::action("Zoom Out", crate::zoom::ZoomOut),
        MenuItem::action("Actual Size", crate::zoom::ActualSize),
    ]);
    items
}

pub fn set_menus(cx: &mut App) {
    cx.set_menus([
        Menu::new("Kanso").items([
            MenuItem::action("Settings…", OpenSettings),
            MenuItem::action("Check for Updates…", crate::updates::CheckForUpdates),
            MenuItem::separator(),
            MenuItem::action("Import…", ImportLibrary),
            MenuItem::action("Sync Now", SyncNow),
            MenuItem::separator(),
            MenuItem::action("Quit Kanso", Quit),
        ]),
        Menu::new("File").items([
            MenuItem::action("New Note", NewNote),
            MenuItem::action("New Notebook", NewNotebook),
            MenuItem::action("New from Template…", crate::journal::NewFromTemplate),
            MenuItem::action("Today's Note", crate::journal::OpenToday),
            MenuItem::action("Journal Calendar…", crate::calendar_view::ShowCalendar),
            MenuItem::action("Quick Note", crate::journal::QuickCapture),
            MenuItem::separator(),
            MenuItem::action("Open…", OpenFile),
            MenuItem::action("Import…", ImportLibrary),
            MenuItem::separator(),
            MenuItem::action("Save", SaveNote),
            MenuItem::action("Export Note…", ExportNote),
            MenuItem::action("Export as Web Page…", ExportHtml),
            MenuItem::action("Export as PDF…", ExportPdf),
            MenuItem::action("Export All Notes…", ExportLibrary),
        ]),
        Menu::new("Edit").items([
            MenuItem::action("Paste and Match Style", PastePlainText),
            MenuItem::action("Format Table", FormatTable),
            MenuItem::action("New Drawing", NewDrawing),
            MenuItem::separator(),
            MenuItem::os_submenu("Services", SystemMenuType::Services),
        ]),
        Menu::new("Note").items([
            MenuItem::action("Pin / Unpin Note", TogglePinned),
            MenuItem::action("Open in New Window", crate::note_window::OpenInNewWindow),
            MenuItem::action("Move to Trash", TrashNote),
            MenuItem::action("Show History…", ShowHistory),
            MenuItem::action("Voice Input", Dictate),
            MenuItem::action("Transcribe Recording", crate::transcribe::TranscribeAudio),
            MenuItem::action("Publish…", crate::publish::PublishNote),
            MenuItem::action("Publish Notebook…", crate::publish_site::PublishNotebook),
            MenuItem::separator(),
            MenuItem::action("Lock Note…", crate::locked::LockNote),
            MenuItem::action("Remove Lock…", crate::locked::RemoveLock),
            MenuItem::action("Lock Notes Now", crate::locked::LockNotesNow),
            MenuItem::action("Change Lock Password…", crate::locked::ChangeLockPassword),
            MenuItem::action(
                concat!("Unlock with ", crate::touch_id::unlock_name!()),
                crate::locked::TurnOnTouchId,
            ),
            MenuItem::action(
                concat!("Stop Using ", crate::touch_id::unlock_name!()),
                crate::locked::TurnOffTouchId,
            ),
            MenuItem::separator(),
            MenuItem::submenu(Menu::new("Format").items(format_menu())),
            MenuItem::separator(),
            MenuItem::action("Next Note", NextNote),
            MenuItem::action("Previous Note", PreviousNote),
            MenuItem::action("Go Back", GoBack),
            MenuItem::action("Go Forward", GoForward),
            MenuItem::separator(),
            MenuItem::action("Copy Note Link", CopyNoteLink),
            MenuItem::action("Copy Kanso Link", CopyKansoLink),
        ]),
        Menu::new("View").items(view_menu(cx)),
        Menu::new("Help").items([MenuItem::action(
            "What's New in Kanso",
            crate::updates::ShowWhatsNew,
        )]),
    ]);
    crate::title_bar::publish_menus(cx);
}

/// Runs `f` on the workspace in every main window.
pub fn for_each_workspace(
    cx: &mut App,
    mut f: impl FnMut(&mut Workspace, &mut Context<Workspace>),
) {
    for window in cx.windows() {
        window
            .update(cx, |root, _, cx| {
                let Ok(root) = root.downcast::<gpui_kit::base::Root>() else {
                    return;
                };
                let view = root.read(cx).view().clone();
                if let Ok(workspace) = view.downcast::<Workspace>() {
                    workspace.update(cx, |workspace, cx| f(workspace, cx));
                }
            })
            .ok();
    }
}

/// The first main window and its workspace, if one is open.
pub fn main_workspace(cx: &mut App) -> Option<(AnyWindowHandle, Entity<Workspace>)> {
    cx.windows().into_iter().find_map(|window| {
        let workspace = window
            .update(cx, |root, _, cx| {
                let root = root.downcast::<gpui_kit::base::Root>().ok()?;
                root.read(cx).view().clone().downcast::<Workspace>().ok()
            })
            .ok()
            .flatten()?;
        Some((window, workspace))
    })
}

/// Runs `f` on every note editor: each main window's, and each note's own
/// window's (`note_window.rs`).
pub fn for_each_editor(cx: &mut App, mut f: impl FnMut(&mut NoteEditor, &mut Context<NoteEditor>)) {
    let mut editors = Vec::new();
    for_each_workspace(cx, |workspace, _| editors.push(workspace.editor.clone()));
    editors.extend(
        crate::note_window::note_windows(cx)
            .into_iter()
            .map(|(_, view)| view.read(cx).editor.clone()),
    );
    for editor in editors {
        editor.update(cx, &mut f);
    }
}

/// Shows the themes the settings choose, in every window.
pub fn apply_theme(window: Option<&mut Window>, cx: &mut App) {
    crate::themes::apply(window, cx);
}

/// How the note about to be selected should open: in the tab showing, in a
/// new tab after it, or by showing the tab already holding it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OpenIntent {
    Current,
    NewTab,
    Activate(usize),
    /// Going back or forward: this tab takes the note, even when another
    /// tab shows it too.
    InPlace,
}

pub struct Workspace {
    pub(crate) store: Arc<Store>,
    pub(crate) library: Entity<Library>,
    pub(crate) sidebar: Entity<Sidebar>,
    pub(crate) note_list: Entity<NoteList>,
    /// The tasks across the notes, in the list's place while Tasks shows.
    pub(crate) tasks: Entity<crate::tasks_view::TasksView>,
    pub editor: Entity<NoteEditor>,
    /// The command palette, drawn over the rest when open.
    pub(crate) palette: Entity<Palette>,
    /// The outline and backlinks beside the editor.
    pub(crate) inspector: Entity<Inspector>,
    pub(crate) inspector_open: bool,
    /// The inspector's width as set; it narrows or hides to leave the
    /// editor its room.
    pub(crate) inspector_width: f32,
    /// The graph of every note, over the window while open
    /// (`graph_view.rs`); made the first time it opens.
    pub(crate) graph: Option<Entity<crate::graph_view::GraphView>>,
    pub(crate) graph_open: bool,
    /// The board over the window while open (`board_view.rs`); made the
    /// first time it opens.
    pub(crate) board: Option<Entity<crate::board_view::BoardView>>,
    pub(crate) board_open: bool,
    pub(crate) _board_events: Option<Subscription>,
    /// The open calendar sheet's picks.
    pub(crate) _calendar_events: Option<Subscription>,
    /// Settings, drawn in place of the list and the editor while open.
    pub(crate) settings: Entity<SettingsView>,
    /// While Settings are open: where the notes were, to go back to.
    pub(crate) settings_return: Option<SettingsReturn>,
    /// The full sidebar, or the icon rail it collapses to.
    pub(crate) show_sidebar: bool,
    /// Only the note, centred: no sidebar and no list.
    pub(crate) focus_mode: bool,
    /// The sidebar's and the list's widths; the editor takes the rest.
    pub(crate) pane_widths: PaneWidths,
    /// The workspace's own focus. It always has a place in the window, so
    /// the window's shortcuts work whenever nothing inside has the keyboard:
    /// at launch, or after the focused element stops being drawn.
    pub(crate) focus: FocusHandle,
    /// A press in a top band that may become a window drag.
    band_drag: BandDrag,
    /// The notes open as tabs above the editor, which one shows, and each
    /// one's own back-and-forward history and place (`tabs`).
    pub(crate) tabs: Tabs,
    /// The tab bar drawn above the editor, when two or more notes are open.
    pub(crate) tab_bar: Entity<TabBar>,
    /// How the note being selected should open.
    pending_open: OpenIntent,
    /// Notes whose tabs are kept while the editor holds unsaved text about
    /// a delete, to close them once that settles.
    pending_tab_closes: std::collections::BTreeSet<String>,
    /// [`Workspace::nav_reach`] until a visit or a change to the notes.
    pub(crate) nav_reach: Option<(bool, bool)>,
    /// Draws stand-ins for the traffic lights, which the system draws over
    /// the window and a headless snapshot can't show.
    #[cfg(test)]
    pub(crate) placeholder_traffic_lights: bool,
    /// The menus of the title bar Kanso draws where the system draws no
    /// menu bar (see `title_bar`); none on macOS.
    pub(crate) title_bar: Option<Entity<gpui_kit::component::menu::AppMenuBar>>,
    _subscriptions: Vec<Subscription>,
    _links: Task<()>,
    /// Opens the review sheet when an agent suggests an edit while this
    /// window is focused.
    _reviews: Vec<Task<()>>,
    /// Locks the notes again once they go unused for a while.
    _auto_lock: Task<()>,
}

impl Workspace {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // The sync offer counts its week from the first launch.
        if cx
            .global::<AppState>()
            .config
            .nudges
            .first_seen_ms
            .is_none()
        {
            let now = crate::clock::Clock::now_ms(cx);
            AppState::update_config(cx, |config| config.nudges.first_seen_ms = Some(now));
        }
        let state = cx.global::<AppState>();
        let store = state.store.clone();
        let appearance = state.config.appearance.clone();
        let last_note = state.config.last_note.clone();
        let saved_tabs = state.config.layout.open_tabs.clone();
        let saved_active_tab = state.config.layout.active_tab;
        let focus_mode = state.config.layout.focus_mode;
        let inspector_open = state.config.layout.inspector_open;
        let inspector_width = state.config.layout.inspector_width;
        let inspector_tab =
            if state.config.layout.inspector_tab == InspectorTab::Ask && !state.config.ai.enabled {
                InspectorTab::Outline
            } else {
                state.config.layout.inspector_tab
            };
        let graph_hops = state.config.discovery.graph_hops;

        // The tabs kept in the settings, without the notes that have gone
        // since and the ones in the trash.
        let restored: Vec<(usize, TabMeta)> = saved_tabs
            .iter()
            .enumerate()
            .filter_map(|(at, id)| {
                let note = store.note(id).ok().flatten().filter(|n| !n.is_trashed())?;
                let locked = note.is_locked();
                Some((
                    at,
                    TabMeta {
                        id: note.id,
                        title: note.title,
                        locked,
                    },
                ))
            })
            .collect();
        let restore_note = saved_tabs
            .get(saved_active_tab)
            .and_then(|id| restored.iter().find(|(_, meta)| &meta.id == id))
            .map(|(_, meta)| meta.id.clone());
        let restore_active = restore_note
            .as_ref()
            .and_then(|id| restored.iter().position(|(_, meta)| &meta.id == id))
            .unwrap_or(0);

        let library = cx.new(|cx| Library::new(store.clone(), cx));
        let sidebar = cx.new(|cx| Sidebar::new(library.clone(), cx));
        let note_list = cx.new(|cx| NoteList::new(library.clone(), window, cx));
        let (links_tx, links_rx) = async_channel::unbounded::<String>();
        let editor = cx.new(|cx| {
            NoteEditor::new(
                store.clone(),
                library.clone(),
                links_tx,
                &appearance,
                window,
                cx,
            )
        });

        let tasks =
            cx.new(|cx| crate::tasks_view::TasksView::new(library.clone(), editor.clone(), cx));
        let focus = cx.focus_handle();
        let list_focus = note_list.read(cx).focus_handle(cx);
        let palette = cx.new(|cx| {
            Palette::new(
                store.clone(),
                library.clone(),
                editor.clone(),
                focus.clone(),
                list_focus,
                window,
                cx,
            )
        });

        let settings = cx.new(|cx| SettingsView::new(library.clone(), window, cx));
        let tab_bar = cx.new(|_| TabBar::default());
        let inspector = cx.new(|cx| {
            Inspector::new(
                store.clone(),
                library.clone(),
                editor.clone(),
                inspector_tab,
                graph_hops,
                window,
                cx,
            )
        });
        editor.update(cx, |editor, cx| {
            editor.inspector_open = inspector_open;
            editor.refresh_wiki(cx);
        });

        let subscriptions = vec![
            cx.subscribe_in(&library, window, |this, _, event, window, cx| match event {
                // A place asked for: the list shows, whatever hid it. A
                // locked notebook asks for its password once.
                LibraryEvent::Shown => {
                    this.leave_settings(window, cx);
                    this.set_focus_mode(false, window, cx);
                    if let Filter::Book(id) = this.library.read(cx).filter.clone()
                        && this.library.read(cx).is_book_locked(&id)
                        && !window.has_active_dialog(cx)
                    {
                        crate::notebook_lock::open_notebook(&id, this.editor.clone(), window, cx);
                    }
                }
                // A note someone picked shows; one the app picked itself
                // (the last note at launch, the first of a new list) waits
                // behind Settings.
                LibraryEvent::Selected { id, asked } => {
                    let id = id.clone();
                    this.accept_selection(id.clone(), *asked, window, cx);
                    AppState::update_config_soon(cx, |config| config.last_note = id);
                }
                LibraryEvent::Changed(change) => {
                    this.nav_reach = None;
                    let change = change.clone();
                    let note = change.id.starts_with("note:");
                    this.editor.update(cx, |editor, cx| {
                        editor.on_store_change(&change, window, cx);
                        editor.queries_changed(cx);
                        if note {
                            editor.refresh_backlinks(cx);
                            editor.refresh_wiki(cx);
                        }
                    });
                    if note {
                        this.inspector
                            .update(cx, |inspector, cx| inspector.refresh(cx));
                        this.sync_tabs_with_store(&change, window, cx);
                    }
                }
            }),
            cx.subscribe_in(&sidebar, window, |this, _, event, window, cx| {
                this.on_sidebar_event(event, window, cx);
            }),
            cx.subscribe_in(&note_list, window, |this, _, event, window, cx| {
                this.on_note_list_event(event, window, cx);
            }),
            cx.subscribe_in(&tasks, window, |this, _, event, window, cx| {
                this.on_tasks_event(event, window, cx);
            }),
            cx.subscribe_in(&editor, window, |this, _, event, window, cx| match event {
                EditorEvent::Notice(message) => {
                    window.push_notification(Notification::error(message.clone()), cx);
                }
                EditorEvent::ShowHistory => this.show_history(window, cx),
                EditorEvent::Command(command) => {
                    let id = this.editor.read(cx).note_id().map(str::to_owned);
                    if let Some(id) = id {
                        this.run_note_command(&id, *command, window, cx);
                    }
                }
                EditorEvent::Pick(picker) => {
                    let mode = Mode::Pick(*picker);
                    this.palette
                        .update(cx, |palette, cx| palette.open(mode, window, cx));
                }
                EditorEvent::ShowBacklinks => {
                    this.show_inspector(Some(InspectorTab::Backlinks), cx);
                }
                EditorEvent::ToggleInspector => this.toggle_inspector(cx),
                EditorEvent::Closed(id) => this.close_tab_of_note(id, window, cx),
                EditorEvent::Renamed {
                    id,
                    old,
                    new,
                    links,
                    notes,
                } => this.offer_relink(id, old, new, *links, *notes, window, cx),
            }),
            cx.subscribe_in(&tab_bar, window, |this, _, event, window, cx| {
                this.on_tab_bar_event(event, window, cx);
            }),
            // A kept tab goes once the editor's text about a deleted note
            // is saved or the question answered; editor notifies are the
            // signal that either may have happened.
            cx.observe_in(&editor, window, |this, _, window, cx| {
                if this.pending_tab_closes.is_empty() {
                    return;
                }
                cx.defer_in(window, |this, window, cx| {
                    this.recheck_pending_tab_closes(window, cx)
                });
            }),
            cx.subscribe(&inspector, |_, _, event, cx| match event {
                InspectorEvent::Tab(tab) => {
                    let tab = *tab;
                    AppState::update_config_soon(cx, |config| config.layout.inspector_tab = tab);
                }
            }),
            cx.observe_window_appearance(window, |_, window, cx| {
                if cx
                    .global::<AppState>()
                    .config
                    .appearance
                    .themes
                    .follow_system
                {
                    apply_theme(Some(window), cx);
                }
            }),
            // The preview alone hides the text that had the keyboard, say:
            // the keyboard comes back here rather than going nowhere.
            cx.on_focus_lost(window, |this, window, cx| window.focus(&this.focus, cx)),
            // Approval notifications are best-effort hints. Coming back to the
            // app is one of the GET fallbacks; it never runs on a timer.
            cx.observe_window_activation(window, |_, window, cx| {
                if window.is_window_active() {
                    crate::device_approval::refresh(cx);
                }
            }),
        ];
        window.focus(&focus, cx);

        let links = cx.spawn_in(window, async move |this, cx| {
            while let Ok(id) = links_rx.recv().await {
                let opened =
                    this.update_in(cx, |this, window, cx| this.follow_link(id, window, cx));
                if opened.is_err() {
                    break;
                }
            }
        });

        let reviews = crate::review_queue::watch(window, cx);

        let auto_lock = cx.spawn_in(window, async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_secs(30))
                    .await;
                let locked = this.update_in(cx, |this, window, cx| {
                    if crate::locked::LockKeys::idle(cx, std::time::Instant::now()) {
                        crate::locked::lock_now(&this.editor, window, cx);
                    }
                });
                if locked.is_err() {
                    break;
                }
            }
        });

        let opening =
            restore_note.or_else(|| last_note.filter(|id| store.note(id).ok().flatten().is_some()));
        if let Some(id) = opening {
            library.update(cx, |library, cx| library.select_quietly(Some(id), cx));
        }
        // Focus mode kept from last time: once the note is open, the
        // keyboard goes into its text, as turning focus mode on does.
        if focus_mode {
            cx.defer_in(window, |this: &mut Workspace, window, cx| {
                if this.focus_mode && !this.settings_open() {
                    this.editor
                        .update(cx, |editor, cx| editor.focus_for_writing(window, cx));
                }
            });
        }

        Workspace {
            store,
            library,
            sidebar,
            note_list,
            tasks,
            editor,
            palette,
            inspector,
            inspector_open,
            inspector_width,
            graph: None,
            graph_open: false,
            board: None,
            board_open: false,
            _board_events: None,
            _calendar_events: None,
            settings,
            settings_return: None,
            show_sidebar: appearance.show_sidebar,
            focus_mode,
            pane_widths: PaneWidths::default(),
            focus,
            band_drag: BandDrag::default(),
            tabs: Tabs::restore(restored, restore_active),
            tab_bar,
            pending_open: OpenIntent::Current,
            pending_tab_closes: Default::default(),
            nav_reach: None,
            #[cfg(test)]
            placeholder_traffic_lights: false,
            title_bar: crate::title_bar::SHOWN
                .then(|| gpui_kit::component::menu::AppMenuBar::new(cx)),
            _subscriptions: subscriptions,
            _links: links,
            _reviews: reviews,
            _auto_lock: auto_lock,
        }
    }

    pub(crate) fn notify_error(
        &self,
        message: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.push_notification(Notification::error(message), cx);
    }

    /// Saves the open note before an operation that may touch it.
    pub(crate) fn flush(&self, cx: &mut App) {
        self.editor.update(cx, |editor, cx| editor.flush(cx));
    }

    /// Opens Markdown and text files the system handed to Kanso, as File ›
    /// Open… does, and shows the last one. Nothing makes a new note.
    pub(crate) fn open_documents(
        &mut self,
        paths: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.flush(cx);
        let library = self.library.clone();
        crate::library_import::open_paths_in(paths, window, cx, move |id, _, cx| {
            library.update(cx, |library, cx| library.select(Some(id), cx));
        });
    }

    fn new_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.flush(cx);
        let library = self.library.read(cx);
        // The engine decides where the note goes and what it starts with;
        // the phones ask it the same.
        let target = new_note_target(
            &library.filter,
            Narrowing {
                status: library.status_only,
                pinned: library.pinned_only,
            },
            None,
            &library.books,
        );
        let trash = matches!(library.filter, Filter::Trash);
        let book = match self.target_book(&target, window, cx) {
            Some(book) => book,
            None => return,
        };
        let mut note = json!({"title": "", "body": "", "bookId": book});
        let mut tags = target.tags.clone();
        // A level of the tag tree: its own tag, made if only tags inside it
        // exist so far.
        if let Some(path) = &target.tag_path {
            match self.store.tag_named(path, Origin::Ui) {
                Ok(tag) => tags.push(tag),
                Err(err) => {
                    return self.notify_error(format!("Couldn't tag the note: {err}"), window, cx);
                }
            }
        }
        if !tags.is_empty() {
            note["tags"] = json!(tags);
        }
        if let Some(status) = target.status {
            note["status"] = json!(status.as_str());
        }
        if target.pinned {
            note["pinned"] = json!(true);
        }
        if trash {
            self.library.update(cx, |library, cx| {
                library.set_filter(Filter::Book(book.clone()), cx)
            });
        }
        match self.store.put(DocType::Note, note, Put::Upsert, Origin::Ui) {
            Ok(created) => {
                self.library
                    .update(cx, |library, cx| library.select(Some(created.id), cx));
                self.editor
                    .update(cx, |editor, cx| editor.focus_title(window, cx));
            }
            Err(err) => self.notify_error(format!("Couldn't create the note: {err}"), window, cx),
        }
    }

    /// The notebook `target` names, made when the library has none.
    fn target_book(
        &mut self,
        target: &NewNoteTarget,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<String> {
        if let Some(book) = &target.book_id {
            return Some(book.clone());
        }
        match self.store.put(
            DocType::Book,
            json!({"name": FALLBACK_BOOK_NAME}),
            Put::Upsert,
            Origin::Ui,
        ) {
            Ok(created) => Some(created.id),
            Err(err) => {
                self.notify_error(format!("Couldn't create a notebook: {err}"), window, cx);
                None
            }
        }
    }

    /// Follows a link clicked in the preview: a note's ID opens the note,
    /// and an unresolved wiki-link's `kanso-new:` link creates its note.
    pub(crate) fn follow_link(
        &mut self,
        link: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(title) = crate::wikilinks::new_note_title(&link) {
            self.new_note_titled(&title, window, cx);
            return;
        }
        self.library
            .update(cx, |library, cx| library.select(Some(link), cx));
    }

    /// Creates a note titled `title` in the notebook a new note would go
    /// into, and opens it with the keyboard in its text: what clicking an
    /// unresolved wiki-link does. A note with that title already (made since
    /// the preview was drawn) opens instead.
    fn new_note_titled(&mut self, title: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.flush(cx);
        if let Ok(Some((id, _))) = self.store.resolve_title(title) {
            self.library
                .update(cx, |library, cx| library.select(Some(id), cx));
            return;
        }
        let target = {
            let library = self.library.read(cx);
            new_note_target(&library.filter, Narrowing::default(), None, &library.books)
        };
        let Some(book) = self.target_book(&target, window, cx) else {
            return;
        };
        let note = json!({"title": title, "body": "", "bookId": book});
        match self.store.put(DocType::Note, note, Put::Upsert, Origin::Ui) {
            Ok(created) => {
                self.library
                    .update(cx, |library, cx| library.select(Some(created.id), cx));
                self.editor
                    .update(cx, |editor, cx| editor.focus_body(window, cx));
            }
            Err(err) => self.notify_error(format!("Couldn't create the note: {err}"), window, cx),
        }
    }

    /// Opens the inspector, on `tab` when one is given.
    pub(crate) fn show_inspector(&mut self, tab: Option<InspectorTab>, cx: &mut Context<Self>) {
        if let Some(tab) = tab {
            self.inspector
                .update(cx, |inspector, cx| inspector.set_tab(tab, cx));
        }
        if !self.inspector_open {
            self.set_inspector_open(true, cx);
        }
    }

    pub(crate) fn toggle_inspector(&mut self, cx: &mut Context<Self>) {
        self.set_inspector_open(!self.inspector_open, cx);
    }

    fn set_inspector_open(&mut self, open: bool, cx: &mut Context<Self>) {
        if !open {
            // Closing the pane ends the Ask conversation; it is never saved.
            self.inspector
                .update(cx, |inspector, cx| inspector.stop_ask(cx));
        }
        self.inspector_open = open;
        self.editor.update(cx, |editor, cx| {
            editor.inspector_open = open;
            cx.notify();
        });
        AppState::update_config(cx, |config| config.layout.inspector_open = open);
        cx.notify();
    }

    /// After a rename, offers to point the wiki-links that used the old
    /// title at the new one.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn offer_relink(
        &mut self,
        id: &str,
        old: &str,
        new: &str,
        links: usize,
        notes: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (id, old, new) = (id.to_owned(), old.to_owned(), new.to_owned());
        let store = self.store.clone();
        let message = relink_message(&old, links, notes);
        let label = relink_label(links);
        let notification = Notification::info(message).action(move |_, _, _| {
            let (id, old, new, store) = (id.clone(), old.clone(), new.clone(), store.clone());
            gpui_kit::component::button::Button::new("update-links")
                .label(label.clone())
                .on_click(move |_, window, cx| {
                    match store.retitle_links(&old, &new, &id, Origin::Ui) {
                        Ok(_) => {}
                        Err(err) => window.push_notification(
                            Notification::error(format!("Couldn't update the links: {err}")),
                            cx,
                        ),
                    }
                })
        });
        window.push_notification(notification, cx);
    }

    /// Writes `fields` into document `id` with a shallow merge.
    fn merge(
        &self,
        doc_type: DocType,
        id: &str,
        mut fields: Value,
        window: &mut Window,
        cx: &mut App,
    ) {
        fields["_id"] = json!(id);
        if let Err(err) = self.store.put(doc_type, fields, Put::Merge, Origin::Ui) {
            self.notify_error(err.to_string(), window, cx);
        }
    }

    fn with_open_note(
        &self,
        id: &str,
        cx: &mut App,
        f: impl FnOnce(&mut NoteEditor, &mut Context<NoteEditor>),
    ) -> bool {
        let open = self.editor.read(cx).note_id() == Some(id);
        if open {
            self.editor.update(cx, f);
        }
        open
    }

    /// Opens a task's note at its line, from the Tasks view.
    fn on_tasks_event(
        &mut self,
        event: &crate::tasks_view::TasksEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        use crate::tasks_view::TasksEvent;
        match event {
            TasksEvent::Notice(message) => {
                window.push_notification(Notification::info(message.clone()), cx);
            }
            TasksEvent::Open {
                note_id,
                line,
                focus,
            } => self.open_note_at(note_id, *line, *focus, window, cx),
        }
    }

    /// Opens note `note_id` with `line` in view; `focus` puts the keyboard
    /// in its text.
    pub(crate) fn open_note_at(
        &mut self,
        note_id: &str,
        line: u32,
        focus: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let open = self.editor.read(cx).note_id() == Some(note_id);
        if open {
            self.leave_settings(window, cx);
            self.editor
                .update(cx, |editor, cx| editor.reveal_line(line, window, cx));
        } else {
            let tab = self.tabs.index_of(note_id);
            if let Some(index) = tab {
                // The line asked for wins over where the reader was.
                self.tabs.take_place(index);
            }
            self.editor.update(cx, |editor, _| {
                editor.reveal_after_open(note_id.to_owned(), line)
            });
            let intent = tab.map_or(OpenIntent::Current, OpenIntent::Activate);
            self.select_note(note_id, intent, cx);
        }
        if focus {
            self.editor
                .update(cx, |editor, cx| editor.focus_body(window, cx));
        }
    }

    fn on_note_list_event(
        &mut self,
        event: &NoteListEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            NoteListEvent::NewNote => self.new_note(window, cx),
            NoteListEvent::OpenSelected => self
                .editor
                .update(cx, |editor, cx| editor.focus_body(window, cx)),
            NoteListEvent::Command(id, command) => self.run_note_command(id, *command, window, cx),
            NoteListEvent::CommandSeveral(ids, command) => {
                self.run_several_command(ids, *command, window, cx)
            }
            NoteListEvent::Restore(id) => {
                let Some(book) = self.library.read(cx).books.first().map(|b| b.id.clone()) else {
                    return self.notify_error(
                        "Create a notebook to restore notes into.",
                        window,
                        cx,
                    );
                };
                if !self.with_open_note(id, cx, |editor, cx| editor.set_book(book.clone(), cx)) {
                    self.merge(DocType::Note, id, json!({"bookId": book}), window, cx);
                }
            }
            NoteListEvent::DeleteForever(id) => {
                let id = id.clone();
                let store = self.store.clone();
                confirm(
                    "Delete this note permanently?",
                    "It can't be restored afterwards.",
                    "Delete",
                    window,
                    cx,
                    move |window, cx| {
                        if let Err(err) = store.delete(&id, Origin::Ui) {
                            window.push_notification(Notification::error(err.to_string()), cx);
                        }
                    },
                );
            }
        }
    }

    /// The notes chosen in the list, when there are several and the list
    /// has the keyboard: from the note's text, Note menu items act on the
    /// open note alone.
    fn several_in_list(&self, window: &Window, cx: &App) -> Option<Vec<String>> {
        let list = self.note_list.focus_handle(cx).contains_focused(window, cx);
        list.then(|| self.library.read(cx).several_selected())
            .flatten()
    }

    /// Carries out a choice from the menu of several selected notes, for
    /// all of `ids`; what only makes sense for one note is for the open one.
    fn run_several_command(
        &mut self,
        ids: &[String],
        command: NoteCommand,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let change = match command {
            NoteCommand::TogglePinned => BatchChange::TogglePin,
            NoteCommand::Trash => BatchChange::Move(TRASH_BOOK_ID.into()),
            NoteCommand::SetStatus(status) => BatchChange::Status(status),
            NoteCommand::Export => return self.export(Some(ids.to_vec()), window, cx),
            NoteCommand::Duplicate
            | NoteCommand::ShowHistory
            | NoteCommand::Publish
            | NoteCommand::Share
            | NoteCommand::Lock
            | NoteCommand::RemoveLock
            | NoteCommand::CopyLink
            | NoteCommand::CopyKansoLink
            | NoteCommand::OpenInNewWindow
            | NoteCommand::OpenInNewTab => {
                if let Some(id) = self.library.read(cx).selected.clone() {
                    self.run_note_command(&id, command, window, cx);
                }
                return;
            }
        };
        self.change_notes(ids, change, window, cx);
    }

    /// Makes `change` to the notes `ids` at once, with Undo.
    pub(crate) fn change_notes(
        &mut self,
        ids: &[String],
        change: BatchChange,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        crate::note_batch::apply(ids, change, &self.library, window, cx);
    }

    /// Carries out a choice from a note's menu (see `note_menu`), for the
    /// note `id`, which may not be the open one.
    pub(crate) fn run_note_command(
        &mut self,
        id: &str,
        command: NoteCommand,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match command {
            NoteCommand::TogglePinned => {
                if !self.with_open_note(id, cx, |editor, cx| editor.toggle_pinned(cx)) {
                    let pinned = self
                        .library
                        .read(cx)
                        .notes
                        .iter()
                        .any(|n| n.id == id && n.pinned);
                    self.merge(DocType::Note, id, json!({"pinned": !pinned}), window, cx);
                }
            }
            NoteCommand::Trash => {
                if !self.with_open_note(id, cx, |editor, cx| editor.trash(cx)) {
                    self.merge(
                        DocType::Note,
                        id,
                        json!({"bookId": TRASH_BOOK_ID}),
                        window,
                        cx,
                    );
                }
            }
            NoteCommand::Duplicate => {
                self.flush(cx);
                let Ok(Some(mut note)) = self.store.note(id) else {
                    return;
                };
                // A locked note's copy is locked too, sealed for its own ID.
                if crate::locked::reveal(&mut note, cx).is_closed() {
                    self.notify_error("Unlock the note to duplicate it.", window, cx);
                    return;
                }
                let copy = json!({
                    "title": kanso_engine::notes::copy_title(&note.title),
                    "body": if note.is_locked() { "" } else { note.body.as_str() },
                    "bookId": note.book_id,
                    "tags": note.tags,
                    "status": note.status.as_str(),
                });
                let created = self
                    .store
                    .put(DocType::Note, copy, Put::Upsert, Origin::Ui)
                    .map_err(|err| err.to_string())
                    .and_then(|created| {
                        if note.is_locked() {
                            self.lock_copy(&created.id, note.body.clone(), cx)?;
                        }
                        Ok(created.id)
                    });
                match created {
                    Ok(created) => self
                        .library
                        .update(cx, |library, cx| library.select(Some(created), cx)),
                    Err(err) => self.notify_error(err, window, cx),
                }
            }
            NoteCommand::SetStatus(status) => {
                if !self.with_open_note(id, cx, |editor, cx| editor.set_status(status, cx)) {
                    self.merge(
                        DocType::Note,
                        id,
                        json!({"status": status.as_str()}),
                        window,
                        cx,
                    );
                }
            }
            NoteCommand::Export => self.export(Some(vec![id.to_owned()]), window, cx),
            NoteCommand::CopyLink => self.copy_link_of(id, false, window, cx),
            NoteCommand::CopyKansoLink => self.copy_link_of(id, true, window, cx),
            NoteCommand::OpenInNewTab => self.open_note_in_new_tab(id, window, cx),
            NoteCommand::OpenInNewWindow => self.open_in_new_window(id, window, cx),
            // These work on the open note, so the note opens first.
            NoteCommand::ShowHistory => {
                if self.open_note(id, window, cx) {
                    self.show_history(window, cx);
                }
            }
            NoteCommand::Publish => {
                if self.open_note(id, window, cx) {
                    crate::publish::open_publish(self.editor.clone(), window, cx);
                }
            }
            NoteCommand::Share => {
                if self.open_note(id, window, cx) {
                    crate::share_sheet::open_share(self.editor.clone(), window, cx);
                }
            }
            NoteCommand::Lock => {
                if self.open_note(id, window, cx) {
                    crate::locked::lock_open_note(self.editor.clone(), window, cx);
                }
            }
            NoteCommand::RemoveLock => {
                if self.open_note(id, window, cx) {
                    crate::locked::remove_lock(self.editor.clone(), window, cx);
                }
            }
        }
    }

    /// Opens note `id` in a window of its own (`note_window.rs`), beside
    /// this one. Its typing waiting here is saved first, so it opens with it.
    pub(crate) fn open_in_new_window(
        &mut self,
        id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.flush(cx);
        let id = id.to_owned();
        let bounds = window.bounds();
        let from = (
            f32::from(bounds.origin.x),
            f32::from(bounds.origin.y),
            f32::from(bounds.size.width),
            f32::from(bounds.size.height),
        );
        // The windows can't be looked through from inside this one's update.
        cx.defer(move |cx| crate::note_window::open_note_window(&id, Some(from), cx));
    }

    /// For a note's own window: opens note `id` here and the palette's
    /// list `picker` for it.
    pub(crate) fn pick_for_note(
        &mut self,
        id: &str,
        picker: Picker,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.open_note(id, window, cx) {
            self.palette.update(cx, |palette, cx| {
                palette.open(Mode::Pick(picker), window, cx)
            });
        }
    }

    /// For a note's own window: opens note `id` here with its backlinks
    /// beside it.
    pub(crate) fn show_backlinks_of(
        &mut self,
        id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.open_note(id, window, cx) {
            self.set_focus_mode(false, window, cx);
            self.show_inspector(Some(InspectorTab::Backlinks), cx);
        }
    }

    /// Seals `body` into the new note `id`, a locked note's copy.
    fn lock_copy(&self, id: &str, body: String, cx: &mut App) -> Result<(), String> {
        let mut copy = self
            .store
            .note(id)
            .map_err(|err| err.to_string())?
            .ok_or_else(|| "The copy wasn't saved.".to_owned())?;
        copy.body = body;
        crate::locked::seal(&mut copy, cx)?;
        let value = serde_json::to_value(&copy).map_err(|err| err.to_string())?;
        self.store.lock_note(value).map_err(|err| err.to_string())?;
        Ok(())
    }

    /// Selects and opens the note `id`, and says whether it is now open: a
    /// held Replace can keep the editor on its note.
    fn open_note(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) -> bool {
        self.close_settings(window, cx);
        // The library's Selected event comes after this update, so the
        // editor is opened here, as it was before there were tabs; the
        // event then only keeps the tabs in step. Without this, a command
        // on a note that isn't showing (Lock, History, Publish, Share)
        // reported "not open" and did nothing.
        self.stash_place(cx);
        self.editor
            .update(cx, |editor, cx| editor.open(Some(id), window, cx));
        self.select_note(id, OpenIntent::Current, cx);
        self.editor.read(cx).note_id() == Some(id)
    }

    fn on_sidebar_event(
        &mut self,
        event: &SidebarEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let store = self.store.clone();
        let library = self.library.clone();
        let report =
            |result: Result<(), kanso_store::StoreError>, window: &mut Window, cx: &mut App| {
                if let Err(err) = result {
                    window.push_notification(Notification::error(err.to_string()), cx);
                }
            };
        match event {
            SidebarEvent::NewNote => self.new_note(window, cx),
            SidebarEvent::GoToNotebook => {
                let mode = Mode::Pick(Picker::GoToNotebook);
                self.palette
                    .update(cx, |palette, cx| palette.open(mode, window, cx));
            }
            SidebarEvent::DropNotes(ids, change) => {
                self.change_notes(ids, change.clone(), window, cx)
            }
            SidebarEvent::GoToTag => {
                let mode = Mode::Pick(Picker::GoToTag);
                self.palette
                    .update(cx, |palette, cx| palette.open(mode, window, cx));
            }
            SidebarEvent::NewBook { parent } => {
                let parent = parent.clone();
                prompt_text(
                    "New Notebook",
                    "",
                    "Create",
                    window,
                    cx,
                    move |name, window, cx| {
                        let result = store.put(
                            DocType::Book,
                            json!({"name": name, "parentBookId": parent}),
                            Put::Upsert,
                            Origin::Ui,
                        );
                        match result {
                            Ok(created) => library.update(cx, |library, cx| {
                                library.set_filter(Filter::Book(created.id), cx)
                            }),
                            Err(err) => report(Err(err), window, cx),
                        }
                    },
                );
            }
            SidebarEvent::RenameBook(id) => {
                let current = self
                    .library
                    .read(cx)
                    .book(id)
                    .map(|b| b.name.clone())
                    .unwrap_or_default();
                let id = id.clone();
                prompt_text(
                    "Rename Notebook",
                    &current,
                    "Rename",
                    window,
                    cx,
                    move |name, window, cx| {
                        let result = store.put(
                            DocType::Book,
                            json!({"_id": id, "name": name}),
                            Put::Merge,
                            Origin::Ui,
                        );
                        report(result.map(drop), window, cx);
                    },
                );
            }
            SidebarEvent::LockBook(id) => {
                crate::notebook_lock::lock_notebook(id.clone(), self.editor.clone(), window, cx);
            }
            SidebarEvent::RemoveBookLock(id) => {
                crate::notebook_lock::remove_notebook_lock(
                    id.clone(),
                    self.editor.clone(),
                    window,
                    cx,
                );
            }
            SidebarEvent::PublishBook(id) => {
                self.publish_notebook(Some(id.clone()), window, cx);
            }
            SidebarEvent::DeleteBook(id) => {
                let name = self
                    .library
                    .read(cx)
                    .book(id)
                    .map(|b| b.name.clone())
                    .unwrap_or_default();
                let id = id.clone();
                let editor = self.editor.clone();
                confirm(
                    format!("Delete “{name}”?"),
                    "Its notes and the notes in its sub-notebooks move to the trash.",
                    "Delete",
                    window,
                    cx,
                    move |window, cx| {
                        editor.update(cx, |editor, cx| editor.flush(cx));
                        let result = store.delete(&id, Origin::Ui);
                        library.update(cx, |library, cx| {
                            if library.filter == Filter::Book(id.clone()) {
                                library.set_filter(Filter::All, cx);
                            }
                        });
                        report(result.map(drop), window, cx);
                    },
                );
            }
            SidebarEvent::NewTag => {
                prompt_text(
                    "New Tag",
                    "",
                    "Create",
                    window,
                    cx,
                    move |name, window, cx| {
                        let result =
                            store.put(DocType::Tag, json!({"name": name}), Put::Upsert, Origin::Ui);
                        report(result.map(drop), window, cx);
                    },
                );
            }
            SidebarEvent::RenameTag(id) => {
                let current = self
                    .library
                    .read(cx)
                    .tag(id)
                    .map(|t| t.name.clone())
                    .unwrap_or_default();
                let id = id.clone();
                prompt_text(
                    "Rename Tag",
                    &current,
                    "Rename",
                    window,
                    cx,
                    move |name, window, cx| {
                        let result = store.put(
                            DocType::Tag,
                            json!({"_id": id, "name": name}),
                            Put::Merge,
                            Origin::Ui,
                        );
                        report(result.map(drop), window, cx);
                    },
                );
            }
            SidebarEvent::ChangeBookIcon(id) => {
                let Some(book) = stored_book(&self.store, id) else {
                    return;
                };
                let current = crate::book_icons::chosen(&book);
                let store = self.store.clone();
                let id = id.clone();
                crate::book_icons::open_picker(
                    &book.name,
                    current,
                    window,
                    cx,
                    move |icon, window, cx| {
                        let field = kanso_core::model::BOOK_ICON_FIELD;
                        if let Err(err) = set_book_field(&store, &id, field, icon) {
                            window.push_notification(
                                Notification::error(format!("Couldn't change the icon: {err}")),
                                cx,
                            );
                        }
                    },
                );
            }
            SidebarEvent::ChangeBookColor(id) => {
                let Some(book) = stored_book(&self.store, id) else {
                    return;
                };
                // The preview shows the row as the sidebar does, lock and all.
                let locked = self.library.read(cx).is_book_locked(id);
                let icon = crate::book_icons::shown(
                    crate::book_icons::book_icon(&book, cx),
                    locked,
                    crate::locked::LockKeys::is_unlocked(cx),
                );
                let current = crate::book_colors::book_color(&book);
                let store = self.store.clone();
                let id = id.clone();
                crate::book_colors::open_dialog(
                    &book.name,
                    icon,
                    current,
                    window,
                    cx,
                    move |color, window, cx| {
                        let field = kanso_core::model::BOOK_COLOR_FIELD;
                        let color = color.map(crate::book_colors::hex);
                        if let Err(err) = set_book_field(&store, &id, field, color) {
                            window.push_notification(
                                Notification::error(format!("Couldn't change the color: {err}")),
                                cx,
                            );
                        }
                    },
                );
            }
            SidebarEvent::RecolorTag(id, color) => {
                self.merge(
                    DocType::Tag,
                    id,
                    json!({"color": tag_color_name(*color)}),
                    window,
                    cx,
                );
            }
            SidebarEvent::DeleteTag(id) => {
                let name = self
                    .library
                    .read(cx)
                    .tag(id)
                    .map(|t| t.name.clone())
                    .unwrap_or_default();
                let id = id.clone();
                let editor = self.editor.clone();
                confirm(
                    format!("Delete the tag “{name}”?"),
                    "Notes keep their content; the tag is removed from them.",
                    "Delete",
                    window,
                    cx,
                    move |window, cx| {
                        editor.update(cx, |editor, cx| editor.flush(cx));
                        report(store.delete(&id, Origin::Ui).map(drop), window, cx);
                    },
                );
            }
            SidebarEvent::EmptyTrash => {
                let count = self.library.read(cx).counts.trash;
                confirm(
                    format!("Delete {count} notes in the trash permanently?"),
                    "They can't be restored afterwards.",
                    "Empty Trash",
                    window,
                    cx,
                    move |window, cx| {
                        let params = crate::library::search_params(
                            &Filter::Trash,
                            "",
                            Default::default(),
                            true,
                        );
                        let result = store.search_summaries(&params).and_then(|notes| {
                            notes
                                .iter()
                                .try_for_each(|n| store.delete(&n.id, Origin::Ui).map(drop))
                        });
                        report(result, window, cx);
                    },
                );
            }
        }
    }

    /// Settings › Account with `form` open: Sign in, or Create account.
    pub fn show_account_form(
        &mut self,
        form: SignedOutForm,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.open_settings(Some(SettingsPage::Account), window, cx);
        let account = self.settings.read(cx).account.clone();
        account.update(cx, |account, cx| account.open_form(form, window, cx));
    }

    /// Asks for a folder, then exports `only` those notes, or all of them.
    fn export(&mut self, only: Option<Vec<String>>, window: &mut Window, cx: &mut Context<Self>) {
        self.flush(cx);
        let store = self.store.clone();
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Export Here".into()),
        });
        cx.spawn_in(window, async move |_, cx| {
            let Ok(Ok(Some(paths))) = picked.await else {
                return;
            };
            let Some(root) = paths.into_iter().next() else {
                return;
            };
            let result = cx
                .background_executor()
                .spawn({
                    let root = root.clone();
                    async move { kanso_engine::export::export(&store, &root, only.as_deref()) }
                })
                .await;
            cx.update(|window, cx| match result {
                Ok(report) => {
                    window.push_notification(
                        Notification::success(format!(
                            "Exported {} notes and {} attachments.{}",
                            report.notes,
                            report.attachments,
                            match report.locked {
                                0 => String::new(),
                                1 => " 1 locked note was left out.".into(),
                                n => format!(" {n} locked notes were left out."),
                            }
                        )),
                        cx,
                    );
                    cx.reveal_path(&root);
                }
                Err(err) => window
                    .push_notification(Notification::error(format!("Export failed: {err:#}")), cx),
            })
            .ok();
        })
        .detach();
    }

    /// Exports the open note as a web page, or as a PDF, in the preview's
    /// theme.
    fn export_page(&mut self, pdf: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.flush(cx);
        let Some((title, body)) = self.editor.read(cx).export_text(cx) else {
            return;
        };
        let note = self.editor.read(cx).note_id().map(str::to_owned);
        let store = self.store.clone();
        if pdf {
            crate::export_html::export_pdf(store, title, body, note, window, cx);
        } else {
            crate::export_html::export_html(store, title, body, note, window, cx);
        }
    }

    /// Shows the open note as `mode`, as the View menu does.
    #[cfg(test)]
    pub(crate) fn set_view_mode_for_test(&mut self, mode: ViewMode, cx: &mut Context<Self>) {
        self.set_view_mode(mode, cx);
    }

    fn set_view_mode(&mut self, mode: ViewMode, cx: &mut Context<Self>) {
        self.editor
            .update(cx, |editor, cx| editor.set_view_mode(mode, cx));
        AppState::update_config(cx, |config| config.appearance.view_mode = mode);
    }

    pub(crate) fn show_history(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.flush(cx);
        crate::history::open_history(self.editor.clone(), window, cx);
    }

    /// Global search: the search box over every note, whatever place or
    /// narrowing the list showed. Text already in the box stays.
    fn search_all_notes(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.library.update(cx, |library, cx| {
            library.set_filter(Filter::All, cx);
            library.clear_refinements(cx);
        });
        self.note_list
            .update(cx, |list, cx| list.focus_search(window, cx));
    }

    /// Acts on a band drag's verdict.
    fn on_band(&mut self, action: BandAction, window: &mut Window) {
        match action {
            BandAction::None => {}
            BandAction::StartMove => window.start_window_move(),
            BandAction::TitlebarDoubleClick => window.titlebar_double_click(),
        }
    }

    /// Turns focus mode on or off, and keeps it for next time. Going in,
    /// the keyboard goes to the note's text, for writing. (An open palette
    /// closes as its field loses the keyboard.)
    pub(crate) fn set_focus_mode(&mut self, on: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.focus_mode == on {
            return;
        }
        self.focus_mode = on;
        AppState::update_config(cx, |config| config.layout.focus_mode = on);
        if on {
            self.editor
                .update(cx, |editor, cx| editor.focus_for_writing(window, cx));
        }
        cx.notify();
    }

    /// The rail's width while the sidebar is folded to it.
    fn rail(&self, window: &Window) -> Option<f32> {
        (!self.show_sidebar).then(|| rail_width(window.is_fullscreen()))
    }

    /// The panes' widths as drawn: as set, or narrower when the window
    /// leaves the editor less than its minimum.
    fn fitted_widths(&self, window: &Window) -> PaneWidths {
        let width = crate::zoom::unscaled(window.viewport_size().width);
        self.pane_widths.fitted(width, self.rail(window))
    }

    /// The inspector's width as drawn, or `None` while it is closed or the
    /// window has no room for it.
    pub(crate) fn inspector_shown(&self, window: &Window) -> Option<f32> {
        if !self.inspector_open || self.focus_mode || self.settings_open() {
            return None;
        }
        let width = crate::zoom::unscaled(window.viewport_size().width);
        let before = self.leading_width(window) + self.fitted_widths(window).list;
        inspector_fitted(self.inspector_width, width, before)
    }

    /// The width of what is left of the note list: the sidebar or its rail.
    fn leading_width(&self, window: &Window) -> f32 {
        self.rail(window)
            .unwrap_or_else(|| self.fitted_widths(window).sidebar)
    }

    /// A pane's edge followed the pointer.
    fn on_pane_drag(
        &mut self,
        event: &DragMoveEvent<Rc<PaneEdge>>,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        let edge = **event.drag(cx);
        let x = crate::zoom::unscaled(event.event.position.x - event.bounds.left());
        if edge == PaneEdge::Inspector {
            let before = self.leading_width(window) + self.fitted_widths(window).list;
            let width =
                inspector_dragged(x, crate::zoom::unscaled(event.bounds.size.width), before);
            if (width - self.inspector_width).abs() > 0.1 {
                self.inspector_width = width;
                AppState::update_config_soon(cx, |config| config.layout.inspector_width = width);
                cx.notify();
            }
            return;
        }
        let rail = self.rail(window);
        // From the widths as drawn, so the edge doesn't jump to the pointer.
        let from = self.fitted_widths(window);
        let window = crate::zoom::unscaled(event.bounds.size.width);
        let widths = from.dragged(edge, x, window, rail);
        if widths != self.pane_widths {
            self.pane_widths = widths;
            cx.notify();
        }
    }

    /// The three traffic lights where the system draws them, for snapshots.
    #[cfg(test)]
    fn render_placeholder_traffic_lights(&self) -> Option<impl IntoElement> {
        use crate::layout::TRAFFIC_LIGHTS;
        // In the window's own pixels, like the system's, at any zoom.
        use gpui_kit::px;
        self.placeholder_traffic_lights.then(|| {
            div()
                .flex()
                .absolute()
                .left(px(TRAFFIC_LIGHTS.0))
                .top(px(TRAFFIC_LIGHTS.1))
                .gap(px(8.))
                .children(
                    [0xFF5F57, 0xFEBC2E, 0x28C840]
                        .map(|color| div().size(px(12.)).rounded_full().bg(gpui_kit::rgb(color))),
                )
        })
    }

    #[cfg(not(test))]
    fn render_placeholder_traffic_lights(&self) -> Option<Div> {
        None
    }
}

impl Workspace {
    /// The library picked a note: open it where `pending_open` says, and
    /// record the visit for back and forward. A silent app selection (the
    /// first note of a list) opens in the tab showing when there is one tab,
    /// and is left alone when there are more.
    fn accept_selection(
        &mut self,
        id: Option<String>,
        asked: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if asked {
            self.leave_settings(window, cx);
        }
        let intent = std::mem::replace(&mut self.pending_open, OpenIntent::Current);
        let several = self.library.read(cx).several_selected().is_some();
        // The app picking the first note of a new list (a notebook click,
        // say) with two or more tabs open changes nothing: the tab showing
        // keeps its note, and no tab is added for a note nobody chose, so
        // browsing notebooks doesn't pile up tabs.
        if !asked
            && intent == OpenIntent::Current
            && self.tabs.len() >= 2
            && id
                .as_deref()
                .is_none_or(|id| self.tabs.index_of(id).is_none())
        {
            return;
        }
        match id {
            Some(id) => {
                self.stash_place(cx);
                self.show_selected(&id, intent, window, cx);
            }
            None => {
                self.editor
                    .update(cx, |editor, cx| editor.open(None, window, cx));
            }
        }
        // Choosing several notes isn't going anywhere.
        if !several {
            self.record_visit(cx);
        }
    }

    /// Opens note `id` as `intent` asks, adding it to the tabs when it isn't
    /// open already.
    fn show_selected(
        &mut self,
        id: &str,
        intent: OpenIntent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(meta) = self.tab_meta(id) else {
            return;
        };
        // The editor can refuse to switch while a Replace holds writes and
        // the note has unsaved edits. Keep the tabs as they were then, so
        // the bar and the editor never disagree about the note showing.
        let before = self.tabs.clone();
        match intent {
            OpenIntent::NewTab => self.tabs.open_in_new(meta),
            OpenIntent::InPlace => self.tabs.set_active_meta(meta),
            OpenIntent::Activate(index)
                if self.tabs.get(index).is_some_and(|tab| tab.meta.id == id) =>
            {
                self.tabs.activate(index);
            }
            _ => self.tabs.open(meta),
        }
        self.open_active(window, cx);
        if self.editor.read(cx).note_id() == Some(id) {
            self.save_tabs(cx);
        } else {
            self.tabs = before;
            cx.notify();
        }
    }

    /// The note and title a tab for `id` shows, if the note exists.
    fn tab_meta(&self, id: &str) -> Option<TabMeta> {
        let note = self.store.note(id).ok().flatten()?;
        let locked = note.is_locked();
        Some(TabMeta {
            id: note.id,
            title: note.title,
            locked,
        })
    }

    /// Puts the showing tab's note in the editor, at the place kept for it.
    fn open_active(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(index) = (!self.tabs.is_empty()).then(|| self.tabs.active_index()) else {
            return;
        };
        let Some(id) = self.tabs.active_id().map(str::to_owned) else {
            return;
        };
        if self.editor.read(cx).note_id() == Some(id.as_str()) {
            return;
        }
        if let Some(place) = self.tabs.take_place(index) {
            self.editor
                .update(cx, |editor, _| editor.place_after_open(id.clone(), place));
        }
        self.editor
            .update(cx, |editor, cx| editor.open(Some(&id), window, cx));
    }

    /// Keeps where the reader is in the note showing, for its tab.
    fn stash_place(&mut self, cx: &mut Context<Self>) {
        if let Some((id, place)) = self.editor.read(cx).place(cx) {
            self.tabs.leave(&id, place);
        }
    }

    /// Keeps the open notes in the settings, for the next launch.
    fn save_tabs(&mut self, cx: &mut Context<Self>) {
        let open = self.tabs.ids();
        let active = self.tabs.active_index();
        AppState::update_config_soon(cx, |config| {
            config.layout.open_tabs = open;
            config.layout.active_tab = active;
        });
    }

    /// Selects note `id` in the list, opening it as `intent` asks.
    pub(crate) fn select_note(&mut self, id: &str, intent: OpenIntent, cx: &mut Context<Self>) {
        self.pending_open = intent;
        self.library
            .update(cx, |library, cx| library.select(Some(id.to_owned()), cx));
    }

    /// Shows the tab at `index` and its note.
    pub(crate) fn show_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(id) = self.tabs.get(index).map(|tab| tab.meta.id.clone()) else {
            return;
        };
        self.select_note(&id, OpenIntent::Activate(index), cx);
    }

    /// Opens note `id` in a new tab after the showing one.
    pub(crate) fn open_note_in_new_tab(
        &mut self,
        id: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_note(id, OpenIntent::NewTab, cx);
    }

    pub(crate) fn close_tab(&mut self, index: usize, _window: &mut Window, cx: &mut Context<Self>) {
        let was_active = self.tabs.active_index() == index;
        if !self.tabs.close(index) {
            return;
        }
        self.save_tabs(cx);
        if was_active && !self.tabs.is_empty() {
            self.show_tab(self.tabs.active_index(), cx);
        }
        cx.notify();
    }

    fn close_tabs_from(
        &mut self,
        index: usize,
        right: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let was_active = self.tabs.active_index();
        let closed = if right {
            self.tabs.close_right(index)
        } else {
            self.tabs.close_others(index)
        };
        if !closed {
            return;
        }
        self.save_tabs(cx);
        if self.tabs.active_index() != was_active {
            self.show_tab(self.tabs.active_index(), cx);
        }
        cx.notify();
        let _ = window;
    }

    fn on_tab_bar_event(
        &mut self,
        event: &TabBarEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            TabBarEvent::Activate(index) => self.show_tab(*index, cx),
            TabBarEvent::Close(index) => self.close_tab(*index, window, cx),
            TabBarEvent::CloseOthers(index) => self.close_tabs_from(*index, false, window, cx),
            TabBarEvent::CloseRight(index) => self.close_tabs_from(*index, true, window, cx),
            TabBarEvent::OpenInNewWindow(id) => self.open_in_new_window(id, window, cx),
            TabBarEvent::Reorder { from, to } => {
                self.tabs.reorder(*from, *to);
                self.save_tabs(cx);
                cx.notify();
            }
        }
    }

    /// Keeps the tabs in step with a note changed elsewhere: a rename or a
    /// new lock updates the title and mark, and a trashed or deleted note
    /// closes its tab.
    fn sync_tabs_with_store(
        &mut self,
        change: &kanso_store::Change,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !change.id.starts_with("note:") {
            return;
        }
        let note = if change.deleted {
            None
        } else {
            self.store.note(&change.id).ok().flatten()
        };
        // A note someone is looking at in the trash keeps its tab until
        // they leave the trash.
        let trashed = note.as_ref().is_some_and(|note| note.is_trashed())
            && self.library.read(cx).filter != Filter::Trash;
        if note.is_none() || trashed {
            // A note with text the editor has not saved stays in its tab
            // until that settles, so nothing is lost; the tab closes once
            // the text is saved or the question about the delete answered
            // (`recheck_pending_tab_closes`).
            let showing = self.tabs.active_id() == Some(change.id.as_str());
            if showing && self.editor.read(cx).has_unsaved_edits(cx) {
                self.pending_tab_closes.insert(change.id.clone());
                return;
            }
            self.pending_tab_closes.remove(&change.id);
            let was_active = self.tabs.active_id() == Some(change.id.as_str());
            if !self.tabs.remove_note(&change.id) {
                return;
            }
            self.save_tabs(cx);
            if was_active {
                self.open_after_tab_gone(window, cx);
            }
        } else if let Some(note) = note {
            // The note came back (Keep Mine, or a sync): its tab stays.
            self.pending_tab_closes.remove(&change.id);
            let locked = note.is_locked();
            self.tabs.update_note(&change.id, note.title, locked);
            cx.notify();
        }
    }

    /// Tries again to close the tabs kept while the editor had unsaved
    /// text about a note deleted elsewhere: once the text is saved, the
    /// note is back (Keep Mine), or the question is answered, the tab can
    /// go.
    fn recheck_pending_tab_closes(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let pending: Vec<String> = self.pending_tab_closes.iter().cloned().collect();
        for id in pending {
            let note = self.store.note(&id).ok().flatten();
            let trashed = note.as_ref().is_some_and(|note| note.is_trashed())
                && self.library.read(cx).filter != Filter::Trash;
            if note.is_some() && !trashed {
                self.pending_tab_closes.remove(&id);
                continue;
            }
            let showing = self.tabs.active_id() == Some(id.as_str());
            if showing && self.editor.read(cx).has_unsaved_edits(cx) {
                continue;
            }
            self.pending_tab_closes.remove(&id);
            if !self.tabs.remove_note(&id) {
                continue;
            }
            self.save_tabs(cx);
            if showing {
                self.open_after_tab_gone(window, cx);
            }
        }
    }

    /// Closes the tab of a note the editor has let go of.
    fn close_tab_of_note(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.pending_tab_closes.remove(id);
        let was_active = self.tabs.active_id() == Some(id);
        if !self.tabs.remove_note(id) {
            return;
        }
        self.save_tabs(cx);
        if was_active {
            self.open_after_tab_gone(window, cx);
        } else {
            cx.notify();
        }
    }

    /// Shows the tab next to one whose note has gone, or no note when no
    /// tab is left.
    fn open_after_tab_gone(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.tabs.active_id().map(str::to_owned) {
            let index = self.tabs.active_index();
            if let Some(place) = self.tabs.take_place(index) {
                self.editor
                    .update(cx, |editor, _| editor.place_after_open(id.clone(), place));
            }
            self.editor
                .update(cx, |editor, cx| editor.open(Some(&id), window, cx));
            // The list is in the middle of a change; select its neighbour
            // once that is done.
            cx.defer_in(window, move |this, _window, cx| {
                this.pending_open = OpenIntent::Activate(index);
                this.library
                    .update(cx, |library, cx| library.select_quietly(Some(id), cx));
            });
        } else {
            self.editor
                .update(cx, |editor, cx| editor.open(None, window, cx));
            cx.defer_in(window, |this, _window, cx| {
                this.pending_open = OpenIntent::Current;
                this.library
                    .update(cx, |library, cx| library.select_quietly(None, cx));
            });
        }
    }
}

/// The divider on a pane's left edge, which drags `edge`: the pane before
/// it, the sidebar or the list.
// no-prop-test: drawing; the widths a drag makes are tested in `layout`.
fn pane_handle(edge: PaneEdge) -> impl IntoElement {
    let id = match edge {
        PaneEdge::Sidebar => "sidebar-edge",
        PaneEdge::NoteList => "list-edge",
        PaneEdge::Inspector => "inspector-edge",
    };
    resize_handle::<PaneEdge, PaneEdge>(id, Axis::Horizontal).on_drag(edge, |edge, _, _, cx| {
        cx.stop_propagation();
        cx.new(|_| *edge)
    })
}

/// What a pane drag shows under the pointer: nothing, the divider moves.
impl Render for PaneEdge {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

pub(crate) use kanso_engine::links::{relink_label, relink_message};

/// The JSON name of a tag color.
pub fn tag_color_name(color: TagColor) -> String {
    serde_json::to_value(color)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_else(|| "default".into())
}

impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        // The sidebar or its rail is leftmost and keeps the traffic lights'
        // room; the list's top band leaves whatever is left of it free.
        let leading = self.leading_width(window);
        let list_inset = px(inset_after(leading, window.is_fullscreen()));
        self.note_list
            .update(cx, |list, cx| list.set_leading_inset(list_inset, cx));
        self.tasks
            .update(cx, |tasks, cx| tasks.set_leading_inset(list_inset, cx));
        let tasks_shown = self.library.read(cx).filter == Filter::Tasks;
        self.settings.update(cx, |settings, cx| {
            settings.set_leading_inset(list_inset, cx)
        });
        let collapsed = !self.show_sidebar;
        self.sidebar
            .update(cx, |sidebar, cx| sidebar.set_collapsed(collapsed, cx));
        // Opening Settings leaves focus mode, so the two never show together;
        // Settings would win if they did.
        let settings_open = self.settings_open();
        let focus_mode = self.focus_mode && !settings_open;
        self.editor
            .update(cx, |editor, cx| editor.set_focus_mode(focus_mode, cx));
        let widths = self.fitted_widths(window);
        let inspector = self.inspector_shown(window);
        self.inspector.update(cx, |inspector_view, _| {
            inspector_view.visible = inspector.is_some();
        });
        self.palette
            .update(cx, |palette, _| palette.set_notes_hidden(settings_open));
        let show_tabs = self.tabs.len() >= 2 && !focus_mode;
        let tab_metas = self.tabs.metas();
        let active_tab = self.tabs.active_index();
        self.tab_bar
            .update(cx, |bar, cx| bar.set(tab_metas, active_tab, window, cx));
        let signed_in = cx
            .try_global::<crate::services::Services>()
            .is_some_and(|services| services.signed_in());
        let signed_in_locked = cx
            .try_global::<crate::services::Services>()
            .is_some_and(crate::services::Services::has_locked_session);
        let (can_go_back, can_go_forward) = self.nav_reach();
        v_flex()
            .size_full()
            .relative()
            // Escape leaves focus mode, or Settings, once nothing inside
            // (the palette, a menu, an input) has taken it.
            .key_context(if focus_mode {
                "Workspace FocusMode"
            } else if settings_open {
                "Workspace Settings"
            } else {
                "Workspace"
            })
            .track_focus(&self.focus)
            .bg(colors.app)
            .text_color(cx.theme().foreground)
            // Controls mark the presses they take; start each press clean.
            .capture_any_mouse_down(cx.listener(|this, event: &MouseDownEvent, _, cx| {
                if cx.has_global::<GlobalState>() {
                    GlobalState::reset_text_selection_suppression(cx);
                }
                // The mouse's back and forward buttons, wherever they press.
                match event.button {
                    MouseButton::Navigate(NavigationDirection::Back) => this.go_back(cx),
                    MouseButton::Navigate(NavigationDirection::Forward) => this.go_forward(cx),
                    _ => {}
                }
            }))
            // Presses on empty parts of the top bands move the window. A
            // button prevents the default; an input claims the press for its
            // own text selection.
            .on_any_mouse_down(cx.listener(|this, event: &MouseDownEvent, window, cx| {
                let claimed = window.default_prevented()
                    || (cx.has_global::<GlobalState>()
                        && GlobalState::is_text_selection_suppressed(cx));
                // Where Kanso draws a title bar, that is what moves the window.
                if event.button == MouseButton::Left && this.title_bar.is_none() {
                    let action = this
                        .band_drag
                        .down(event.position, event.click_count, claimed);
                    this.on_band(action, window);
                }
                // The workspace's focus only gives the keyboard a place when
                // nothing else has it. A press on a band or any other bare
                // chrome, or on text selected for copying, leaves the
                // keyboard where it was, as on macOS: without this the focus
                // would move here and typing would go nowhere.
                window.prevent_default();
            }))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, window, _| {
                if this.band_drag.is_armed() {
                    let pressed = event.pressed_button == Some(MouseButton::Left);
                    let action = this.band_drag.moved(event.position, pressed);
                    this.on_band(action, window);
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _, _| this.band_drag.up()),
            )
            .on_action(cx.listener(|this, _: &NewNote, window, cx| this.new_note(window, cx)))
            .on_action(cx.listener(|this, _: &NewNotebook, window, cx| {
                this.on_sidebar_event(&SidebarEvent::NewBook { parent: None }, window, cx);
            }))
            .on_action(cx.listener(|this, _: &FocusSearch, window, cx| {
                this.leave_settings(window, cx);
                // The search box is in the list, which focus mode hides.
                this.set_focus_mode(false, window, cx);
                this.note_list
                    .update(cx, |list, cx| list.focus_search(window, cx));
            }))
            .on_action(cx.listener(|this, _: &SearchAllNotes, window, cx| {
                this.search_all_notes(window, cx);
            }))
            .on_action(cx.listener(|this, _: &FocusNoteList, window, cx| {
                // The list is hidden behind Settings and in focus mode.
                this.leave_settings(window, cx);
                this.set_focus_mode(false, window, cx);
                if this.library.read(cx).filter == Filter::Tasks {
                    this.tasks
                        .update(cx, |tasks, cx| tasks.focus_list(window, cx));
                } else {
                    this.note_list
                        .update(cx, |list, cx| list.focus_list(window, cx));
                }
            }))
            .on_action(cx.listener(|this, _: &FocusEditor, window, cx| {
                this.editor
                    .update(cx, |editor, cx| editor.focus_body(window, cx));
            }))
            .on_action(cx.listener(|this, _: &ToggleSidebar, window, cx| {
                // In focus mode there is no sidebar to toggle: it comes back.
                let show = this.focus_mode || !this.show_sidebar;
                this.set_focus_mode(false, window, cx);
                this.show_sidebar = show;
                AppState::update_config(cx, |config| config.appearance.show_sidebar = show);
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &ToggleFocusMode, window, cx| {
                // From Settings, back to the note, to write in it alone.
                if this.settings_open() {
                    this.close_settings(window, cx);
                    this.set_focus_mode(true, window, cx);
                } else {
                    this.set_focus_mode(!this.focus_mode, window, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &ExitFocusMode, window, cx| {
                this.set_focus_mode(false, window, cx);
            }))
            .on_action(cx.listener(|this, _: &ToggleInspector, window, cx| {
                this.close_settings(window, cx);
                this.set_focus_mode(false, window, cx);
                this.toggle_inspector(cx);
            }))
            .on_action(cx.listener(|this, _: &ShowOutline, window, cx| {
                this.close_settings(window, cx);
                this.set_focus_mode(false, window, cx);
                this.show_inspector(Some(InspectorTab::Outline), cx);
            }))
            .on_action(cx.listener(|this, _: &ShowBacklinks, window, cx| {
                this.close_settings(window, cx);
                this.set_focus_mode(false, window, cx);
                this.show_inspector(Some(InspectorTab::Backlinks), cx);
            }))
            .on_action(cx.listener(|this, action: &InlineAssist, window, cx| {
                // The palette closed with the focus elsewhere; the editor
                // takes it from here.
                this.close_settings(window, cx);
                this.editor
                    .update(cx, |editor, cx| editor.start_inline(action.action, cx));
            }))
            .on_action(cx.listener(|this, _: &SummarizeNote, window, cx| {
                if !cx.global::<AppState>().config.ai.enabled {
                    return;
                }
                this.close_settings(window, cx);
                this.set_focus_mode(false, window, cx);
                this.show_inspector(Some(InspectorTab::Ask), cx);
                this.inspector
                    .update(cx, |inspector, cx| inspector.summarize_note(window, cx));
            }))
            .on_action(cx.listener(|this, _: &AskYourNotes, window, cx| {
                if !cx.global::<AppState>().config.ai.enabled {
                    return;
                }
                this.close_settings(window, cx);
                this.set_focus_mode(false, window, cx);
                this.show_inspector(Some(InspectorTab::Ask), cx);
                this.inspector
                    .update(cx, |inspector, cx| inspector.focus_ask(window, cx));
            }))
            // What acts on the open note brings it back from behind
            // Settings first, so the note it changes is in view.
            .on_action(cx.listener(|this, _: &ShowEditor, window, cx| {
                this.close_settings(window, cx);
                this.set_view_mode(ViewMode::Edit, cx)
            }))
            .on_action(cx.listener(|this, _: &ShowSplit, window, cx| {
                this.close_settings(window, cx);
                this.set_view_mode(ViewMode::Split, cx)
            }))
            .on_action(cx.listener(|this, _: &ShowPreview, window, cx| {
                this.close_settings(window, cx);
                this.set_view_mode(ViewMode::Preview, cx)
            }))
            .on_action(cx.listener(|this, _: &SaveNote, _, cx| this.flush(cx)))
            .on_action(cx.listener(|this, _: &TogglePinned, window, cx| {
                this.close_settings(window, cx);
                if let Some(ids) = this.several_in_list(window, cx) {
                    return this.change_notes(&ids, BatchChange::TogglePin, window, cx);
                }
                this.editor
                    .update(cx, |editor, cx| editor.toggle_pinned(cx));
            }))
            .on_action(cx.listener(|this, _: &PastePlainText, window, cx| {
                this.editor
                    .update(cx, |editor, cx| editor.paste_plain(window, cx));
            }))
            .on_action(cx.listener(|this, _: &FormatTable, window, cx| {
                this.close_settings(window, cx);
                this.editor
                    .update(cx, |editor, cx| editor.format_table(window, cx));
            }))
            .on_action(cx.listener(|this, _: &NewDrawing, window, cx| {
                this.close_settings(window, cx);
                this.editor
                    .update(cx, |editor, cx| editor.insert_drawing(window, cx));
            }))
            .on_action(cx.listener(|this, _: &TrashNote, window, cx| {
                this.close_settings(window, cx);
                if let Some(ids) = this.several_in_list(window, cx) {
                    let trash = BatchChange::Move(TRASH_BOOK_ID.into());
                    return this.change_notes(&ids, trash, window, cx);
                }
                this.editor.update(cx, |editor, cx| editor.trash(cx));
            }))
            .on_action(
                cx.listener(|this, _: &crate::transcribe::TranscribeAudio, window, cx| {
                    this.close_settings(window, cx);
                    let found = this.editor.read(cx).audio_on_caret_line(cx);
                    match found {
                        Some((file_id, name)) => crate::transcribe::start(
                            this.editor.clone(),
                            crate::transcribe::Source::Attachment { file_id, name },
                            window,
                            cx,
                        ),
                        None => window.push_notification(
                            gpui_kit::component::notification::Notification::info(
                                "Put the cursor on the line with a recording to transcribe it, or drop a recording into the note.",
                            ),
                            cx,
                        ),
                    }
                }),
            )
            .on_action(cx.listener(|this, _: &Dictate, window, cx| {
                this.close_settings(window, cx);
                crate::voice::press(this.editor.clone(), window, cx);
            }))
            .on_action(cx.listener(|this, _: &ShowHistory, window, cx| {
                this.close_settings(window, cx);
                this.show_history(window, cx);
            }))
            .on_action(cx.listener(
                |this, _: &crate::note_window::OpenInNewWindow, window, cx| {
                    this.close_settings(window, cx);
                    let id = this.editor.read(cx).note_id().map(str::to_owned);
                    if let Some(id) = id {
                        this.open_in_new_window(&id, window, cx);
                    }
                },
            ))
            .on_action(cx.listener(|this, _: &ExportNote, window, cx| {
                this.close_settings(window, cx);
                let id = this.editor.read(cx).note_id().map(str::to_owned);
                if let Some(id) = id {
                    this.export(Some(vec![id]), window, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &ExportHtml, window, cx| {
                this.close_settings(window, cx);
                this.export_page(false, window, cx);
            }))
            .on_action(cx.listener(|this, _: &ExportPdf, window, cx| {
                this.close_settings(window, cx);
                this.export_page(true, window, cx);
            }))
            .on_action(cx.listener(
                |_, _: &crate::settings::ImportIconTheme, window, cx| {
                    crate::settings_themes::import_icon_theme(window, cx);
                },
            ))
            .on_action(cx.listener(
                |_, _: &crate::settings::ReloadIconThemes, window, cx| {
                    crate::settings_themes::reload_icon_themes(window, cx);
                },
            ))
            .on_action(
                cx.listener(|this, _: &crate::publish::PublishNote, window, cx| {
                    this.close_settings(window, cx);
                    crate::publish::open_publish(this.editor.clone(), window, cx);
                }),
            )
            .on_action(cx.listener(|this, _: &crate::collab::ShareNote, window, cx| {
                this.close_settings(window, cx);
                crate::share_sheet::open_share(this.editor.clone(), window, cx);
            }))
            .on_action(cx.listener(|_, _: &crate::collab::JoinSharedNote, window, cx| {
                crate::collab::join_dialog(window, cx);
            }))
            .on_action(
                cx.listener(|this, _: &crate::collab::CommentOnSelection, window, cx| {
                    let editor = this.editor.clone();
                    let workspace = cx.entity().downgrade();
                    crate::dialogs::prompt_checked(
                        "Comment on the selection",
                        "",
                        "Comment",
                        window,
                        cx,
                        move |text, window, cx| {
                            editor.update(cx, |editor, cx| {
                                editor.comment_on_selection(&text, window, cx)
                            })?;
                            // The new comment, among the others.
                            workspace
                                .update(cx, |workspace, cx| {
                                    workspace.show_inspector(Some(InspectorTab::Comments), cx)
                                })
                                .ok();
                            Ok(())
                        },
                    );
                }),
            )
            .on_action(cx.listener(|this, _: &crate::collab::ShowComments, window, cx| {
                this.close_settings(window, cx);
                this.set_focus_mode(false, window, cx);
                this.show_inspector(Some(InspectorTab::Comments), cx);
            }))
            .on_action(
                cx.listener(|this, _: &crate::locked::LockNote, window, cx| {
                    this.close_settings(window, cx);
                    crate::locked::lock_open_note(this.editor.clone(), window, cx);
                }),
            )
            .on_action(
                cx.listener(|this, _: &crate::locked::UnlockNote, window, cx| {
                    crate::locked::unlock_open_note(this.editor.clone(), window, cx);
                }),
            )
            .on_action(
                cx.listener(|this, _: &crate::locked::RemoveLock, window, cx| {
                    this.close_settings(window, cx);
                    crate::locked::remove_lock(this.editor.clone(), window, cx);
                }),
            )
            .on_action(
                cx.listener(|this, _: &crate::locked::LockNotesNow, window, cx| {
                    crate::locked::lock_now(&this.editor, window, cx);
                }),
            )
            .on_action(
                cx.listener(|this, _: &crate::locked::ChangeLockPassword, window, cx| {
                    crate::locked::change_password(this.editor.clone(), window, cx);
                }),
            )
            .on_action(
                cx.listener(|_, _: &crate::locked::TurnOnTouchId, window, cx| {
                    crate::locked::turn_on_touch_id(window, cx);
                }),
            )
            .on_action(
                cx.listener(|_, _: &crate::locked::TurnOffTouchId, window, cx| {
                    crate::locked::turn_off_touch_id(window, cx);
                }),
            )
            .on_action(
                cx.listener(|this, _: &ExportLibrary, window, cx| this.export(None, window, cx)),
            )
            .on_action(cx.listener(|_, _: &ImportPouchLibrary, window, cx| {
                crate::pouch_import::start_import(window, cx);
            }))
            .on_action(cx.listener(|_, _: &ImportLibrary, window, cx| {
                crate::library_import::start(window, cx);
            }))
            .on_action(cx.listener(|this, _: &OpenFile, window, cx| {
                this.flush(cx);
                let library = this.library.clone();
                crate::library_import::open_files(window, cx, move |id, _, cx| {
                    library.update(cx, |library, cx| library.select(Some(id), cx));
                });
            }))
            .on_action(cx.listener(|this, _: &NextNote, _, cx| {
                this.library
                    .update(cx, |library, cx| library.select_relative(1, cx));
            }))
            .on_action(cx.listener(|this, _: &PreviousNote, _, cx| {
                this.library
                    .update(cx, |library, cx| library.select_relative(-1, cx));
            }))
            .on_action(cx.listener(|this, _: &crate::tabs::CloseTab, window, cx| {
                this.close_current_tab(window, cx)
            }))
            .on_action(cx.listener(|this, _: &crate::tabs::NextTab, _, cx| this.next_tab(cx)))
            .on_action(cx.listener(|this, _: &crate::tabs::PreviousTab, _, cx| {
                this.previous_tab(cx)
            }))
            .on_action(cx.listener(|this, _: &crate::tabs::ReopenClosedTab, _, cx| {
                this.reopen_closed_tab(cx)
            }))
            .on_action(cx.listener(|this, _: &crate::tabs::GoToTab1, _, cx| this.go_to_tab(1, cx)))
            .on_action(cx.listener(|this, _: &crate::tabs::GoToTab2, _, cx| this.go_to_tab(2, cx)))
            .on_action(cx.listener(|this, _: &crate::tabs::GoToTab3, _, cx| this.go_to_tab(3, cx)))
            .on_action(cx.listener(|this, _: &crate::tabs::GoToTab4, _, cx| this.go_to_tab(4, cx)))
            .on_action(cx.listener(|this, _: &crate::tabs::GoToTab5, _, cx| this.go_to_tab(5, cx)))
            .on_action(cx.listener(|this, _: &crate::tabs::GoToTab6, _, cx| this.go_to_tab(6, cx)))
            .on_action(cx.listener(|this, _: &crate::tabs::GoToTab7, _, cx| this.go_to_tab(7, cx)))
            .on_action(cx.listener(|this, _: &crate::tabs::GoToTab8, _, cx| this.go_to_tab(8, cx)))
            .on_action(cx.listener(|this, _: &crate::tabs::GoToTab9, _, cx| this.go_to_tab(9, cx)))
            // Only where they would land, so the menu's items are disabled
            // otherwise.
            .when(can_go_back, |el| {
                el.on_action(cx.listener(|this, _: &GoBack, _, cx| this.go_back(cx)))
            })
            .when(can_go_forward, |el| {
                el.on_action(cx.listener(|this, _: &GoForward, _, cx| this.go_forward(cx)))
            })
            .on_action(cx.listener(|this, _: &CopyNoteLink, window, cx| {
                this.copy_note_link(false, window, cx)
            }))
            .on_action(cx.listener(|this, _: &CopyKansoLink, window, cx| {
                this.copy_note_link(true, window, cx)
            }))
            .on_action(cx.listener(|this, _: &OpenCommandPalette, window, cx| {
                this.palette
                    .update(cx, |palette, cx| palette.open(Mode::Commands, window, cx));
            }))
            .on_action(cx.listener(|this, _: &OpenNotePicker, window, cx| {
                this.palette
                    .update(cx, |palette, cx| palette.open(Mode::Notes, window, cx));
            }))
            // Only while signed in, so the menu's Sync Now is disabled
            // otherwise.
            .when(signed_in, |el| {
                el.on_action(cx.listener(|_, _: &SyncNow, _, cx| crate::services::sync_now(cx)))
                    .on_action(cx.listener(|this, _: &ChangePassword, window, cx| {
                        this.open_settings(Some(SettingsPage::Account), window, cx);
                        crate::account_page::change_password_dialog(window, cx);
                    }))
                    .on_action(cx.listener(|this, _: &NewRecoveryKey, window, cx| {
                        this.open_settings(Some(SettingsPage::Account), window, cx);
                        crate::account_page::new_recovery_key_dialog(window, cx);
                    }))
                    .on_action(cx.listener(|_, _: &SignOut, window, cx| {
                        crate::account_page::sign_out_dialog(window, cx);
                    }))
            })
            .when(!signed_in && !signed_in_locked, |el| {
                el.on_action(cx.listener(|this, _: &ShowSignIn, window, cx| {
                    this.show_account_form(SignedOutForm::SignIn, window, cx)
                }))
                .on_action(cx.listener(
                    |this, _: &ShowCreateAccount, window, cx| {
                        this.show_account_form(SignedOutForm::CreateAccount, window, cx)
                    },
                ))
            })
            .map(|el| crate::settings::on_actions(el, cx))
            .map(|el| crate::journal::on_actions(el, cx))
            .map(|el| crate::calendar_view::on_actions(el, cx))
            .map(|el| crate::publish_site::on_actions(el, cx))
            .map(|el| crate::replace::on_actions(el, cx))
            .map(|el| crate::updates::on_actions(el, cx))
            .map(|el| crate::graph_view::on_actions(el, cx))
            .map(|el| crate::board_view::on_actions(el, cx))
            // Off macOS, Kanso's own title bar with the menus.
            .when_some(self.title_bar.clone(), |el, menus| {
                el.child(crate::title_bar::render(&menus, colors))
            })
            // The panes run the rest of the height: each has its own top
            // band (on macOS, the title bar's row) and bottom band. The
            // sidebar and the list keep their widths and the editor takes
            // the rest; each pane's left edge drags the pane before it.
            .child(
                h_flex()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .on_drag_move(cx.listener(
                        |this, event: &DragMoveEvent<Rc<PaneEdge>>, window, cx| {
                            this.on_pane_drag(event, window, cx)
                        },
                    ))
                    // Focus mode leaves the note alone in the window.
                    .when(!focus_mode, |el| {
                        el.child(
                            div()
                                .h_full()
                                .w(px(leading))
                                .flex_none()
                                .debug_selector(|| "sidebar-pane".into())
                                .child(self.sidebar.clone()),
                        )
                    })
                    // Settings take the list's and the editor's place, beside
                    // the sidebar or its rail.
                    .when(settings_open, |el| {
                        el.child(
                            div()
                                .relative()
                                .h_full()
                                .flex_1()
                                .min_w_0()
                                .debug_selector(|| "settings-pane".into())
                                .child(self.settings.clone())
                                // The rail keeps its width.
                                .when(self.show_sidebar, |el| {
                                    el.child(pane_handle(PaneEdge::Sidebar))
                                }),
                        )
                    })
                    .when(!settings_open, |el| {
                        el.when(!focus_mode, |el| {
                            el.child(
                                div()
                                    .relative()
                                    .h_full()
                                    .w(px(widths.list))
                                    .flex_none()
                                    .debug_selector(|| "list-pane".into())
                                    .map(|el| {
                                        if tasks_shown {
                                            el.child(self.tasks.clone())
                                        } else {
                                            el.child(self.note_list.clone())
                                        }
                                    })
                                    // The rail keeps its width.
                                    .when(self.show_sidebar, |el| {
                                        el.child(pane_handle(PaneEdge::Sidebar))
                                    }),
                            )
                        })
                        .child(
                            div()
                                .relative()
                                .h_full()
                                .flex_1()
                                .min_w_0()
                                .debug_selector(|| "editor-pane".into())
                                .child(
                                    v_flex()
                                        .size_full()
                                        // Between the window's title bar and
                                        // the editor, as in a browser.
                                        .when(show_tabs, |el| {
                                            el.child(self.tab_bar.clone())
                                        })
                                        .child(
                                            div()
                                                .flex_1()
                                                .min_h_0()
                                                .child(self.editor.clone()),
                                        ),
                                )
                                .when(!focus_mode, |el| el.child(pane_handle(PaneEdge::NoteList))),
                        )
                        .when_some(inspector, |el, width| {
                            el.child(
                                div()
                                    .relative()
                                    .h_full()
                                    .w(px(width))
                                    .flex_none()
                                    .debug_selector(|| "inspector-pane".into())
                                    .child(self.inspector.clone())
                                    .child(pane_handle(PaneEdge::Inspector)),
                            )
                        })
                    }),
            )
            .children(crate::graph_view::overlay(self))
            .children(crate::board_view::overlay(self))
            // Over everything else when open.
            .child(self.palette.clone())
            // The system draws the real traffic lights above the palette too.
            .children(self.render_placeholder_traffic_lights())
    }
}

/// Notebook `id` as the store keeps it, if there is one.
fn stored_book(store: &Store, id: &str) -> Option<kanso_core::Book> {
    let doc = store.get(id).ok()??;
    serde_json::from_value(doc.to_api(Default::default())).ok()
}

/// Sets notebook `id`'s `field` (its icon or color) to `value`, or takes it
/// away for the default. A whole write, since a merge can't remove a field.
fn set_book_field(
    store: &Store,
    id: &str,
    field: &str,
    value: Option<String>,
) -> Result<(), kanso_store::StoreError> {
    let doc = store.get(id)?.ok_or(kanso_store::StoreError::NotFound)?;
    let mut book = doc.to_api(Default::default());
    let Some(fields) = book.as_object_mut() else {
        return Err(kanso_store::StoreError::NotFound);
    };
    match value {
        Some(value) => fields.insert(field.into(), value.into()),
        None => fields.remove(field),
    };
    store.put(DocType::Book, book, Put::Upsert, Origin::Ui)?;
    Ok(())
}
