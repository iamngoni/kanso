//! The one catalog of keyboard shortcuts (issue #48, package Q): every key
//! the desktop binds, as plain data.
//!
//! The desktop binds these keys itself, and a test in `kanso-app` holds that
//! binding to this catalog in both directions, so the catalog stays exact as
//! the desktop changes. The phones and tablets build their key map from it,
//! and the shortcut overlay and the command palette read the same entries.
//!
//! Keys are written as GPUI writes them: modifiers (`secondary`, `ctrl`,
//! `alt`, `shift`, `cmd`) joined to a key by `-`, as in `secondary-shift-t`,
//! `ctrl-tab`, `cmd-[` or `escape`. `secondary` is ⌘ on macOS and iPadOS and
//! Ctrl elsewhere. [`Shortcut::mac`] is what the desktop binds on macOS (and
//! what an iPad uses) and [`Shortcut::other`] what it binds on Windows and
//! Linux (and what an Android device uses). They differ only where the
//! desktop binds per system. An empty `other` means the system binds
//! nothing there.
//!
//! An action with two keys has two entries with the same `id`; the same
//! action in two contexts does too. The `id` is the desktop action's name in
//! snake case.

/// Where a shortcut acts: the desktop's key contexts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ShortcutContext {
    /// Anywhere in the window.
    Global,
    /// The note list has the keyboard.
    NoteList,
    /// The note's text has the keyboard.
    Editor,
    /// Focus mode is on.
    FocusMode,
    /// A note open in a window of its own.
    NoteWindow,
    /// The command palette or quick open is open.
    Palette,
    /// The tasks view.
    TasksView,
    /// The board view.
    BoardView,
    /// The journal calendar.
    CalendarView,
    /// The graph view.
    GraphView,
    /// The settings window.
    Settings,
    /// The settings window's list of pages.
    SettingsNav,
    /// The Quick Capture window.
    Capture,
}

/// The menu a shortcut is listed under, in the order the overlay arranges
/// them: the desktop's menus first, then the views with keys of their own.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ShortcutGroup {
    /// The Kanso menu.
    App,
    File,
    Edit,
    Format,
    View,
    Go,
    Note,
    Window,
    Tabs,
    /// The note list's own keys.
    List,
    Palette,
    Calendar,
    Tasks,
    Board,
    Graph,
    Settings,
    Capture,
}

/// One key for one action in one context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shortcut {
    /// The desktop action's name in snake case.
    pub id: &'static str,
    /// The keys on macOS and iPadOS.
    pub mac: &'static str,
    /// The keys on Windows, Linux and Android; empty when nothing is bound.
    pub other: &'static str,
    /// Where the keys act.
    pub context: ShortcutContext,
    /// What the desktop's menu calls the action.
    pub label: &'static str,
    /// The menu it is listed under.
    pub group: ShortcutGroup,
    /// Whether a phone or tablet binds it: false for Quit, which the system
    /// does, and the Quick Capture window's keys.
    pub on_mobile: bool,
}

impl Shortcut {
    /// The keys for macOS (`apple`) or for the other systems.
    pub fn keys(&self, apple: bool) -> &'static str {
        if apple { self.mac } else { self.other }
    }
}

/// Every shortcut, in the order of the desktop's menus.
pub const SHORTCUTS: &[Shortcut] = &[
    Shortcut {
        id: "new_note",
        mac: "secondary-n",
        other: "secondary-n",
        context: ShortcutContext::Global,
        label: "New Note",
        group: ShortcutGroup::File,
        on_mobile: true,
    },
    Shortcut {
        id: "new_notebook",
        mac: "secondary-shift-n",
        other: "secondary-shift-n",
        context: ShortcutContext::Global,
        label: "New Notebook",
        group: ShortcutGroup::File,
        on_mobile: true,
    },
    Shortcut {
        id: "save_note",
        mac: "secondary-s",
        other: "secondary-s",
        context: ShortcutContext::Global,
        label: "Save",
        group: ShortcutGroup::File,
        on_mobile: true,
    },
    Shortcut {
        id: "open_today",
        mac: "secondary-d",
        other: "secondary-d",
        context: ShortcutContext::Global,
        label: "Today's Note",
        group: ShortcutGroup::File,
        on_mobile: true,
    },
    Shortcut {
        id: "paste_plain_text",
        mac: "secondary-shift-v",
        other: "secondary-shift-v",
        context: ShortcutContext::Editor,
        label: "Paste and Match Style",
        group: ShortcutGroup::Edit,
        on_mobile: true,
    },
    Shortcut {
        id: "format_bold",
        mac: "secondary-b",
        other: "secondary-b",
        context: ShortcutContext::Editor,
        label: "Bold",
        group: ShortcutGroup::Format,
        on_mobile: true,
    },
    Shortcut {
        id: "format_italic",
        mac: "secondary-i",
        other: "secondary-i",
        context: ShortcutContext::Editor,
        label: "Italic",
        group: ShortcutGroup::Format,
        on_mobile: true,
    },
    Shortcut {
        id: "format_strikethrough",
        mac: "secondary-shift-x",
        other: "secondary-shift-x",
        context: ShortcutContext::Editor,
        label: "Strikethrough",
        group: ShortcutGroup::Format,
        on_mobile: true,
    },
    Shortcut {
        id: "format_code",
        mac: "secondary-e",
        other: "secondary-e",
        context: ShortcutContext::Editor,
        label: "Code",
        group: ShortcutGroup::Format,
        on_mobile: true,
    },
    Shortcut {
        id: "format_link",
        mac: "secondary-shift-k",
        other: "secondary-shift-k",
        context: ShortcutContext::Editor,
        label: "Link",
        group: ShortcutGroup::Format,
        on_mobile: true,
    },
    Shortcut {
        id: "format_heading_1",
        mac: "secondary-alt-1",
        other: "secondary-alt-1",
        context: ShortcutContext::Editor,
        label: "Heading 1",
        group: ShortcutGroup::Format,
        on_mobile: true,
    },
    Shortcut {
        id: "format_heading_2",
        mac: "secondary-alt-2",
        other: "secondary-alt-2",
        context: ShortcutContext::Editor,
        label: "Heading 2",
        group: ShortcutGroup::Format,
        on_mobile: true,
    },
    Shortcut {
        id: "format_heading_3",
        mac: "secondary-alt-3",
        other: "secondary-alt-3",
        context: ShortcutContext::Editor,
        label: "Heading 3",
        group: ShortcutGroup::Format,
        on_mobile: true,
    },
    Shortcut {
        id: "format_heading_4",
        mac: "secondary-alt-4",
        other: "secondary-alt-4",
        context: ShortcutContext::Editor,
        label: "Heading 4",
        group: ShortcutGroup::Format,
        on_mobile: true,
    },
    Shortcut {
        id: "format_heading_5",
        mac: "secondary-alt-5",
        other: "secondary-alt-5",
        context: ShortcutContext::Editor,
        label: "Heading 5",
        group: ShortcutGroup::Format,
        on_mobile: true,
    },
    Shortcut {
        id: "format_heading_6",
        mac: "secondary-alt-6",
        other: "secondary-alt-6",
        context: ShortcutContext::Editor,
        label: "Heading 6",
        group: ShortcutGroup::Format,
        on_mobile: true,
    },
    Shortcut {
        id: "format_task",
        mac: "secondary-alt-9",
        other: "secondary-alt-9",
        context: ShortcutContext::Editor,
        label: "Task",
        group: ShortcutGroup::Format,
        on_mobile: true,
    },
    Shortcut {
        id: "format_bulleted",
        mac: "secondary-alt-8",
        other: "secondary-alt-8",
        context: ShortcutContext::Editor,
        label: "Bulleted List",
        group: ShortcutGroup::Format,
        on_mobile: true,
    },
    Shortcut {
        id: "format_numbered",
        mac: "secondary-alt-7",
        other: "secondary-alt-7",
        context: ShortcutContext::Editor,
        label: "Numbered List",
        group: ShortcutGroup::Format,
        on_mobile: true,
    },
    Shortcut {
        id: "format_quote",
        mac: "secondary-shift-.",
        other: "secondary-shift-.",
        context: ShortcutContext::Editor,
        label: "Quote",
        group: ShortcutGroup::Format,
        on_mobile: true,
    },
    Shortcut {
        id: "format_code_block",
        mac: "secondary-alt-c",
        other: "secondary-alt-c",
        context: ShortcutContext::Editor,
        label: "Code Block",
        group: ShortcutGroup::Format,
        on_mobile: true,
    },
    Shortcut {
        id: "toggle_pinned",
        mac: "secondary-shift-p",
        other: "secondary-shift-p",
        context: ShortcutContext::Global,
        label: "Pin / Unpin Note",
        group: ShortcutGroup::Note,
        on_mobile: true,
    },
    Shortcut {
        id: "dictate",
        mac: "secondary-shift-m",
        other: "secondary-shift-m",
        context: ShortcutContext::Global,
        label: "Voice Input",
        group: ShortcutGroup::Note,
        on_mobile: true,
    },
    Shortcut {
        id: "open_in_new_window",
        mac: "secondary-shift-o",
        other: "secondary-shift-o",
        context: ShortcutContext::Global,
        label: "Open in New Window",
        group: ShortcutGroup::Window,
        on_mobile: true,
    },
    Shortcut {
        id: "close_note_window",
        mac: "secondary-w",
        other: "secondary-w",
        context: ShortcutContext::NoteWindow,
        label: "Close Window",
        group: ShortcutGroup::Window,
        on_mobile: true,
    },
    Shortcut {
        id: "show_editor",
        mac: "secondary-1",
        other: "secondary-1",
        context: ShortcutContext::Global,
        label: "Editor Only",
        group: ShortcutGroup::View,
        on_mobile: true,
    },
    Shortcut {
        id: "show_split",
        mac: "secondary-2",
        other: "secondary-2",
        context: ShortcutContext::Global,
        label: "Editor and Preview",
        group: ShortcutGroup::View,
        on_mobile: true,
    },
    Shortcut {
        id: "show_preview",
        mac: "secondary-3",
        other: "secondary-3",
        context: ShortcutContext::Global,
        label: "Preview Only",
        group: ShortcutGroup::View,
        on_mobile: true,
    },
    Shortcut {
        id: "toggle_sidebar",
        mac: "secondary-\\",
        other: "secondary-\\",
        context: ShortcutContext::Global,
        label: "Toggle Sidebar",
        group: ShortcutGroup::View,
        on_mobile: true,
    },
    Shortcut {
        id: "toggle_inspector",
        mac: "secondary-alt-i",
        other: "secondary-alt-i",
        context: ShortcutContext::Global,
        label: "Outline and Backlinks",
        group: ShortcutGroup::View,
        on_mobile: true,
    },
    Shortcut {
        id: "toggle_focus_mode",
        mac: "secondary-shift-d",
        other: "secondary-shift-d",
        context: ShortcutContext::Global,
        label: "Focus Mode",
        group: ShortcutGroup::View,
        on_mobile: true,
    },
    Shortcut {
        id: "exit_focus_mode",
        mac: "escape",
        other: "escape",
        context: ShortcutContext::FocusMode,
        label: "Leave Focus Mode",
        group: ShortcutGroup::View,
        on_mobile: true,
    },
    Shortcut {
        id: "ask_your_notes",
        mac: "secondary-shift-a",
        other: "secondary-shift-a",
        context: ShortcutContext::Global,
        label: "Ask Your Notes",
        group: ShortcutGroup::View,
        on_mobile: true,
    },
    Shortcut {
        id: "search_all_notes",
        mac: "secondary-shift-f",
        other: "secondary-shift-f",
        context: ShortcutContext::Global,
        label: "Search All Notes",
        group: ShortcutGroup::View,
        on_mobile: true,
    },
    Shortcut {
        id: "replace_in_notes",
        mac: "secondary-shift-h",
        other: "secondary-shift-h",
        context: ShortcutContext::Global,
        label: "Replace in All Notes",
        group: ShortcutGroup::View,
        on_mobile: true,
    },
    Shortcut {
        id: "focus_search",
        mac: "secondary-l",
        other: "secondary-l",
        context: ShortcutContext::Global,
        label: "Search This List",
        group: ShortcutGroup::View,
        on_mobile: true,
    },
    Shortcut {
        id: "focus_note_list",
        mac: "secondary-shift-l",
        other: "secondary-shift-l",
        context: ShortcutContext::Global,
        label: "Go to Note List",
        group: ShortcutGroup::View,
        on_mobile: true,
    },
    Shortcut {
        id: "zoom_in",
        mac: "secondary-=",
        other: "secondary-=",
        context: ShortcutContext::Global,
        label: "Zoom In",
        group: ShortcutGroup::View,
        on_mobile: true,
    },
    Shortcut {
        id: "zoom_in",
        mac: "secondary-+",
        other: "secondary-+",
        context: ShortcutContext::Global,
        label: "Zoom In",
        group: ShortcutGroup::View,
        on_mobile: true,
    },
    Shortcut {
        id: "zoom_out",
        mac: "secondary--",
        other: "secondary--",
        context: ShortcutContext::Global,
        label: "Zoom Out",
        group: ShortcutGroup::View,
        on_mobile: true,
    },
    Shortcut {
        id: "actual_size",
        mac: "secondary-0",
        other: "secondary-0",
        context: ShortcutContext::Global,
        label: "Actual Size",
        group: ShortcutGroup::View,
        on_mobile: true,
    },
    Shortcut {
        id: "go_back",
        mac: "cmd-[",
        other: "alt-left",
        context: ShortcutContext::Global,
        label: "Go Back",
        group: ShortcutGroup::Go,
        on_mobile: true,
    },
    Shortcut {
        id: "go_forward",
        mac: "cmd-]",
        other: "alt-right",
        context: ShortcutContext::Global,
        label: "Go Forward",
        group: ShortcutGroup::Go,
        on_mobile: true,
    },
    Shortcut {
        id: "go_back",
        mac: "cmd-[",
        other: "",
        context: ShortcutContext::Editor,
        label: "Go Back",
        group: ShortcutGroup::Go,
        on_mobile: true,
    },
    Shortcut {
        id: "go_forward",
        mac: "cmd-]",
        other: "",
        context: ShortcutContext::Editor,
        label: "Go Forward",
        group: ShortcutGroup::Go,
        on_mobile: true,
    },
    Shortcut {
        id: "next_note",
        mac: "secondary-alt-down",
        other: "secondary-alt-down",
        context: ShortcutContext::Global,
        label: "Next Note",
        group: ShortcutGroup::Go,
        on_mobile: true,
    },
    Shortcut {
        id: "previous_note",
        mac: "secondary-alt-up",
        other: "secondary-alt-up",
        context: ShortcutContext::Global,
        label: "Previous Note",
        group: ShortcutGroup::Go,
        on_mobile: true,
    },
    Shortcut {
        id: "open_command_palette",
        mac: "secondary-k",
        other: "secondary-k",
        context: ShortcutContext::Global,
        label: "Command Palette",
        group: ShortcutGroup::Go,
        on_mobile: true,
    },
    Shortcut {
        id: "open_note_picker",
        mac: "secondary-p",
        other: "secondary-p",
        context: ShortcutContext::Global,
        label: "Quick Open",
        group: ShortcutGroup::Go,
        on_mobile: true,
    },
    Shortcut {
        id: "close_tab",
        mac: "secondary-w",
        other: "secondary-w",
        context: ShortcutContext::Global,
        label: "Close Tab",
        group: ShortcutGroup::Tabs,
        on_mobile: true,
    },
    Shortcut {
        id: "next_tab",
        mac: "ctrl-tab",
        other: "ctrl-tab",
        context: ShortcutContext::Global,
        label: "Next Tab",
        group: ShortcutGroup::Tabs,
        on_mobile: true,
    },
    Shortcut {
        id: "previous_tab",
        mac: "ctrl-shift-tab",
        other: "ctrl-shift-tab",
        context: ShortcutContext::Global,
        label: "Previous Tab",
        group: ShortcutGroup::Tabs,
        on_mobile: true,
    },
    Shortcut {
        id: "reopen_closed_tab",
        mac: "secondary-shift-t",
        other: "secondary-shift-t",
        context: ShortcutContext::Global,
        label: "Reopen Closed Tab",
        group: ShortcutGroup::Tabs,
        on_mobile: true,
    },
    Shortcut {
        id: "go_to_tab_1",
        mac: "secondary-shift-1",
        other: "secondary-shift-1",
        context: ShortcutContext::Global,
        label: "Go to Tab 1",
        group: ShortcutGroup::Tabs,
        on_mobile: true,
    },
    Shortcut {
        id: "go_to_tab_2",
        mac: "secondary-shift-2",
        other: "secondary-shift-2",
        context: ShortcutContext::Global,
        label: "Go to Tab 2",
        group: ShortcutGroup::Tabs,
        on_mobile: true,
    },
    Shortcut {
        id: "go_to_tab_3",
        mac: "secondary-shift-3",
        other: "secondary-shift-3",
        context: ShortcutContext::Global,
        label: "Go to Tab 3",
        group: ShortcutGroup::Tabs,
        on_mobile: true,
    },
    Shortcut {
        id: "go_to_tab_4",
        mac: "secondary-shift-4",
        other: "secondary-shift-4",
        context: ShortcutContext::Global,
        label: "Go to Tab 4",
        group: ShortcutGroup::Tabs,
        on_mobile: true,
    },
    Shortcut {
        id: "go_to_tab_5",
        mac: "secondary-shift-5",
        other: "secondary-shift-5",
        context: ShortcutContext::Global,
        label: "Go to Tab 5",
        group: ShortcutGroup::Tabs,
        on_mobile: true,
    },
    Shortcut {
        id: "go_to_tab_6",
        mac: "secondary-shift-6",
        other: "secondary-shift-6",
        context: ShortcutContext::Global,
        label: "Go to Tab 6",
        group: ShortcutGroup::Tabs,
        on_mobile: true,
    },
    Shortcut {
        id: "go_to_tab_7",
        mac: "secondary-shift-7",
        other: "secondary-shift-7",
        context: ShortcutContext::Global,
        label: "Go to Tab 7",
        group: ShortcutGroup::Tabs,
        on_mobile: true,
    },
    Shortcut {
        id: "go_to_tab_8",
        mac: "secondary-shift-8",
        other: "secondary-shift-8",
        context: ShortcutContext::Global,
        label: "Go to Tab 8",
        group: ShortcutGroup::Tabs,
        on_mobile: true,
    },
    Shortcut {
        id: "go_to_tab_9",
        mac: "secondary-shift-9",
        other: "secondary-shift-9",
        context: ShortcutContext::Global,
        label: "Go to Tab 9",
        group: ShortcutGroup::Tabs,
        on_mobile: true,
    },
    Shortcut {
        id: "open_settings",
        mac: "secondary-,",
        other: "secondary-,",
        context: ShortcutContext::Global,
        label: "Settings",
        group: ShortcutGroup::App,
        on_mobile: true,
    },
    Shortcut {
        id: "quit",
        mac: "secondary-q",
        other: "secondary-q",
        context: ShortcutContext::Global,
        label: "Quit Kanso",
        group: ShortcutGroup::App,
        on_mobile: false,
    },
    Shortcut {
        id: "focus_editor",
        mac: "enter",
        other: "enter",
        context: ShortcutContext::NoteList,
        label: "Open Note in Editor",
        group: ShortcutGroup::List,
        on_mobile: true,
    },
    Shortcut {
        id: "select_next",
        mac: "down",
        other: "down",
        context: ShortcutContext::NoteList,
        label: "Next Note",
        group: ShortcutGroup::List,
        on_mobile: true,
    },
    Shortcut {
        id: "select_next",
        mac: "j",
        other: "j",
        context: ShortcutContext::NoteList,
        label: "Next Note",
        group: ShortcutGroup::List,
        on_mobile: true,
    },
    Shortcut {
        id: "select_previous",
        mac: "up",
        other: "up",
        context: ShortcutContext::NoteList,
        label: "Previous Note",
        group: ShortcutGroup::List,
        on_mobile: true,
    },
    Shortcut {
        id: "select_previous",
        mac: "k",
        other: "k",
        context: ShortcutContext::NoteList,
        label: "Previous Note",
        group: ShortcutGroup::List,
        on_mobile: true,
    },
    Shortcut {
        id: "select_all_notes",
        mac: "secondary-a",
        other: "secondary-a",
        context: ShortcutContext::NoteList,
        label: "Select All Notes",
        group: ShortcutGroup::List,
        on_mobile: true,
    },
    Shortcut {
        id: "open_focused_in_new_tab",
        mac: "secondary-enter",
        other: "secondary-enter",
        context: ShortcutContext::NoteList,
        label: "Open in New Tab",
        group: ShortcutGroup::List,
        on_mobile: true,
    },
    Shortcut {
        id: "select_one_note",
        mac: "escape",
        other: "escape",
        context: ShortcutContext::NoteList,
        label: "Select Only This Note",
        group: ShortcutGroup::List,
        on_mobile: true,
    },
    Shortcut {
        id: "select_next",
        mac: "down",
        other: "down",
        context: ShortcutContext::Palette,
        label: "Next Result",
        group: ShortcutGroup::Palette,
        on_mobile: true,
    },
    Shortcut {
        id: "select_previous",
        mac: "up",
        other: "up",
        context: ShortcutContext::Palette,
        label: "Previous Result",
        group: ShortcutGroup::Palette,
        on_mobile: true,
    },
    Shortcut {
        id: "confirm",
        mac: "enter",
        other: "enter",
        context: ShortcutContext::Palette,
        label: "Choose",
        group: ShortcutGroup::Palette,
        on_mobile: true,
    },
    Shortcut {
        id: "dismiss",
        mac: "escape",
        other: "escape",
        context: ShortcutContext::Palette,
        label: "Close Palette",
        group: ShortcutGroup::Palette,
        on_mobile: true,
    },
    Shortcut {
        id: "back",
        mac: "backspace",
        other: "backspace",
        context: ShortcutContext::Palette,
        label: "Back",
        group: ShortcutGroup::Palette,
        on_mobile: true,
    },
    Shortcut {
        id: "previous_day_cell",
        mac: "left",
        other: "left",
        context: ShortcutContext::CalendarView,
        label: "Previous Day",
        group: ShortcutGroup::Calendar,
        on_mobile: true,
    },
    Shortcut {
        id: "next_day_cell",
        mac: "right",
        other: "right",
        context: ShortcutContext::CalendarView,
        label: "Next Day",
        group: ShortcutGroup::Calendar,
        on_mobile: true,
    },
    Shortcut {
        id: "previous_week",
        mac: "up",
        other: "up",
        context: ShortcutContext::CalendarView,
        label: "Previous Week",
        group: ShortcutGroup::Calendar,
        on_mobile: true,
    },
    Shortcut {
        id: "next_week",
        mac: "down",
        other: "down",
        context: ShortcutContext::CalendarView,
        label: "Next Week",
        group: ShortcutGroup::Calendar,
        on_mobile: true,
    },
    Shortcut {
        id: "previous_month",
        mac: "pageup",
        other: "pageup",
        context: ShortcutContext::CalendarView,
        label: "Previous Month",
        group: ShortcutGroup::Calendar,
        on_mobile: true,
    },
    Shortcut {
        id: "next_month",
        mac: "pagedown",
        other: "pagedown",
        context: ShortcutContext::CalendarView,
        label: "Next Month",
        group: ShortcutGroup::Calendar,
        on_mobile: true,
    },
    Shortcut {
        id: "go_to_today",
        mac: "t",
        other: "t",
        context: ShortcutContext::CalendarView,
        label: "Go to Today",
        group: ShortcutGroup::Calendar,
        on_mobile: true,
    },
    Shortcut {
        id: "open_day",
        mac: "enter",
        other: "enter",
        context: ShortcutContext::CalendarView,
        label: "Open Day",
        group: ShortcutGroup::Calendar,
        on_mobile: true,
    },
    Shortcut {
        id: "select_next_task",
        mac: "down",
        other: "down",
        context: ShortcutContext::TasksView,
        label: "Next Task",
        group: ShortcutGroup::Tasks,
        on_mobile: true,
    },
    Shortcut {
        id: "select_next_task",
        mac: "j",
        other: "j",
        context: ShortcutContext::TasksView,
        label: "Next Task",
        group: ShortcutGroup::Tasks,
        on_mobile: true,
    },
    Shortcut {
        id: "select_previous_task",
        mac: "up",
        other: "up",
        context: ShortcutContext::TasksView,
        label: "Previous Task",
        group: ShortcutGroup::Tasks,
        on_mobile: true,
    },
    Shortcut {
        id: "select_previous_task",
        mac: "k",
        other: "k",
        context: ShortcutContext::TasksView,
        label: "Previous Task",
        group: ShortcutGroup::Tasks,
        on_mobile: true,
    },
    Shortcut {
        id: "toggle_task",
        mac: "space",
        other: "space",
        context: ShortcutContext::TasksView,
        label: "Check Off Task",
        group: ShortcutGroup::Tasks,
        on_mobile: true,
    },
    Shortcut {
        id: "toggle_task",
        mac: "x",
        other: "x",
        context: ShortcutContext::TasksView,
        label: "Check Off Task",
        group: ShortcutGroup::Tasks,
        on_mobile: true,
    },
    Shortcut {
        id: "open_task",
        mac: "enter",
        other: "enter",
        context: ShortcutContext::TasksView,
        label: "Open Task",
        group: ShortcutGroup::Tasks,
        on_mobile: true,
    },
    Shortcut {
        id: "undo_check_off",
        mac: "secondary-z",
        other: "secondary-z",
        context: ShortcutContext::TasksView,
        label: "Undo Check-Off",
        group: ShortcutGroup::Tasks,
        on_mobile: true,
    },
    Shortcut {
        id: "close_settings",
        mac: "escape",
        other: "escape",
        context: ShortcutContext::Settings,
        label: "Close Settings",
        group: ShortcutGroup::Settings,
        on_mobile: true,
    },
    Shortcut {
        id: "next_settings_page",
        mac: "down",
        other: "down",
        context: ShortcutContext::SettingsNav,
        label: "Next Page",
        group: ShortcutGroup::Settings,
        on_mobile: true,
    },
    Shortcut {
        id: "previous_settings_page",
        mac: "up",
        other: "up",
        context: ShortcutContext::SettingsNav,
        label: "Previous Page",
        group: ShortcutGroup::Settings,
        on_mobile: true,
    },
    Shortcut {
        id: "close_graph",
        mac: "escape",
        other: "escape",
        context: ShortcutContext::GraphView,
        label: "Close Graph",
        group: ShortcutGroup::Graph,
        on_mobile: true,
    },
    Shortcut {
        id: "graph_zoom_in",
        mac: "secondary-=",
        other: "secondary-=",
        context: ShortcutContext::GraphView,
        label: "Zoom In",
        group: ShortcutGroup::Graph,
        on_mobile: true,
    },
    Shortcut {
        id: "graph_zoom_in",
        mac: "secondary-+",
        other: "secondary-+",
        context: ShortcutContext::GraphView,
        label: "Zoom In",
        group: ShortcutGroup::Graph,
        on_mobile: true,
    },
    Shortcut {
        id: "graph_zoom_out",
        mac: "secondary--",
        other: "secondary--",
        context: ShortcutContext::GraphView,
        label: "Zoom Out",
        group: ShortcutGroup::Graph,
        on_mobile: true,
    },
    Shortcut {
        id: "graph_show_all",
        mac: "secondary-0",
        other: "secondary-0",
        context: ShortcutContext::GraphView,
        label: "Show All",
        group: ShortcutGroup::Graph,
        on_mobile: true,
    },
    Shortcut {
        id: "select_card_up",
        mac: "up",
        other: "up",
        context: ShortcutContext::BoardView,
        label: "Card Above",
        group: ShortcutGroup::Board,
        on_mobile: true,
    },
    Shortcut {
        id: "select_card_up",
        mac: "k",
        other: "k",
        context: ShortcutContext::BoardView,
        label: "Card Above",
        group: ShortcutGroup::Board,
        on_mobile: true,
    },
    Shortcut {
        id: "select_card_down",
        mac: "down",
        other: "down",
        context: ShortcutContext::BoardView,
        label: "Card Below",
        group: ShortcutGroup::Board,
        on_mobile: true,
    },
    Shortcut {
        id: "select_card_down",
        mac: "j",
        other: "j",
        context: ShortcutContext::BoardView,
        label: "Card Below",
        group: ShortcutGroup::Board,
        on_mobile: true,
    },
    Shortcut {
        id: "select_card_left",
        mac: "left",
        other: "left",
        context: ShortcutContext::BoardView,
        label: "Card to the Left",
        group: ShortcutGroup::Board,
        on_mobile: true,
    },
    Shortcut {
        id: "select_card_left",
        mac: "h",
        other: "h",
        context: ShortcutContext::BoardView,
        label: "Card to the Left",
        group: ShortcutGroup::Board,
        on_mobile: true,
    },
    Shortcut {
        id: "select_card_right",
        mac: "right",
        other: "right",
        context: ShortcutContext::BoardView,
        label: "Card to the Right",
        group: ShortcutGroup::Board,
        on_mobile: true,
    },
    Shortcut {
        id: "select_card_right",
        mac: "l",
        other: "l",
        context: ShortcutContext::BoardView,
        label: "Card to the Right",
        group: ShortcutGroup::Board,
        on_mobile: true,
    },
    Shortcut {
        id: "move_card_left",
        mac: "shift-left",
        other: "shift-left",
        context: ShortcutContext::BoardView,
        label: "Move Card Left",
        group: ShortcutGroup::Board,
        on_mobile: true,
    },
    Shortcut {
        id: "move_card_left",
        mac: "shift-h",
        other: "shift-h",
        context: ShortcutContext::BoardView,
        label: "Move Card Left",
        group: ShortcutGroup::Board,
        on_mobile: true,
    },
    Shortcut {
        id: "move_card_right",
        mac: "shift-right",
        other: "shift-right",
        context: ShortcutContext::BoardView,
        label: "Move Card Right",
        group: ShortcutGroup::Board,
        on_mobile: true,
    },
    Shortcut {
        id: "move_card_right",
        mac: "shift-l",
        other: "shift-l",
        context: ShortcutContext::BoardView,
        label: "Move Card Right",
        group: ShortcutGroup::Board,
        on_mobile: true,
    },
    Shortcut {
        id: "open_card",
        mac: "enter",
        other: "enter",
        context: ShortcutContext::BoardView,
        label: "Open Card",
        group: ShortcutGroup::Board,
        on_mobile: true,
    },
    Shortcut {
        id: "close_board",
        mac: "escape",
        other: "escape",
        context: ShortcutContext::BoardView,
        label: "Close Board",
        group: ShortcutGroup::Board,
        on_mobile: true,
    },
    Shortcut {
        id: "save_capture",
        mac: "secondary-enter",
        other: "secondary-enter",
        context: ShortcutContext::Capture,
        label: "Save Capture",
        group: ShortcutGroup::Capture,
        on_mobile: false,
    },
    Shortcut {
        id: "save_capture",
        mac: "secondary-s",
        other: "secondary-s",
        context: ShortcutContext::Capture,
        label: "Save Capture",
        group: ShortcutGroup::Capture,
        on_mobile: false,
    },
    Shortcut {
        id: "cancel_capture",
        mac: "escape",
        other: "escape",
        context: ShortcutContext::Capture,
        label: "Cancel Capture",
        group: ShortcutGroup::Capture,
        on_mobile: false,
    },
];

/// The first entry with this `id`, or none.
pub fn find(id: &str) -> Option<&'static Shortcut> {
    SHORTCUTS.iter().find(|shortcut| shortcut.id == id)
}

/// Every entry with this `id`: one per key, and one per context.
pub fn with_id(id: &str) -> impl Iterator<Item = &'static Shortcut> + '_ {
    SHORTCUTS.iter().filter(move |shortcut| shortcut.id == id)
}

/// The entries a phone or tablet binds.
pub fn on_mobile() -> impl Iterator<Item = &'static Shortcut> {
    SHORTCUTS.iter().filter(|shortcut| shortcut.on_mobile)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    use super::*;

    /// No two entries share keys and context, on either system.
    #[test]
    fn no_keys_are_bound_twice_in_a_context() {
        for apple in [true, false] {
            let mut seen = HashSet::new();
            for shortcut in SHORTCUTS {
                let keys = shortcut.keys(apple);
                if keys.is_empty() {
                    continue;
                }
                assert!(
                    seen.insert((keys, shortcut.context)),
                    "{keys} is bound twice in {:?}",
                    shortcut.context
                );
            }
        }
    }

    #[test]
    fn ids_are_snake_case() {
        for shortcut in SHORTCUTS {
            let id = shortcut.id;
            let ok = !id.is_empty()
                && id.split('_').all(|part| {
                    !part.is_empty()
                        && part
                            .chars()
                            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
                });
            assert!(ok, "{id} is not snake case");
        }
    }

    /// The same keys on both systems unless the desktop binds per system.
    #[test]
    fn only_navigation_differs_by_system() {
        for shortcut in SHORTCUTS {
            if shortcut.mac != shortcut.other {
                assert!(
                    matches!(shortcut.id, "go_back" | "go_forward"),
                    "{} differs by system",
                    shortcut.id
                );
            }
        }
    }

    #[test]
    fn every_entry_has_a_label_and_keys() {
        for shortcut in SHORTCUTS {
            assert!(!shortcut.label.is_empty(), "{} has no label", shortcut.id);
            assert!(!shortcut.mac.is_empty(), "{} has no mac keys", shortcut.id);
        }
    }

    #[test]
    fn only_quit_and_capture_stay_off_the_phone() {
        for shortcut in SHORTCUTS {
            let off = shortcut.id == "quit" || shortcut.context == ShortcutContext::Capture;
            assert_eq!(shortcut.on_mobile, !off, "{}", shortcut.id);
        }
    }

    #[test]
    fn all_sixteen_format_keys_are_there() {
        let format = SHORTCUTS
            .iter()
            .filter(|shortcut| shortcut.group == ShortcutGroup::Format)
            .count();
        assert_eq!(format, 16);
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Invariant: looking an id up finds the first entry with that id,
        /// and `with_id` lists exactly the entries that have it. A string
        /// that is not an id finds nothing.
        #[test]
        fn prop_lookup_by_id_is_consistent(ix in 0..SHORTCUTS.len(), junk in "[A-Z ]{1,8}") {
            let id = SHORTCUTS[ix].id;
            let first = find(id).expect("an id in the catalog is found");
            prop_assert_eq!(first.id, id);
            let at = SHORTCUTS.iter().position(|s| s.id == id).unwrap();
            prop_assert!(at <= ix);
            prop_assert_eq!(first, &SHORTCUTS[at]);
            let all: Vec<_> = with_id(id).collect();
            prop_assert!(all.iter().all(|s| s.id == id));
            prop_assert_eq!(all.len(), SHORTCUTS.iter().filter(|s| s.id == id).count());
            prop_assert!(all.contains(&&SHORTCUTS[ix]));
            prop_assert!(find(&junk).is_none());
        }
    }
}
