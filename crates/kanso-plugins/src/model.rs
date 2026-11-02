//! The shapes of a plugin: its manifest, what it asks for, what the person
//! granted, and how it is installed. Validation lives in [`crate::manifest`]
//! and [`crate::grant`]; this file only describes.

use std::collections::BTreeSet;

use kanso_plugin_host::protocol::Registered;
use serde::{Deserialize, Serialize};

/// The manifest layout this app reads.
pub const SCHEMA: u32 = 1;
/// The plugin API this app offers.
pub const API_VERSION: u32 = 1;

/// `manifest.json`, as a bundle carries it. Unknown keys anywhere are an
/// error, so a manifest can't smuggle a future capability past this app.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Manifest {
    pub schema: u32,
    pub api_version: u32,
    /// Reverse-DNS: `com.example.wordcount`.
    pub id: String,
    /// What the person sees, at most 48 characters.
    pub name: String,
    /// Semver.
    pub version: String,
    /// The script, a relative `.js` path inside the bundle.
    pub entry: String,
    pub publisher: Publisher,
    /// The oldest Kanso the plugin runs in, as semver.
    pub min_app_version: String,
    /// A relative `.svg` or `.png` path inside the bundle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// One sentence for the directory and the install sheet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub permissions: Permissions,
}

/// Who publishes a plugin, and the key they sign it with.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Publisher {
    pub name: String,
    /// `ed25519:` and the public key's 64 lowercase hex digits.
    pub key: String,
}

/// What a manifest asks for. Every field is a host call the plugin may
/// make; with a field empty or off, the call is refused.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Permissions {
    /// Palette commands.
    #[serde(default)]
    pub commands: bool,
    #[serde(default)]
    pub editor: BTreeSet<EditorPermission>,
    #[serde(default)]
    pub notes: NotesRequest,
    /// Fenced languages it draws.
    #[serde(default)]
    pub fences: Vec<Fence>,
    #[serde(default)]
    pub ui: BTreeSet<Surface>,
    /// Keychain tokens it may read, by name.
    #[serde(default)]
    pub secrets: BTreeSet<String>,
    /// HTTPS hosts it may fetch from, exactly as written.
    #[serde(default)]
    pub network: BTreeSet<String>,
    /// The clipboard. Reading the history makes Kanso watch the clipboard
    /// while the plugin is on; a plugin that reads it can't use the network.
    #[serde(default)]
    pub clipboard: BTreeSet<ClipboardPermission>,
}

/// What a plugin may do with the clipboard.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ClipboardPermission {
    /// Read the texts copied in any app while Kanso runs: the last 15,
    /// with passwords and what looks like a key left out.
    #[serde(rename = "history.read")]
    HistoryRead,
    /// Put text on the clipboard.
    #[serde(rename = "write")]
    Write,
}

/// What a plugin may do with the open note.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EditorPermission {
    /// Read the note that is open: its ID, and `notes.get` of it.
    #[serde(rename = "note.read")]
    NoteRead,
    /// Read the text selected in it.
    #[serde(rename = "selection.read")]
    SelectionRead,
    /// Replace the selection, as one edit Undo takes back.
    #[serde(rename = "selection.replace")]
    SelectionReplace,
}

/// What a manifest asks of the notes. Notebook and tag IDs differ between
/// libraries, so the lists are only what the author suggests; the person
/// picks the notebooks and tags at install.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NotesRequest {
    /// Reading notes; `None` is none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read: Option<ReadRequest>,
    /// Changing notes in these notebooks; empty is none.
    #[serde(default)]
    pub write: Vec<String>,
    /// Making new notes in a notebook it may change.
    #[serde(default)]
    pub create: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReadRequest {
    #[serde(default)]
    pub books: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Every notebook: a switch of its own on the install sheet.
    #[serde(default)]
    pub all: bool,
}

/// A fenced language a plugin draws. There is no network field: a renderer
/// never fetches.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Fence {
    pub language: String,
}

/// Where a plugin may show something of its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Surface {
    /// A panel in the inspector column, framed by Kanso.
    Panel,
}

/// What the person allowed, which is never more than the manifest asks
/// ([`crate::grant::check`]). Kept in the local database; it never syncs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Grant {
    pub commands: bool,
    pub editor: BTreeSet<EditorPermission>,
    pub notes: NotesGrant,
    /// Fenced languages it may draw.
    pub fences: BTreeSet<String>,
    pub panel: bool,
    pub secrets: BTreeSet<String>,
    pub network: BTreeSet<String>,
    /// Grants made before the clipboard existed have none.
    #[serde(default)]
    pub clipboard: BTreeSet<ClipboardPermission>,
    /// Review every change to notes before it is applied, even to the open
    /// note. On by default.
    pub ask_before_changing: bool,
}

/// The notes a plugin may read and change.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotesGrant {
    /// Notebooks it may read, with every notebook inside them.
    pub read_books: BTreeSet<String>,
    /// Tags whose notes it may read, wherever they are.
    pub read_tags: BTreeSet<String>,
    /// Every unlocked notebook.
    pub read_all: bool,
    /// Notebooks (with those inside) whose notes it may change.
    pub write_books: BTreeSet<String>,
    /// It may make notes in `write_books`.
    pub create: bool,
}

impl NotesGrant {
    /// Whether any read of notes was granted.
    pub fn reads(&self) -> bool {
        self.read_all || !self.read_books.is_empty() || !self.read_tags.is_empty()
    }

    /// Whether any change to notes was granted.
    pub fn writes(&self) -> bool {
        !self.write_books.is_empty()
    }
}

/// Where an installed plugin came from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Source {
    /// The signed directory, at this bundle hash.
    Registry { sha256: String },
    /// A `.kansoplugin` file, signed by the directory, at this hash.
    File { sha256: String },
    /// An unpacked folder in developer mode: never updated or published.
    Developer { path: String },
}

/// Why a plugin is off, other than being switched off.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Disabled {
    /// The person switched it off.
    ByYou,
    /// It crashed twice in ten minutes.
    Crashed,
    /// It failed to start three times.
    FailedToStart,
    /// It asked three times for something it wasn't granted.
    Refused,
    /// The directory revoked this version.
    Revoked { reason: String },
    /// It is on this Mac's block list.
    Blocked,
    /// It needs a newer Kanso.
    NeedsNewerKanso { version: String },
}

/// An installed plugin.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Installed {
    pub manifest: Manifest,
    pub source: Source,
    /// The author key pinned at first install (trust on first use).
    pub pinned_key: String,
    pub grant: Grant,
    /// `None` while it may run.
    pub disabled: Option<Disabled>,
    /// What it registered the last time it started, so its commands are in
    /// the palette without running it.
    #[serde(default)]
    pub registered: Registered,
    /// Milliseconds since the epoch.
    pub installed_at: i64,
    /// The last thing that went wrong, for Settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
}

impl Installed {
    pub fn id(&self) -> &str {
        &self.manifest.id
    }

    pub fn is_developer(&self) -> bool {
        matches!(self.source, Source::Developer { .. })
    }

    pub fn may_run(&self) -> bool {
        self.disabled.is_none()
    }
}
