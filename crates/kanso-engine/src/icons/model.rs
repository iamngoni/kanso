//! An icon theme as Kanso keeps it once read (`file` reads one), the
//! library's icon slots a theme can restyle, and where a theme came from.
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

/// An icon in Kanso's library that a theme can draw on its own, apart from
/// the UI icon it shows by default: a notebook, a note's states, the places
/// in the sidebar. A slot a theme leaves out falls back, per slot, to
/// [`Slot::fallback`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Slot {
    Notebook,
    NotebookOpen,
    NotebookLocked,
    FoldClosed,
    FoldOpen,
    Inbox,
    Journal,
    Templates,
    Note,
    NotePinned,
    NoteLocked,
    NoteShared,
    NotePublished,
    Tag,
    AllNotes,
    Pinned,
    Recent,
    Tasks,
    Shared,
    Trash,
    Notebooks,
    Tags,
    Board,
    Calendar,
    Graph,
    AttachmentPdf,
    AttachmentImage,
    AttachmentAudio,
}

/// What a slot shows when a theme doesn't draw it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fallback {
    /// Another slot, say a special notebook falling back to the notebook.
    Slot(Slot),
    /// A UI icon by its name, which the theme may draw in turn.
    Icon(&'static str),
}

impl Slot {
    /// Every slot, in the order the format documents them.
    pub const ALL: [Slot; 28] = [
        Slot::Notebook,
        Slot::NotebookOpen,
        Slot::NotebookLocked,
        Slot::FoldClosed,
        Slot::FoldOpen,
        Slot::Inbox,
        Slot::Journal,
        Slot::Templates,
        Slot::Note,
        Slot::NotePinned,
        Slot::NoteLocked,
        Slot::NoteShared,
        Slot::NotePublished,
        Slot::Tag,
        Slot::AllNotes,
        Slot::Pinned,
        Slot::Recent,
        Slot::Tasks,
        Slot::Shared,
        Slot::Trash,
        Slot::Notebooks,
        Slot::Tags,
        Slot::Board,
        Slot::Calendar,
        Slot::Graph,
        Slot::AttachmentPdf,
        Slot::AttachmentImage,
        Slot::AttachmentAudio,
    ];

    /// The slot's key in a theme file's `library`.
    pub fn key(self) -> &'static str {
        self.info().0
    }

    /// What the slot draws, in words for the format's documentation.
    pub fn description(self) -> &'static str {
        self.info().1
    }

    pub fn fallback(self) -> Fallback {
        self.info().2
    }

    pub fn parse(key: &str) -> Option<Slot> {
        Slot::ALL.into_iter().find(|slot| slot.key() == key)
    }

    fn info(self) -> (&'static str, &'static str, Fallback) {
        use Fallback::{Icon, Slot as S};
        match self {
            Slot::Notebook => (
                "notebook",
                "A notebook, folded, without an icon of its own.",
                Icon("notebook"),
            ),
            Slot::NotebookOpen => (
                "notebook-open",
                "A notebook opened in the sidebar to show the ones inside it.",
                S(Slot::Notebook),
            ),
            Slot::NotebookLocked => (
                "notebook-locked",
                "A locked notebook, in place of its icon.",
                Icon("lock"),
            ),
            Slot::FoldClosed => (
                "fold-closed",
                "The arrow beside a folded notebook or tag.",
                Icon("chevron-right"),
            ),
            Slot::FoldOpen => (
                "fold-open",
                "The arrow beside an open notebook or tag.",
                Icon("chevron-down"),
            ),
            Slot::Inbox => (
                "inbox",
                "The Inbox notebook, where quick captures go.",
                S(Slot::Notebook),
            ),
            Slot::Journal => (
                "journal",
                "The Journal notebook of daily notes.",
                S(Slot::Notebook),
            ),
            Slot::Templates => ("templates", "The Templates notebook.", S(Slot::Notebook)),
            Slot::Note => (
                "note",
                "A note in lists and the palette.",
                Icon("file-text"),
            ),
            Slot::NotePinned => ("note-pinned", "The mark on a pinned note.", Icon("pin")),
            Slot::NoteLocked => ("note-locked", "The mark on a locked note.", Icon("lock")),
            Slot::NoteShared => ("note-shared", "The mark on a shared note.", Icon("users")),
            Slot::NotePublished => (
                "note-published",
                "The mark on a published note.",
                Icon("globe"),
            ),
            Slot::Tag => ("tag", "A tag in lists and the palette.", Icon("hash")),
            Slot::AllNotes => (
                "all-notes",
                "All Notes, in the sidebar and the palette.",
                Icon("file-text"),
            ),
            Slot::Pinned => (
                "pinned",
                "Pinned notes, in the sidebar and the palette.",
                Icon("pin"),
            ),
            Slot::Recent => (
                "recent",
                "Recent notes, in the sidebar and the palette.",
                Icon("clock"),
            ),
            Slot::Tasks => (
                "tasks",
                "Tasks, in the sidebar and the palette.",
                Icon("circle-check"),
            ),
            Slot::Shared => ("shared", "Notes shared with you.", Icon("users")),
            Slot::Trash => ("trash", "The Trash.", Icon("trash")),
            Slot::Notebooks => (
                "notebooks",
                "The notebooks, in the collapsed sidebar.",
                Icon("notebook"),
            ),
            Slot::Tags => ("tags", "The tags, in the collapsed sidebar.", Icon("hash")),
            Slot::Board => (
                "board",
                "The board of notes by status.",
                Icon("square-kanban"),
            ),
            Slot::Calendar => (
                "calendar",
                "The calendar of daily notes.",
                Icon("calendar-days"),
            ),
            Slot::Graph => ("graph", "The graph of linked notes.", Icon("waypoints")),
            Slot::AttachmentPdf => (
                "attachment-pdf",
                "A PDF attachment (kept for attachment lists).",
                Icon("file"),
            ),
            Slot::AttachmentImage => (
                "attachment-image",
                "An image attachment (kept for attachment lists).",
                Icon("image"),
            ),
            Slot::AttachmentAudio => (
                "attachment-audio",
                "An audio attachment (kept for attachment lists).",
                Icon("music"),
            ),
        }
    }
}

/// An icon a theme adds to the notebook icon picker.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtraIcon {
    pub name: String,
    pub label: String,
    /// More words the picker's search finds it by, spaced.
    pub keywords: String,
    pub svg: Arc<[u8]>,
}

/// An icon theme, read and checked, its SVGs in memory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IconTheme {
    /// Its folder's name: lowercase letters, digits, `-` and `_`.
    pub id: String,
    pub name: String,
    pub author: String,
    pub license: String,
    pub source: Option<String>,
    pub description: Option<String>,
    /// UI icons by their name (`search`, `chevron-down`).
    pub icons: BTreeMap<String, Arc<[u8]>>,
    pub library: BTreeMap<Slot, Arc<[u8]>>,
    /// In name order.
    pub extras: Vec<ExtraIcon>,
    /// Where a person's own theme was read from; `None` for a built-in one.
    pub path: Option<PathBuf>,
    /// Whether it is compiled in, the person's own, or from the gallery.
    pub origin: Origin,
    /// The ID and a fingerprint of every SVG: what the asset paths of its
    /// icons carry, so a changed theme never draws an icon cached from
    /// before (`icon_themes::themed_path`).
    pub key: String,
}

impl IconTheme {
    pub fn extra(&self, name: &str) -> Option<&ExtraIcon> {
        self.extras
            .binary_search_by(|icon| icon.name.as_str().cmp(name))
            .ok()
            .map(|ix| &self.extras[ix])
    }

    /// How many icons it draws in all.
    pub fn count(&self) -> usize {
        self.icons.len() + self.library.len() + self.extras.len()
    }
}

/// An entry of a theme file's `notebook_icons`, as written: a path, or a
/// path with words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtraEntry {
    pub file: String,
    pub label: Option<String>,
    pub keywords: Option<String>,
}

/// A theme file as written, its SVGs named by path but not yet read.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IconThemeFile {
    pub name: String,
    pub author: String,
    pub license: String,
    pub source: Option<String>,
    pub description: Option<String>,
    pub icons: BTreeMap<String, String>,
    pub library: BTreeMap<Slot, String>,
    pub notebook_icons: BTreeMap<String, ExtraEntry>,
}

// MARK: Where a theme came from

/// A theme's source.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Origin {
    /// Compiled in from `themes/`, reviewed with the release.
    #[default]
    Builtin,
    /// A file in the person's `themes/` or `icon-themes/`, imported or
    /// dropped there.
    Own,
    /// Installed by the gallery under `<data>/gallery/`.
    Gallery,
}

impl Origin {
    /// The detail line's first words.
    pub fn detail(self) -> &'static str {
        match self {
            Origin::Builtin => "Built in",
            Origin::Own => "Yours",
            Origin::Gallery => "From the gallery",
        }
    }
}

/// A theme file that couldn't be used, and why, in words for the Themes
/// page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notice {
    pub file: String,
    pub message: String,
}

impl Notice {
    pub fn sentence(&self) -> String {
        format!("{} {}.", self.file, self.message.trim_end_matches('.'))
    }
}
