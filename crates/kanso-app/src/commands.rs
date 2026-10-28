//! What the command palette can do, apart from the places and notes that
//! come from the library (notebooks, statuses, notes). Most commands
//! dispatch the same action as their menu item or shortcut, so the palette
//! shows the shortcut the keymap really has; a few open a second list in
//! the palette to pick a notebook, tag or status from.

use gpui_kit::Action;
use gpui_kit::assets::IconName;

use kanso_ai::InlineAction;

use crate::editor::inline_glue::InlineAssist;
use crate::journal::{NewTemplate, NextDay, OpenToday, PreviousDay, QuickCapture};
use crate::settings::{
    BrowseIconThemes, BrowseThemes, CloseSettings, ImportIconTheme, OpenAccountSettings,
    OpenAiSettings, OpenApiSettings, OpenAppearanceSettings, OpenEditorSettings,
    OpenGeneralSettings, OpenIconThemeSettings, OpenPluginSettings, OpenUpdateSettings,
    RefreshThemeGallery, ReloadIconThemes, ReloadThemes, StopThemePreview,
};
use crate::tabs::{
    CloseTab, GoToTab1, GoToTab2, GoToTab3, GoToTab4, GoToTab5, GoToTab6, GoToTab7, GoToTab8,
    GoToTab9, NextTab, PreviousTab, ReopenClosedTab,
};
use crate::workspace::{
    AskYourNotes, ChangePassword, CopyKansoLink, CopyNoteLink, Dictate, ExportHtml, ExportLibrary,
    ExportNote, ExportPdf, FocusNoteList, FocusSearch, FormatTable, GoBack, GoForward,
    ImportLibrary, NewNote, NewNotebook, NewRecoveryKey, NextNote, OpenFile, OpenNotePicker,
    OpenSettings, PreviousNote, Quit, SaveNote, SearchAllNotes, ShowBacklinks, ShowCreateAccount,
    ShowEditor, ShowHistory, ShowOutline, ShowPreview, ShowSignIn, ShowSplit, SignOut,
    SummarizeNote, SyncNow, ToggleFocusMode, ToggleInspector, TogglePinned, ToggleSidebar,
    TrashNote,
};

/// The palette's sections, in the order they appear with nothing typed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Group {
    Note,
    View,
    GoTo,
    App,
}

impl Group {
    pub fn label(self) -> &'static str {
        match self {
            Group::Note => "Note actions",
            Group::View => "View",
            Group::GoTo => "Go to",
            Group::App => "App",
        }
    }
}

/// A second list the palette shows to choose from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Picker {
    /// The notebook to move the open note into.
    Notebook,
    Tag,
    Status,
    /// A notebook for the note list to show.
    GoToNotebook,
    /// A tag whose notes the list shows.
    GoToTag,
    /// The open note's word goal.
    WordGoal,
    /// A template to make a note from.
    Template,
    /// A width for the image on the cursor's line.
    ImageSize,
    /// The icon theme drawing the app's icons.
    IconTheme,
}

/// What choosing a command does.
#[derive(Clone, Copy, Debug)]
pub enum Run {
    /// Dispatches an action, as its menu item or shortcut would.
    Action(fn() -> Box<dyn Action>),
    /// Opens a list in the palette to pick from.
    Pick(Picker),
}

/// When a command applies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Needs {
    Nothing,
    /// A note is open.
    Note,
    /// A note is open and isn't in the trash.
    LiveNote,
    /// A note is open, isn't in the trash and isn't locked.
    UnlockedNote,
    /// A locked note is open.
    LockedNote,
    /// Settings are open, in place of the notes.
    Settings,
    /// This Mac is signed in to an account.
    SignedIn,
    /// This Mac isn't signed in (or was signed out by the server).
    SignedOut,
    /// The cursor is on a line that is only an image, in the note's text.
    ImageLine,
    /// The cursor is on a line that links to an audio attachment.
    AudioLine,
    /// A shared note this account is in is open.
    SharedNote,
    /// AI is on in Settings › AI.
    AiOn,
    /// AI is on and a note is open that isn't in the trash or locked.
    AiNote,
}

pub struct Command {
    /// Stable and unique, for tests and for remembering the selection.
    pub id: &'static str,
    pub label: &'static str,
    /// The label when the open note already has what the command toggles
    /// (for pinning: "Unpin note").
    pub label_when_pinned: Option<(&'static str, IconName)>,
    /// One or two sentences for the detail pane.
    pub description: &'static str,
    pub icon: IconName,
    /// Other words people might type for it.
    pub keywords: &'static [&'static str],
    pub group: Group,
    pub run: Run,
    pub needs: Needs,
}

impl Command {
    /// The action the command dispatches, if it dispatches one.
    pub fn action(&self) -> Option<Box<dyn Action>> {
        match self.run {
            Run::Action(action) => Some(action()),
            Run::Pick(_) => None,
        }
    }
}

macro_rules! action {
    ($action:expr) => {
        Run::Action(|| Box::new($action))
    };
}

/// Export as PDF: a file on macOS; elsewhere the page opens in the browser
/// to print to PDF.
const PDF_DESCRIPTION: &str = if cfg!(target_os = "macos") {
    "Saves the note as a PDF in the preview's theme, with its math, diagrams and images."
} else {
    "Opens the note as a web page in your browser, in the preview's theme, to print to PDF from there."
};

static COMMANDS: &[Command] = &[
    // Note actions.
    Command {
        id: "new-note",
        label: "New note",
        label_when_pinned: None,
        description: "Creates a note in the current notebook and puts the cursor in its title.",
        icon: IconName::FilePlus,
        keywords: &["create", "add", "write"],
        group: Group::Note,
        run: action!(NewNote),
        needs: Needs::Nothing,
    },
    Command {
        id: "new-notebook",
        label: "New notebook…",
        label_when_pinned: None,
        description: "Creates a notebook. Notes can be moved into it afterwards.",
        icon: IconName::Notebook,
        keywords: &["create", "add", "folder"],
        group: Group::Note,
        run: action!(NewNotebook),
        needs: Needs::Nothing,
    },
    Command {
        id: "search",
        label: "Search all notes",
        label_when_pinned: None,
        description: "Shows all notes and moves to the search box above them, whatever notebook, tag or status was showing. It searches titles and text; book:, tag: and status: narrow it down.",
        icon: IconName::Search,
        keywords: &["find", "global", "everywhere", "query"],
        group: Group::Note,
        run: action!(SearchAllNotes),
        needs: Needs::Nothing,
    },
    Command {
        id: "replace-all",
        label: "Replace in all notes…",
        label_when_pinned: None,
        description: "Finds text in every note and replaces it, with a preview of each match to check first. Match case, whole word, regular expressions and skipping code blocks are options; Undo replace puts the notes back.",
        icon: IconName::Replace,
        keywords: &[
            "find and replace",
            "substitute",
            "rename",
            "bulk",
            "regex",
            "everywhere",
        ],
        group: Group::Note,
        run: action!(crate::replace::ReplaceInNotes),
        needs: Needs::Nothing,
    },
    Command {
        id: "search-list",
        label: "Search this list",
        label_when_pinned: None,
        description: "Moves to the search box above the note list, to search only the notes it shows: a notebook, a tag, a status or the trash.",
        icon: IconName::Search,
        keywords: &["find", "filter", "notebook", "query"],
        group: Group::Note,
        run: action!(FocusSearch),
        needs: Needs::Nothing,
    },
    Command {
        id: "open-note",
        label: "Open note…",
        label_when_pinned: None,
        description: "Finds a note by its title and opens it.",
        icon: IconName::FileText,
        keywords: &["go", "jump", "switch", "find", "title"],
        group: Group::Note,
        run: action!(OpenNotePicker),
        needs: Needs::Nothing,
    },
    Command {
        id: "new-from-template",
        label: "New from template…",
        label_when_pinned: None,
        description: "Makes a note from one in the Templates notebook, with {{date}}, {{time}} and {{title}} filled in.",
        icon: IconName::LayoutTemplate,
        keywords: &["create", "template", "boilerplate", "snippet"],
        group: Group::Note,
        run: Run::Pick(Picker::Template),
        needs: Needs::Nothing,
    },
    Command {
        id: "new-template",
        label: "New template",
        label_when_pinned: None,
        description: "Creates a note in the Templates notebook. Write {{date}}, {{time}}, {{title}}, {{date:%A %e %B}} or {{cursor}} where the new note should have them.",
        icon: IconName::LayoutTemplate,
        keywords: &["create", "template", "add"],
        group: Group::Note,
        run: action!(NewTemplate),
        needs: Needs::Nothing,
    },
    Command {
        id: "quick-capture",
        label: "Quick note",
        label_when_pinned: None,
        description: "Opens the small window for jotting something down, which saves to the Inbox. Its shortcut works from any app; Settings › General sets it.",
        icon: IconName::Zap,
        keywords: &["capture", "jot", "inbox", "scratch"],
        group: Group::Note,
        run: action!(QuickCapture),
        needs: Needs::Nothing,
    },
    Command {
        id: "pin",
        label: "Pin note",
        label_when_pinned: Some(("Unpin note", IconName::PinOff)),
        description: "Keeps the note at the top of every list, and under Pinned in the sidebar. With several notes selected in the list, pins them all.",
        icon: IconName::Pin,
        keywords: &["pin", "unpin", "star", "favorite", "top"],
        group: Group::Note,
        run: action!(TogglePinned),
        needs: Needs::LiveNote,
    },
    Command {
        id: "open-in-new-window",
        label: "Open in new window",
        label_when_pinned: None,
        description: "Opens the note in a window of its own, beside this one. Typing in either shows in the other at once.",
        icon: IconName::ExternalLink,
        keywords: &["window", "separate", "detach", "pop out", "float"],
        group: Group::Note,
        run: action!(crate::note_window::OpenInNewWindow),
        needs: Needs::Note,
    },
    Command {
        id: "move",
        label: "Move to notebook…",
        label_when_pinned: None,
        description: "Moves the note into another notebook. A note in the trash comes back out. With several notes selected in the list, moves them all.",
        icon: IconName::NotebookText,
        keywords: &["notebook", "folder", "file", "restore"],
        group: Group::Note,
        run: Run::Pick(Picker::Notebook),
        needs: Needs::Note,
    },
    Command {
        id: "add-tag",
        label: "Add tag…",
        label_when_pinned: None,
        description: "Adds a tag to the note, or takes one off. Typing a new name creates the tag. With several notes selected in the list, tags them all.",
        icon: IconName::Tag,
        keywords: &["tags", "label", "hashtag", "remove tag"],
        group: Group::Note,
        run: Run::Pick(Picker::Tag),
        needs: Needs::LiveNote,
    },
    Command {
        id: "status",
        label: "Set status…",
        label_when_pinned: None,
        description: "Marks the note Active, On hold, Completed or Dropped, or clears its status. With several notes selected in the list, marks them all.",
        icon: IconName::CircleDot,
        keywords: &["active", "on hold", "completed", "dropped", "done"],
        group: Group::Note,
        run: Run::Pick(Picker::Status),
        needs: Needs::LiveNote,
    },
    Command {
        id: "format-table",
        label: "Format table",
        label_when_pinned: None,
        description: "Lines up the pipes of the table at the cursor, padding each cell to its column's width. The cells' text stays as it is.",
        icon: IconName::Table,
        keywords: &["align", "table", "columns", "pipes", "tidy"],
        group: Group::Note,
        run: action!(FormatTable),
        needs: Needs::LiveNote,
    },
    Command {
        id: "new-drawing",
        label: "New drawing",
        label_when_pinned: None,
        description: "Puts an empty Excalidraw drawing in the note at the cursor. Edit in Excalidraw, from the drawing in the preview, draws in it.",
        icon: IconName::PenTool,
        keywords: &[
            "excalidraw",
            "sketch",
            "diagram",
            "draw",
            "whiteboard",
            "insert",
        ],
        group: Group::Note,
        run: action!(crate::workspace::NewDrawing),
        needs: Needs::LiveNote,
    },
    Command {
        id: "resize-image",
        label: "Resize image…",
        label_when_pinned: None,
        description: "Sets how wide the image on the cursor's line shows: small, medium, its own size, or a width you type.",
        icon: IconName::Image,
        keywords: &["image", "picture", "width", "size", "smaller", "bigger"],
        group: Group::Note,
        run: Run::Pick(Picker::ImageSize),
        needs: Needs::ImageLine,
    },
    Command {
        id: "transcribe-recording",
        label: "Transcribe recording",
        label_when_pinned: None,
        description: "Turns the recording linked on the cursor's line into text, on this device, and puts it under the recording.",
        icon: IconName::Mic,
        keywords: &[
            "audio",
            "voice",
            "memo",
            "speech",
            "text",
            "transcript",
            "dictation",
        ],
        group: Group::Note,
        run: action!(crate::transcribe::TranscribeAudio),
        needs: Needs::AudioLine,
    },
    Command {
        id: "word-goal",
        label: "Set word goal…",
        label_when_pinned: None,
        description: "How many words this note is aiming for. Focus mode shows how far along it is.",
        icon: IconName::Goal,
        keywords: &["target", "count", "words", "progress", "focus"],
        group: Group::Note,
        run: Run::Pick(Picker::WordGoal),
        needs: Needs::LiveNote,
    },
    Command {
        id: "voice-input",
        label: "Voice input",
        label_when_pinned: None,
        description: "Starts listening; again writes what you said at the cursor. Esc drops it.",
        icon: IconName::Mic,
        keywords: &[
            "dictate",
            "dictation",
            "speak",
            "speech",
            "microphone",
            "transcribe",
        ],
        group: Group::Note,
        run: action!(Dictate),
        needs: Needs::LiveNote,
    },
    Command {
        id: "history",
        label: "Show history…",
        label_when_pinned: None,
        description: "Lists the earlier versions of the note. Any of them can be restored; the current text is kept too.",
        icon: IconName::Clock,
        keywords: &["versions", "revisions", "restore", "undo"],
        group: Group::Note,
        run: action!(ShowHistory),
        needs: Needs::Note,
    },
    Command {
        id: "export-note",
        label: "Export note…",
        label_when_pinned: None,
        description: "Saves the note as a Markdown file, with its attachments, in a folder you choose.",
        icon: IconName::ExternalLink,
        keywords: &["markdown", "save as", "file", "share"],
        group: Group::Note,
        run: action!(ExportNote),
        needs: Needs::Note,
    },
    Command {
        id: "export-html",
        label: "Export as web page…",
        label_when_pinned: None,
        description: "Saves the note as one HTML file in the preview's theme, with its math, diagrams and images inside it.",
        icon: IconName::Globe,
        keywords: &["html", "page", "share", "save as", "file"],
        group: Group::Note,
        run: action!(ExportHtml),
        needs: Needs::Note,
    },
    Command {
        id: "copy-note-link",
        label: "Copy note link",
        label_when_pinned: None,
        description: "Copies the note's [[Title]] link, to paste into another note. A title a link can't hold, or one an older note has too, copies the Kanso link instead.",
        icon: IconName::Link,
        keywords: &["wiki", "wikilink", "reference", "clipboard", "url"],
        group: Group::Note,
        run: action!(CopyNoteLink),
        needs: Needs::Note,
    },
    Command {
        id: "copy-kanso-link",
        label: "Copy Kanso link",
        label_when_pinned: None,
        description: "Copies a link to the note by its ID, which keeps working when the note is renamed.",
        icon: IconName::Copy,
        keywords: &["id", "reference", "clipboard", "url", "markdown"],
        group: Group::Note,
        run: action!(CopyKansoLink),
        needs: Needs::Note,
    },
    Command {
        id: "share",
        label: "Share…",
        label_when_pinned: None,
        description: "Invites people to read, comment on or edit the note with you, end-to-end encrypted: your sync server relays it without reading it. Locked notes can't be shared.",
        icon: IconName::Users,
        keywords: &[
            "share",
            "collaborate",
            "invite",
            "members",
            "together",
            "link",
        ],
        group: Group::Note,
        run: action!(crate::collab::ShareNote),
        needs: Needs::UnlockedNote,
    },
    Command {
        id: "join-shared-note",
        label: "Join a shared note…",
        label_when_pinned: None,
        description: "Asks to join a note from an invite link someone sent you. The note shows in Shared with me once its owner lets you in.",
        icon: IconName::Link2,
        keywords: &["join", "invite", "link", "shared", "collaborate"],
        group: Group::Note,
        run: action!(crate::collab::JoinSharedNote),
        needs: Needs::SignedIn,
    },
    Command {
        id: "comment",
        label: "Comment on selection…",
        label_when_pinned: None,
        description: "Comments on the selected words of a shared note. Write @ and a member's username to mention them.",
        icon: IconName::MessageSquarePlus,
        keywords: &["comment", "note", "remark", "mention", "review"],
        group: Group::Note,
        run: action!(crate::collab::CommentOnSelection),
        needs: Needs::SharedNote,
    },
    Command {
        id: "show-comments",
        label: "Show comments",
        label_when_pinned: None,
        description: "Lists the shared note's comments in the inspector, in the order of the words they are on.",
        icon: IconName::MessageSquare,
        keywords: &["comments", "mentions", "resolve", "review", "inspector"],
        group: Group::View,
        run: action!(crate::collab::ShowComments),
        needs: Needs::SharedNote,
    },
    Command {
        id: "publish",
        label: "Publish to the web…",
        label_when_pinned: None,
        description: "Puts the note at a link anyone can open. Your sync server keeps only an encrypted copy it can't read.",
        icon: IconName::Globe,
        keywords: &["publish", "share", "link", "web", "unpublish", "public"],
        group: Group::Note,
        run: action!(crate::publish::PublishNote),
        needs: Needs::Note,
    },
    Command {
        id: "publish-notebook",
        label: "Publish notebook…",
        label_when_pinned: None,
        description: "Puts the notebook the list shows at one link, as a small site with a page for each note and links between them. Locked notes stay out.",
        icon: IconName::Globe,
        keywords: &[
            "publish",
            "notebook",
            "site",
            "website",
            "share",
            "link",
            "web",
            "unpublish",
        ],
        group: Group::Note,
        run: action!(crate::publish_site::PublishNotebook),
        needs: Needs::Nothing,
    },
    Command {
        id: "lock",
        label: "Lock note…",
        label_when_pinned: None,
        description: "Keeps the note's text sealed with your lock password, on this device and in sync. Its earlier versions are deleted; search, backlinks and tasks skip it while it's locked.",
        icon: IconName::Lock,
        keywords: &["lock", "password", "encrypt", "private", "secret", "hide"],
        group: Group::Note,
        run: action!(crate::locked::LockNote),
        needs: Needs::UnlockedNote,
    },
    Command {
        id: "remove-lock",
        label: "Remove lock…",
        label_when_pinned: None,
        description: "Keeps the note's text as plain text again, so search, backlinks and tasks see it.",
        icon: IconName::LockOpen,
        keywords: &["unlock", "lock", "password", "decrypt"],
        group: Group::Note,
        run: action!(crate::locked::RemoveLock),
        needs: Needs::LockedNote,
    },
    Command {
        id: "lock-now",
        label: "Lock notes now",
        label_when_pinned: None,
        description: "Closes every locked note until the lock password is typed again. Kanso also does this 10 minutes after they were unlocked, or after you last edited one.",
        icon: IconName::LockKeyhole,
        keywords: &["lock", "close", "hide", "away"],
        group: Group::Note,
        run: action!(crate::locked::LockNotesNow),
        needs: Needs::Nothing,
    },
    Command {
        id: "lock-password",
        label: "Change lock password…",
        label_when_pinned: None,
        description: "Seals every locked note again with a new lock password.",
        icon: IconName::LockKeyhole,
        keywords: &["lock", "password", "change", "rekey"],
        group: Group::Note,
        run: action!(crate::locked::ChangeLockPassword),
        needs: Needs::Nothing,
    },
    Command {
        id: "touch-id-on",
        label: concat!("Unlock locked notes with ", crate::touch_id::unlock_name!()),
        label_when_pinned: None,
        description: concat!(
            "Once ",
            crate::touch_id::unlock_name!(),
            " says it's you, keeps the lock keys in this device's ",
            crate::touch_id::secret_store_name!(),
            " so it opens locked notes here. The lock password still works, and other devices need it."
        ),
        icon: IconName::FingerprintPattern,
        keywords: crate::touch_id::KEYWORDS,
        group: Group::Note,
        run: action!(crate::locked::TurnOnTouchId),
        needs: Needs::Nothing,
    },
    Command {
        id: "touch-id-off",
        label: concat!(
            "Stop using ",
            crate::touch_id::unlock_name!(),
            " for locked notes"
        ),
        label_when_pinned: None,
        description: concat!(
            "Forgets the lock keys kept for ",
            crate::touch_id::unlock_name!(),
            " on this device; locked notes open with the lock password again."
        ),
        icon: IconName::FingerprintPattern,
        keywords: crate::touch_id::KEYWORDS,
        group: Group::Note,
        run: action!(crate::locked::TurnOffTouchId),
        needs: Needs::Nothing,
    },
    Command {
        id: "export-pdf",
        label: "Export as PDF…",
        label_when_pinned: None,
        description: PDF_DESCRIPTION,
        icon: IconName::File,
        keywords: &["pdf", "print", "share", "save as", "file"],
        group: Group::Note,
        run: action!(ExportPdf),
        needs: Needs::Note,
    },
    Command {
        id: "save",
        label: "Save note",
        label_when_pinned: None,
        description: "Saves now. Notes also save on their own a moment after typing stops.",
        icon: IconName::Check,
        keywords: &["write"],
        group: Group::Note,
        run: action!(SaveNote),
        needs: Needs::Note,
    },
    Command {
        id: "trash",
        label: "Move to trash",
        label_when_pinned: None,
        description: "Moves the note to the trash. It can be restored from there. With several notes selected in the list, moves them all.",
        icon: IconName::Trash,
        keywords: &["delete", "remove", "discard"],
        group: Group::Note,
        run: action!(TrashNote),
        needs: Needs::LiveNote,
    },
    // View.
    Command {
        id: "edit-mode",
        label: "Edit mode",
        label_when_pinned: None,
        description: "Shows the Markdown source on its own.",
        icon: IconName::SquarePen,
        keywords: &["editor", "markdown", "source", "write"],
        group: Group::View,
        run: action!(ShowEditor),
        needs: Needs::Nothing,
    },
    Command {
        id: "split-mode",
        label: "Split mode",
        label_when_pinned: None,
        description: "Shows the Markdown source and the rendered preview side by side.",
        icon: IconName::Columns2,
        keywords: &["side by side", "both", "toggle split"],
        group: Group::View,
        run: action!(ShowSplit),
        needs: Needs::Nothing,
    },
    Command {
        id: "preview-mode",
        label: "Preview mode",
        label_when_pinned: None,
        description: "Shows the rendered note on its own.",
        icon: IconName::Eye,
        keywords: &["read", "rendered", "toggle preview"],
        group: Group::View,
        run: action!(ShowPreview),
        needs: Needs::Nothing,
    },
    Command {
        id: "sidebar",
        label: "Toggle sidebar",
        label_when_pinned: None,
        description: "Collapses the sidebar to a column of icons, or opens it again with the notebooks and tags.",
        icon: IconName::PanelLeft,
        keywords: &["hide", "show", "panel", "collapse", "expand", "icons"],
        group: Group::View,
        run: action!(ToggleSidebar),
        needs: Needs::Nothing,
    },
    Command {
        id: "zoom-in",
        label: "Zoom in",
        label_when_pinned: None,
        description: "Makes the whole app larger, in every window: the sidebar, the note list, the note and its preview, menus and dialogs. Up to 150%.",
        icon: IconName::ZoomIn,
        keywords: &["bigger", "larger", "scale", "magnify", "size", "enlarge"],
        group: Group::View,
        run: action!(crate::zoom::ZoomIn),
        needs: Needs::Nothing,
    },
    Command {
        id: "zoom-out",
        label: "Zoom out",
        label_when_pinned: None,
        description: "Makes the whole app smaller, in every window. Down to 80%.",
        icon: IconName::ZoomOut,
        keywords: &["smaller", "scale", "shrink", "size"],
        group: Group::View,
        run: action!(crate::zoom::ZoomOut),
        needs: Needs::Nothing,
    },
    Command {
        id: "actual-size",
        label: "Actual size",
        label_when_pinned: None,
        description: "Shows the whole app at 100% again. The text sizes in Settings › Editor still apply.",
        icon: IconName::Scan,
        keywords: &["reset zoom", "100%", "normal size", "default size", "scale"],
        group: Group::View,
        run: action!(crate::zoom::ActualSize),
        needs: Needs::Nothing,
    },
    Command {
        id: "toggle-inspector",
        label: "Toggle outline and backlinks",
        label_when_pinned: None,
        description: "Shows or hides the pane beside the note with its outline and the notes that link to it.",
        icon: IconName::PanelRight,
        keywords: &[
            "inspector",
            "panel",
            "outline",
            "backlinks",
            "toc",
            "sidebar",
            "right",
        ],
        group: Group::View,
        run: action!(ToggleInspector),
        needs: Needs::Nothing,
    },
    Command {
        id: "show-outline",
        label: "Show outline",
        label_when_pinned: None,
        description: "Lists the note's headings beside it; click one to go there.",
        icon: IconName::TableOfContents,
        keywords: &[
            "headings",
            "table of contents",
            "toc",
            "sections",
            "inspector",
        ],
        group: Group::View,
        run: action!(ShowOutline),
        needs: Needs::Note,
    },
    Command {
        id: "ask-your-notes",
        label: "Ask Your Notes…",
        label_when_pinned: None,
        description: "Asks a question answered from your own notes, with links to the notes the answer used. It works only while AI is on in Settings › AI.",
        icon: IconName::MessageCircle,
        keywords: &["ai", "question", "answer", "assistant", "search"],
        group: Group::View,
        run: action!(AskYourNotes),
        needs: Needs::AiOn,
    },
    Command {
        id: "summarize-note",
        label: "Summarize Note",
        label_when_pinned: None,
        description: "Summarizes the open note in the Ask panel. The note isn't changed. It works only while AI is on in Settings › AI, and not for a locked note or one tagged #ai-exclude.",
        icon: IconName::Sparkles,
        keywords: &["ai", "summary", "summarise", "tl;dr", "digest", "assistant"],
        group: Group::Note,
        run: action!(SummarizeNote),
        needs: Needs::AiNote,
    },
    Command {
        id: "improve-writing",
        label: InlineAction::Improve.label(),
        label_when_pinned: None,
        description: "Suggests a better-written version of the selected text. Nothing changes until you accept it. It works only while AI is on in Settings › AI.",
        icon: IconName::Sparkles,
        keywords: &["ai", "rewrite", "polish", "edit", "assistant"],
        group: Group::Note,
        run: action!(InlineAssist {
            action: InlineAction::Improve
        }),
        needs: Needs::AiNote,
    },
    Command {
        id: "make-shorter",
        label: InlineAction::Shorten.label(),
        label_when_pinned: None,
        description: "Suggests a shorter version of the selected text. Nothing changes until you accept it. It works only while AI is on in Settings › AI.",
        icon: IconName::Sparkles,
        keywords: &["ai", "shorten", "condense", "trim", "assistant"],
        group: Group::Note,
        run: action!(InlineAssist {
            action: InlineAction::Shorten
        }),
        needs: Needs::AiNote,
    },
    Command {
        id: "fix-spelling-and-grammar",
        label: InlineAction::Fix.label(),
        label_when_pinned: None,
        description: "Suggests the selected text with its spelling and grammar corrected. Nothing changes until you accept it. It works only while AI is on in Settings › AI.",
        icon: IconName::Sparkles,
        keywords: &["ai", "proofread", "typos", "correct", "assistant"],
        group: Group::Note,
        run: action!(InlineAssist {
            action: InlineAction::Fix
        }),
        needs: Needs::AiNote,
    },
    Command {
        id: "continue-writing",
        label: InlineAction::Continue.label(),
        label_when_pinned: None,
        description: "Suggests what comes next after the selected text, or after the paragraph before the cursor. Nothing changes until you accept it. It works only while AI is on in Settings › AI.",
        icon: IconName::Sparkles,
        keywords: &["ai", "write", "next", "extend", "assistant"],
        group: Group::Note,
        run: action!(InlineAssist {
            action: InlineAction::Continue
        }),
        needs: Needs::AiNote,
    },
    Command {
        id: "summarize-selection",
        label: InlineAction::Summarize.label(),
        label_when_pinned: None,
        description: "Suggests a short summary of the selected text, to put after it. Nothing changes until you accept it. It works only while AI is on in Settings › AI.",
        icon: IconName::Sparkles,
        keywords: &["ai", "summary", "summarise", "digest", "assistant"],
        group: Group::Note,
        run: action!(InlineAssist {
            action: InlineAction::Summarize
        }),
        needs: Needs::AiNote,
    },
    Command {
        id: "show-backlinks",
        label: "Show backlinks",
        label_when_pinned: None,
        description: "Lists the notes that link to this one, and those that mention its title without a link.",
        icon: IconName::Link,
        keywords: &["links", "linked", "mentions", "references", "inspector"],
        group: Group::View,
        run: action!(ShowBacklinks),
        needs: Needs::Note,
    },
    Command {
        id: "show-graph",
        label: "Show graph",
        label_when_pinned: None,
        description: "Draws every note and the links between them. Click a note to open it; narrow it to a notebook or a tag.",
        icon: IconName::Waypoints,
        keywords: &["graph", "network", "map", "links", "connections", "web"],
        group: Group::View,
        run: action!(crate::graph_view::ShowGraph),
        needs: Needs::Nothing,
    },
    Command {
        id: "show-board",
        label: "Show board",
        label_when_pinned: None,
        description: "Lays out the notes the list shows in a column per status. Drag a card to another column, or move it with ⇧← and ⇧→, to change its status.",
        icon: IconName::SquareKanban,
        keywords: &["board", "kanban", "columns", "status", "cards", "drag"],
        group: Group::View,
        run: action!(crate::board_view::ShowBoard),
        needs: Needs::Nothing,
    },
    Command {
        id: "show-task-board",
        label: "Show tasks board",
        label_when_pinned: None,
        description: "Lays out the tasks in the notes the list shows as To do and Done. Dragging a task to Done ticks it off in its note.",
        icon: IconName::SquareKanban,
        keywords: &[
            "board", "kanban", "tasks", "to do", "todo", "done", "columns",
        ],
        group: Group::View,
        run: action!(crate::board_view::ShowTaskBoard),
        needs: Needs::Nothing,
    },
    Command {
        id: "show-local-graph",
        label: "Show note's graph",
        label_when_pinned: None,
        description: "Draws the notes this one links with, beside it, one or two links out.",
        icon: IconName::Waypoints,
        keywords: &["graph", "local", "neighbours", "links", "inspector"],
        group: Group::View,
        run: action!(crate::graph_view::ShowLocalGraph),
        needs: Needs::Note,
    },
    Command {
        id: "show-related",
        label: "Show related notes",
        label_when_pinned: None,
        description: "Lists notes about the same things as this one that it doesn't link to yet.",
        icon: IconName::Sparkles,
        keywords: &["similar", "related", "like this", "semantic", "inspector"],
        group: Group::View,
        run: action!(crate::graph_view::ShowRelated),
        needs: Needs::Note,
    },
    Command {
        id: "focus-mode",
        label: "Focus mode",
        label_when_pinned: None,
        description: "Hides the sidebar and the note list and centres the note, for writing. The note's header shows when the pointer is near the top. Escape leaves it.",
        icon: IconName::Focus,
        keywords: &["zen", "distraction free", "writing", "full", "hide panes"],
        group: Group::View,
        run: action!(ToggleFocusMode),
        needs: Needs::Nothing,
    },
    // Go to; the library adds its places before these.
    Command {
        id: "today",
        label: "Today's note",
        label_when_pinned: None,
        description: "Opens today's note in the Journal, making it from the daily template when it doesn't exist yet.",
        icon: IconName::CalendarDays,
        keywords: &["daily", "journal", "date", "diary"],
        group: Group::GoTo,
        run: action!(OpenToday),
        needs: Needs::Nothing,
    },
    Command {
        id: "journal-calendar",
        label: "Journal calendar…",
        label_when_pinned: None,
        description: "Shows a month with the days that have a daily note marked. Click a day to open its note, or to start it from the daily template.",
        icon: IconName::CalendarDays,
        keywords: &["daily", "journal", "calendar", "month", "date", "diary"],
        group: Group::GoTo,
        run: action!(crate::calendar_view::ShowCalendar),
        needs: Needs::Nothing,
    },
    Command {
        id: "previous-day",
        label: "Previous day",
        label_when_pinned: None,
        description: "Opens the daily note before the one open (or before today).",
        icon: IconName::ChevronLeft,
        keywords: &["daily", "journal", "yesterday", "back"],
        group: Group::GoTo,
        run: action!(PreviousDay),
        needs: Needs::Nothing,
    },
    Command {
        id: "next-day",
        label: "Next day",
        label_when_pinned: None,
        description: "Opens the daily note after the one open, up to today's.",
        icon: IconName::ChevronRight,
        keywords: &["daily", "journal", "tomorrow", "forward"],
        group: Group::GoTo,
        run: action!(NextDay),
        needs: Needs::Nothing,
    },
    Command {
        id: "go-to-notebook",
        label: "Go to notebook…",
        label_when_pinned: None,
        description: "Lists the notebooks; the one chosen shows in the note list.",
        icon: IconName::Notebook,
        keywords: &["notebooks", "folder", "open", "switch", "show"],
        group: Group::GoTo,
        run: Run::Pick(Picker::GoToNotebook),
        needs: Needs::Nothing,
    },
    Command {
        id: "go-to-tag",
        label: "Go to tag…",
        label_when_pinned: None,
        description: "Lists the tags; the notes with the one chosen show in the note list.",
        icon: IconName::Hash,
        keywords: &["tags", "label", "show", "switch", "filter"],
        group: Group::GoTo,
        run: Run::Pick(Picker::GoToTag),
        needs: Needs::Nothing,
    },
    Command {
        id: "go-to-list",
        label: "Go to note list",
        label_when_pinned: None,
        description: "Moves the keyboard to the note list, where the arrow keys pick a note and Return opens it for writing.",
        icon: IconName::List,
        keywords: &["focus", "list", "notes", "keyboard", "navigate"],
        group: Group::GoTo,
        run: action!(FocusNoteList),
        needs: Needs::Nothing,
    },
    Command {
        id: "next-tab",
        label: "Next tab",
        label_when_pinned: None,
        description: "Shows the tab to the right of this one, wrapping to the first at the end.",
        icon: IconName::ArrowRight,
        keywords: &["tab", "next", "right"],
        group: Group::GoTo,
        run: action!(NextTab),
        needs: Needs::Nothing,
    },
    Command {
        id: "previous-tab",
        label: "Previous tab",
        label_when_pinned: None,
        description: "Shows the tab to the left of this one, wrapping to the last at the start.",
        icon: IconName::ArrowLeft,
        keywords: &["tab", "previous", "left"],
        group: Group::GoTo,
        run: action!(PreviousTab),
        needs: Needs::Nothing,
    },
    Command {
        id: "close-tab",
        label: "Close tab",
        label_when_pinned: None,
        description: "Closes the tab showing. With one note open, closes the window.",
        icon: IconName::Close,
        keywords: &["tab", "close", "shut"],
        group: Group::GoTo,
        run: action!(CloseTab),
        needs: Needs::Nothing,
    },
    Command {
        id: "reopen-closed-tab",
        label: "Reopen closed tab",
        label_when_pinned: None,
        description: "Brings back the tab closed last, if its note still exists.",
        icon: IconName::RefreshCw,
        keywords: &["tab", "reopen", "undo", "restore"],
        group: Group::GoTo,
        run: action!(ReopenClosedTab),
        needs: Needs::Nothing,
    },
    Command {
        id: "go-to-tab-1",
        label: "Go to tab 1",
        label_when_pinned: None,
        description: "Shows tab 1 in the row; tab 9 is the last.",
        icon: IconName::Hash,
        keywords: &["tab", "go", "show", "1"],
        group: Group::GoTo,
        run: action!(GoToTab1),
        needs: Needs::Nothing,
    },
    Command {
        id: "go-to-tab-2",
        label: "Go to tab 2",
        label_when_pinned: None,
        description: "Shows tab 2 in the row; tab 9 is the last.",
        icon: IconName::Hash,
        keywords: &["tab", "go", "show", "2"],
        group: Group::GoTo,
        run: action!(GoToTab2),
        needs: Needs::Nothing,
    },
    Command {
        id: "go-to-tab-3",
        label: "Go to tab 3",
        label_when_pinned: None,
        description: "Shows tab 3 in the row; tab 9 is the last.",
        icon: IconName::Hash,
        keywords: &["tab", "go", "show", "3"],
        group: Group::GoTo,
        run: action!(GoToTab3),
        needs: Needs::Nothing,
    },
    Command {
        id: "go-to-tab-4",
        label: "Go to tab 4",
        label_when_pinned: None,
        description: "Shows tab 4 in the row; tab 9 is the last.",
        icon: IconName::Hash,
        keywords: &["tab", "go", "show", "4"],
        group: Group::GoTo,
        run: action!(GoToTab4),
        needs: Needs::Nothing,
    },
    Command {
        id: "go-to-tab-5",
        label: "Go to tab 5",
        label_when_pinned: None,
        description: "Shows tab 5 in the row; tab 9 is the last.",
        icon: IconName::Hash,
        keywords: &["tab", "go", "show", "5"],
        group: Group::GoTo,
        run: action!(GoToTab5),
        needs: Needs::Nothing,
    },
    Command {
        id: "go-to-tab-6",
        label: "Go to tab 6",
        label_when_pinned: None,
        description: "Shows tab 6 in the row; tab 9 is the last.",
        icon: IconName::Hash,
        keywords: &["tab", "go", "show", "6"],
        group: Group::GoTo,
        run: action!(GoToTab6),
        needs: Needs::Nothing,
    },
    Command {
        id: "go-to-tab-7",
        label: "Go to tab 7",
        label_when_pinned: None,
        description: "Shows tab 7 in the row; tab 9 is the last.",
        icon: IconName::Hash,
        keywords: &["tab", "go", "show", "7"],
        group: Group::GoTo,
        run: action!(GoToTab7),
        needs: Needs::Nothing,
    },
    Command {
        id: "go-to-tab-8",
        label: "Go to tab 8",
        label_when_pinned: None,
        description: "Shows tab 8 in the row; tab 9 is the last.",
        icon: IconName::Hash,
        keywords: &["tab", "go", "show", "8"],
        group: Group::GoTo,
        run: action!(GoToTab8),
        needs: Needs::Nothing,
    },
    Command {
        id: "go-to-tab-9",
        label: "Go to tab 9",
        label_when_pinned: None,
        description: "Shows tab 9 in the row; tab 9 is the last.",
        icon: IconName::Hash,
        keywords: &["tab", "go", "show", "9"],
        group: Group::GoTo,
        run: action!(GoToTab9),
        needs: Needs::Nothing,
    },
    Command {
        id: "next-note",
        label: "Next note",
        label_when_pinned: None,
        description: "Opens the note below the current one in the list.",
        icon: IconName::ArrowDown,
        keywords: &["down", "following"],
        group: Group::GoTo,
        run: action!(NextNote),
        needs: Needs::Nothing,
    },
    Command {
        id: "previous-note",
        label: "Previous note",
        label_when_pinned: None,
        description: "Opens the note above the current one in the list.",
        icon: IconName::ArrowUp,
        keywords: &["up", "back"],
        group: Group::GoTo,
        run: action!(PreviousNote),
        needs: Needs::Nothing,
    },
    Command {
        id: "go-back",
        label: "Go back",
        label_when_pinned: None,
        description: "Opens the note you were on before this one, where you left it, as a browser's Back does.",
        icon: IconName::ArrowLeft,
        keywords: &["history", "previous", "return", "last"],
        group: Group::GoTo,
        run: action!(GoBack),
        needs: Needs::Nothing,
    },
    Command {
        id: "go-forward",
        label: "Go forward",
        label_when_pinned: None,
        description: "Opens the note you went back from, where you left it.",
        icon: IconName::ArrowRight,
        keywords: &["history", "next", "redo"],
        group: Group::GoTo,
        run: action!(GoForward),
        needs: Needs::Nothing,
    },
    // App.
    Command {
        id: "whats-new",
        label: "What's new in Kanso",
        label_when_pinned: None,
        description: "The changes in this version of Kanso.",
        icon: IconName::Sparkles,
        keywords: &[
            "release notes",
            "changelog",
            "changes",
            "version",
            "new features",
        ],
        group: Group::App,
        run: action!(crate::updates::ShowWhatsNew),
        needs: Needs::Nothing,
    },
    Command {
        id: "check-updates",
        label: "Check for updates…",
        label_when_pinned: None,
        description: "Asks GitHub for a newer Kanso. A new release downloads in the background and installs when you restart.",
        icon: IconName::CircleArrowDown,
        keywords: &["update", "upgrade", "version", "release", "new"],
        group: Group::App,
        run: action!(crate::updates::CheckForUpdates),
        needs: Needs::Nothing,
    },
    Command {
        id: "sync",
        label: "Sync now",
        label_when_pinned: None,
        description: "Sends and fetches changes now, without waiting for the next automatic sync.",
        icon: IconName::RefreshCcw,
        keywords: &["cloud", "upload", "download", "refresh"],
        group: Group::App,
        run: action!(SyncNow),
        needs: Needs::SignedIn,
    },
    Command {
        id: "sign-in",
        label: "Sign in to sync…",
        label_when_pinned: None,
        description: "Signs this device in to your Kanso account, to sync its notes with your other devices, end-to-end encrypted.",
        icon: IconName::LogIn,
        keywords: &["account", "log in", "login", "cloud"],
        group: Group::App,
        run: action!(ShowSignIn),
        needs: Needs::SignedOut,
    },
    Command {
        id: "create-account",
        label: "Create account…",
        label_when_pinned: None,
        description: "Creates a Kanso account to sync notes between your devices, end-to-end encrypted. The notes on this device are uploaded.",
        icon: IconName::UserPlus,
        keywords: &["sign up", "register", "sync", "cloud"],
        group: Group::App,
        run: action!(ShowCreateAccount),
        needs: Needs::SignedOut,
    },
    Command {
        id: "change-password",
        label: "Change password…",
        label_when_pinned: None,
        description: "Changes your account's password. Your other devices ask for the new one.",
        icon: IconName::KeyRound,
        keywords: &["account", "security", "sync"],
        group: Group::App,
        run: action!(ChangePassword),
        needs: Needs::SignedIn,
    },
    Command {
        id: "new-recovery-key",
        label: "New recovery key…",
        label_when_pinned: None,
        description: "Replaces your account's recovery key. The one you saved stops working.",
        icon: IconName::FileKey,
        keywords: &["account", "security", "forgot password"],
        group: Group::App,
        run: action!(NewRecoveryKey),
        needs: Needs::SignedIn,
    },
    Command {
        id: "sign-out",
        label: "Sign out",
        label_when_pinned: None,
        description: "Signs this device out of your account. Notes stay on this device; sync stops until you sign in again.",
        icon: IconName::LogOut,
        keywords: &["account", "log out", "logout", "sync"],
        group: Group::App,
        run: action!(SignOut),
        needs: Needs::SignedIn,
    },
    Command {
        id: "import",
        label: "Import…",
        label_when_pinned: None,
        description: "Brings in notes from a folder of Markdown files (Obsidian and Logseq folders too) or an export from Notion, Evernote, Bear, Joplin, Standard Notes, Simplenote or UpNote, or the notes library found on this computer. Shows what will come in first; the source is only read.",
        icon: IconName::Upload,
        keywords: &[
            "import", "markdown", "obsidian", "notion", "evernote", "enex", "bear", "joplin",
            "logseq", "vault", "migrate",
        ],
        group: Group::App,
        run: action!(ImportLibrary),
        needs: Needs::Nothing,
    },
    Command {
        id: "open-file",
        label: "Open Markdown file…",
        label_when_pinned: None,
        description: "Brings a .md or .txt file into the Inbox and opens it. Opening it again updates the same note.",
        icon: IconName::FileText,
        keywords: &["open", "file", "markdown", "md", "txt"],
        group: Group::App,
        run: action!(OpenFile),
        needs: Needs::Nothing,
    },
    Command {
        id: "export-all",
        label: "Export all notes…",
        label_when_pinned: None,
        description: "Saves every note as a Markdown file, in folders named after the notebooks, in a folder you choose.",
        icon: IconName::HardDrive,
        keywords: &["backup", "markdown", "library"],
        group: Group::App,
        run: action!(ExportLibrary),
        needs: Needs::Nothing,
    },
    Command {
        id: "settings",
        label: "Settings",
        label_when_pinned: None,
        description: "Opens the settings in place of the notes, at the page shown last. Escape goes back to the notes.",
        icon: IconName::Settings,
        keywords: &["preferences", "options"],
        group: Group::App,
        run: action!(OpenSettings),
        needs: Needs::Nothing,
    },
    Command {
        id: "back-to-notes",
        label: "Back to notes",
        label_when_pinned: None,
        description: "Leaves Settings for the notes as they were: the same note, with the keyboard where it was.",
        icon: IconName::ArrowLeft,
        keywords: &["close settings", "leave", "return", "notes", "done"],
        group: Group::App,
        run: action!(CloseSettings),
        needs: Needs::Settings,
    },
    Command {
        id: "settings-general",
        label: "Settings: General",
        label_when_pinned: None,
        description: "Opening at login, the data folder, backups, importing notes, reminders, the quick note shortcut and the daily note's template.",
        icon: IconName::Settings,
        keywords: &[
            "preferences",
            "login",
            "launch at login",
            "startup",
            "backup",
            "data folder",
            "import",
            "reminder",
            "notifications",
            "quick note",
            "capture",
            "hotkey",
            "global shortcut",
            "daily",
            "journal",
            "template",
        ],
        group: Group::App,
        run: action!(OpenGeneralSettings),
        needs: Needs::Nothing,
    },
    Command {
        id: "settings-editor",
        label: "Settings: Editor",
        label_when_pinned: None,
        description: "The editor's font, text size and line numbers, and how writing feels: closing brackets, tables, smart paste, typewriter scrolling, the word goal, tags from #words and Vim keys.",
        icon: IconName::Pencil,
        keywords: &[
            "preferences",
            "font",
            "size",
            "monospace",
            "line numbers",
            "readable width",
            "vim",
            "typewriter",
            "word goal",
            "tags",
            "hashtags",
            "smart paste",
            "brackets",
            "pairing",
            "tables",
        ],
        group: Group::App,
        run: action!(OpenEditorSettings),
        needs: Needs::Nothing,
    },
    Command {
        id: "settings-appearance",
        label: "Settings: Themes",
        label_when_pinned: None,
        description: "The UI, editor syntax and preview themes, light and dark.",
        icon: IconName::Palette,
        keywords: &[
            "preferences",
            "appearance",
            "theme",
            "color scheme",
            "colors",
            "syntax",
            "dark mode",
            "light mode",
            "preview theme",
        ],
        group: Group::App,
        run: action!(OpenAppearanceSettings),
        needs: Needs::Nothing,
    },
    Command {
        id: "settings-browse-themes",
        label: "Settings: Browse Themes",
        label_when_pinned: None,
        description: "Install and update color themes from the signed kanso-themes gallery.",
        icon: IconName::Palette,
        keywords: &["gallery", "browse", "install", "download", "theme"],
        group: Group::App,
        run: action!(BrowseThemes),
        needs: Needs::Nothing,
    },
    Command {
        id: "settings-browse-icon-themes",
        label: "Settings: Browse Icon Themes",
        label_when_pinned: None,
        description: "Install and update icon themes from the signed kanso-themes gallery.",
        icon: IconName::Shapes,
        keywords: &["gallery", "browse", "install", "download", "icons"],
        group: Group::App,
        run: action!(BrowseIconThemes),
        needs: Needs::Nothing,
    },
    Command {
        id: "settings-refresh-theme-gallery",
        label: "Refresh Theme Gallery",
        label_when_pinned: None,
        description: "Check the theme gallery for new and withdrawn themes.",
        icon: IconName::RefreshCw,
        keywords: &["themes", "gallery", "check", "update"],
        group: Group::App,
        run: action!(RefreshThemeGallery),
        needs: Needs::Nothing,
    },
    Command {
        id: "settings-stop-theme-preview",
        label: "Stop Theme Preview",
        label_when_pinned: None,
        description: "Put back the themes in force before a preview.",
        icon: IconName::EyeOff,
        keywords: &["themes", "preview", "stop"],
        group: Group::App,
        run: action!(StopThemePreview),
        needs: Needs::Nothing,
    },
    Command {
        id: "settings-icon-themes",
        label: "Settings: Icon Themes",
        label_when_pinned: None,
        description: "The icon theme that draws every icon in Kanso, and your icon themes folder.",
        icon: IconName::Shapes,
        keywords: &[
            "preferences",
            "icons",
            "icon theme",
            "icon pack",
            "import",
            "zed",
        ],
        group: Group::App,
        run: action!(OpenIconThemeSettings),
        needs: Needs::Nothing,
    },
    Command {
        id: "icon-theme",
        label: "Change icon theme…",
        label_when_pinned: None,
        description: "Picks the icon theme that draws every icon in Kanso, at once: Kanso's own, Phosphor, or one in your icon themes folder.",
        icon: IconName::Shapes,
        keywords: &[
            "icons",
            "icon theme",
            "look",
            "appearance",
            "phosphor",
            "lucide",
        ],
        group: Group::App,
        run: Run::Pick(Picker::IconTheme),
        needs: Needs::Nothing,
    },
    Command {
        id: "import-icon-theme",
        label: "Import Icon Theme…",
        label_when_pinned: None,
        description: "Converts a Zed icon theme, a theme package carrying one, or a Kanso icon theme folder into your icon themes folder.",
        icon: IconName::Download,
        keywords: &["icons", "icon theme", "import", "zed"],
        group: Group::App,
        run: action!(ImportIconTheme),
        needs: Needs::Nothing,
    },
    Command {
        id: "reload-icon-themes",
        label: "Reload Icon Themes",
        label_when_pinned: None,
        description: "Reads the icon themes folder again, after adding or editing an icon theme.",
        icon: IconName::RefreshCw,
        keywords: &["icons", "icon theme", "refresh"],
        group: Group::App,
        run: action!(ReloadIconThemes),
        needs: Needs::Nothing,
    },
    Command {
        id: "reload-themes",
        label: "Reload Themes",
        label_when_pinned: None,
        description: "Reads the themes folder again, after adding or editing a theme file.",
        icon: IconName::RefreshCw,
        keywords: &["theme", "colors", "refresh"],
        group: Group::App,
        run: action!(ReloadThemes),
        needs: Needs::Nothing,
    },
    Command {
        id: "settings-account",
        label: "Settings: Account",
        label_when_pinned: None,
        description: "Your sync account: signing in, storage, devices, the password and recovery key, and how often sync runs. Published links live on the same server.",
        icon: IconName::CircleUser,
        keywords: &[
            "preferences",
            "sync",
            "cloud",
            "devices",
            "recovery key",
            "conflict",
            "login",
            "sign in",
            "log in",
            "server",
            "publish",
        ],
        group: Group::App,
        run: action!(OpenAccountSettings),
        needs: Needs::Nothing,
    },
    Command {
        id: "settings-ai",
        label: "Settings: AI",
        label_when_pinned: None,
        description: "Whether AI is on, where questions go, and the key a cloud provider needs.",
        icon: IconName::Sparkles,
        keywords: &[
            "preferences",
            "ai",
            "ask",
            "model",
            "provider",
            "openai",
            "anthropic",
            "openrouter",
            "ollama",
            "lm studio",
            "api key",
        ],
        group: Group::App,
        run: action!(OpenAiSettings),
        needs: Needs::Nothing,
    },
    Command {
        id: "settings-plugins",
        label: "Settings: Plugins",
        label_when_pinned: None,
        description: "The plugins installed, what each may do, and the plugin directory.",
        icon: IconName::Puzzle,
        keywords: &[
            "preferences",
            "plugins",
            "extensions",
            "add-ons",
            "permissions",
            "directory",
            "developer mode",
            "safe mode",
        ],
        group: Group::App,
        run: action!(OpenPluginSettings),
        needs: Needs::Nothing,
    },
    Command {
        id: "settings-api",
        label: "Settings: Local API",
        label_when_pinned: None,
        description: "Kanso's local API: its address, port, username and password, and the MCP endpoint.",
        icon: IconName::Plug,
        keywords: &[
            "preferences",
            "api",
            "mcp",
            "port",
            "password",
            "rest",
            "agents",
            "server",
        ],
        group: Group::App,
        run: action!(OpenApiSettings),
        needs: Needs::Nothing,
    },
    Command {
        id: "settings-updates",
        label: "Settings: Updates",
        label_when_pinned: None,
        description: "This version, when Kanso last looked for a new one, automatic updates, and what's new.",
        icon: IconName::CircleArrowDown,
        keywords: &[
            "preferences",
            "update",
            "version",
            "release",
            "changelog",
            "what's new",
            "auto-update",
        ],
        group: Group::App,
        run: action!(OpenUpdateSettings),
        needs: Needs::Nothing,
    },
    Command {
        id: "quit",
        label: "Quit Kanso",
        label_when_pinned: None,
        description: "Saves open notes and quits.",
        icon: IconName::Close,
        keywords: &["exit", "close"],
        group: Group::App,
        run: action!(Quit),
        needs: Needs::Nothing,
    },
];

/// Every command, in the order the palette lists them.
pub fn all() -> &'static [Command] {
    COMMANDS
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use gpui_kit::{AssetSource as _, TestAppContext};

    use super::{Run, all};
    use crate::assets::KansoAssets;
    use crate::workspace::OpenCommandPalette;

    #[test]
    fn ids_are_unique() {
        let mut seen = HashSet::new();
        for command in all() {
            assert!(seen.insert(command.id), "{} twice", command.id);
        }
    }

    #[test]
    fn labels_are_unique_and_follow_the_spec_words() {
        let mut seen = HashSet::new();
        for command in all() {
            assert!(seen.insert(command.label), "{} twice", command.label);
            let words = format!("{} {}", command.label, command.description).to_lowercase();
            assert!(!words.contains("vault"), "{}", command.id);
        }
    }

    /// Every shortcut that works anywhere in the window also has a command,
    /// so the palette is a complete list of what the keyboard can do.
    #[gpui_kit::test]
    fn every_window_wide_shortcut_has_a_command(cx: &mut TestAppContext) {
        cx.update(|cx| {
            crate::workspace::bind_keys(cx);
            crate::note_list::bind_keys(cx);
            let keymap = cx.key_bindings();
            let keymap = keymap.borrow();
            let mut checked = 0;
            for binding in keymap.bindings() {
                // Keys for one view (Enter in the list, arrows in the
                // palette) are navigation, not commands.
                if binding.predicate().is_some() || binding.action().partial_eq(&OpenCommandPalette)
                {
                    continue;
                }
                checked += 1;
                assert!(
                    all()
                        .iter()
                        .filter_map(|c| c.action())
                        .any(|a| a.partial_eq(binding.action())),
                    "no command for {}",
                    binding.action().name()
                );
            }
            assert!(checked >= 15, "{checked}");
        });
    }

    /// Shortcuts for one place are skipped above, but some of them leave or
    /// move between places, and those have commands too: Escape leaves
    /// Settings (the palette lists it only while they are open).
    #[test]
    fn leaving_and_moving_have_commands() {
        use super::Needs;
        use crate::settings::CloseSettings;
        use crate::workspace::FocusNoteList;
        let find = |action: &dyn gpui_kit::Action| {
            all()
                .iter()
                .find(|c| c.action().is_some_and(|a| a.partial_eq(action)))
        };
        let back = find(&CloseSettings).expect("a command leaves Settings");
        assert_eq!(back.needs, Needs::Settings);
        assert_eq!(back.label, "Back to notes");
        assert!(find(&FocusNoteList).is_some(), "a command goes to the list");
    }

    #[test]
    fn pickers_have_no_action_and_actions_build() {
        for command in all() {
            match command.run {
                Run::Action(_) => assert!(command.action().is_some()),
                Run::Pick(_) => assert!(command.action().is_none()),
            }
        }
    }

    /// The phones' catalog (`kanso_engine::palette`) is the desktop's:
    /// every command in it is a desktop command with the same label,
    /// description, keywords, group, conditions, list and icon.
    #[test]
    fn the_engine_catalog_is_the_desktops_commands() {
        use kanso_engine::palette as engine;
        let needs = |n: engine::Needs| match n {
            engine::Needs::Nothing => super::Needs::Nothing,
            engine::Needs::Note => super::Needs::Note,
            engine::Needs::LiveNote => super::Needs::LiveNote,
            engine::Needs::UnlockedNote => super::Needs::UnlockedNote,
            engine::Needs::LockedNote => super::Needs::LockedNote,
            engine::Needs::Settings => super::Needs::Settings,
            engine::Needs::SignedIn => super::Needs::SignedIn,
            engine::Needs::SignedOut => super::Needs::SignedOut,
            engine::Needs::ImageLine => super::Needs::ImageLine,
            engine::Needs::AudioLine => super::Needs::AudioLine,
            engine::Needs::SharedNote => super::Needs::SharedNote,
            engine::Needs::AiOn => super::Needs::AiOn,
            engine::Needs::AiNote => super::Needs::AiNote,
        };
        let group = |g: engine::Group| match g {
            engine::Group::Note => super::Group::Note,
            engine::Group::View => super::Group::View,
            engine::Group::GoTo => super::Group::GoTo,
            engine::Group::App => super::Group::App,
        };
        let picker = |p: engine::Picker| match p {
            engine::Picker::Notebook => super::Picker::Notebook,
            engine::Picker::Tag => super::Picker::Tag,
            engine::Picker::Status => super::Picker::Status,
            engine::Picker::GoToNotebook => super::Picker::GoToNotebook,
            engine::Picker::GoToTag => super::Picker::GoToTag,
            engine::Picker::WordGoal => super::Picker::WordGoal,
            engine::Picker::Template => super::Picker::Template,
            engine::Picker::ImageSize => super::Picker::ImageSize,
        };
        // An icon's Lucide name is its file's.
        let lucide = |icon: gpui_kit::assets::IconName| {
            let path = icon.path().to_string();
            path.rsplit('/')
                .next()
                .and_then(|file| file.strip_suffix(".svg"))
                .unwrap_or_default()
                .to_owned()
        };
        assert!(engine::all().len() > 60, "{}", engine::all().len());
        for theirs in engine::all() {
            let ours = all()
                .iter()
                .find(|c| c.id == theirs.id)
                .unwrap_or_else(|| panic!("no desktop command {}", theirs.id));
            assert_eq!(theirs.label, ours.label, "{}", theirs.id);
            // Where the PDF opens differs by system; the catalog has macOS's.
            if theirs.id != "export-pdf" || cfg!(target_os = "macos") {
                assert_eq!(theirs.description, ours.description, "{}", theirs.id);
            }
            assert_eq!(theirs.keywords, ours.keywords, "{}", theirs.id);
            assert_eq!(group(theirs.group), ours.group, "{}", theirs.id);
            assert_eq!(needs(theirs.needs), ours.needs, "{}", theirs.id);
            assert_eq!(lucide(ours.icon), theirs.icon, "{}", theirs.id);
            match (theirs.picker, ours.run) {
                (Some(p), Run::Pick(q)) => assert_eq!(picker(p), q, "{}", theirs.id),
                (None, Run::Action(_)) => {}
                _ => panic!("{}: one opens a list and the other doesn't", theirs.id),
            }
            match (theirs.toggled, ours.label_when_pinned) {
                (None, None) => {}
                (Some(t), Some((label, icon))) => {
                    assert_eq!(t.label, label, "{}", theirs.id);
                    assert_eq!(t.icon, lucide(icon), "{}", theirs.id);
                }
                _ => panic!("{}: toggled labels differ", theirs.id),
            }
        }
    }

    #[test]
    fn every_icon_is_bundled() {
        for command in all() {
            let icons = [
                Some(command.icon),
                command.label_when_pinned.map(|(_, icon)| icon),
            ];
            for icon in icons.into_iter().flatten() {
                let path = icon.path();
                assert!(
                    matches!(KansoAssets.load(&path), Ok(Some(_))),
                    "{}: {path}",
                    command.id
                );
            }
        }
    }
}
