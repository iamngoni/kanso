//! Kanso's library for the phones' app (#48).
//!
//! The Flutter shell calls this crate, through the bridge in `mobile/rust`,
//! and nothing else of Kanso's. It is the facade over the shared core: the
//! store, the engine (lock sessions, accounts, sync) and the renderer. It
//! opens no window, reads no setting of the shell's and depends on no bridge
//! generator, so everything here is plain Rust, tested on a computer and on
//! the simulators.
//!
//! # What is here
//!
//! - [`Library`] opens a library (one database file) and answers for its
//!   notes: lists by place with the desktop's sort, narrowing and grouping,
//!   the counts of the places, one note, saving, creating, changing several
//!   notes at once, notebooks in tree order, tags, search by scope with the
//!   words matched, properties and query views, and the
//!   [`Library::changes`] stream.
//! - [`LockSession`] (`library.locks()`) holds the lock password's keys. A
//!   locked note's text, snippet, properties, tasks and preview never come
//!   back while it is closed; only its title and its lock show in a list.
//!   Keys never cross this edge.
//! - [`Linking`] (`library.linking()`) has a note's outline, backlinks and
//!   unlinked mentions, Link on a mention, Open Note and the command palette,
//!   with a locked note kept out until the lock session is open.
//! - [`EditorSession`] (`library.edit(id)`) is a note open for editing: the
//!   text the shell's field shows, with the undo history, the editor's
//!   commands, the colors of the text, completion, autosave and the
//!   desktop's choice between versions when a note changes elsewhere while
//!   it is typed in.
//! - [`SyncHandle`] (`library.sync()`) signs in to a server, syncs and
//!   reports its status.
//! - [`Renderer`] (`library.renderer()`) draws a note's preview, highlights
//!   source, fits a theme (the built-in ones are in, as the desktop has
//!   them, and source can be colored in any of them) and draws math and
//!   diagrams.
//! - [`AccountHandle`] (`library.account()`) signs in and up with every
//!   way the server offers (password, passkey, Apple, Google, GitHub),
//!   opens a locked account with the password, another device or the
//!   recovery key, and asks the binding question.
//!   [`AccountSettings`] (`library.account_settings()`) is the account's
//!   page, and [`Approvals`] (`library.approvals()`) the requests of other
//!   devices this phone may approve.
//! - [`SharedNotes`] (`library.shared()`) shares notes with other accounts
//!   and keeps an open shared note live.
//! - [`Gallery`] (`library.gallery()`) lists and installs the gallery's
//!   themes, checked as the desktop checks them: the signed index, then
//!   each file's pinned hash.
//! - [`background`] is the sync that runs with no app: the code behind the
//!   C entry `kanso_background_sync` that `mobile/rust` exports.
//! - [`import`] brings notes in from another app's export or a folder of
//!   Markdown (`Library::import_plan` shows what would happen and writes
//!   nothing, `Library::import` does it, both as a stream of progress ending in
//!   one summary, with the words the desktop uses), lists the sources a phone
//!   can read ([`import::import_sources`], [`import::import_kinds_for`]) and
//!   opens the files other apps share (`Library::open_documents`). What an
//!   import writes is announced on [`Library::changes`].
//! - [`capture`] saves what other apps shared into Kanso: the drops the
//!   share extension, App Intents and the Android share activity leave in the
//!   library's `inbox/` folder, each one note in the Inbox. The C entry
//!   `kanso_capture_inbox` in `mobile/rust` calls it.
//! - A note's past and the library's safety nets: [`Library::history`] lists
//!   the saved versions, [`Library::history_diff`] says what changed since
//!   one and [`Library::restore_version`] brings one back (a locked note's
//!   sealed versions are opened with the session's keys, never returned
//!   sealed, and not at all while it is closed). [`Library::replace_preview`],
//!   [`Library::replace_apply`] and [`Library::replace_undo`] are Replace in
//!   all notes, which never touches a locked note. [`Library::backup_if_due`],
//!   [`Library::back_up_now`], [`Library::backups`] and
//!   [`Library::delete_backup`] keep a daily copy of the database inside the
//!   library's folder (so out of the device's backups) and one on request.
//! - [`diagnostics`] has the version, the TLS check and the panic probe.
//!
//! # The edge
//!
//! Every function that can fail returns [`KansoError`], and runs inside
//! [`boundary::guard`]: a panic is caught and returned as
//! [`KansoError::Internal`], and the next call works. Streams are sinks,
//! closures `FnMut(T) -> bool + Send + 'static` that return `false` once
//! nobody listens; the bridge wraps them in its stream type.
//!
//! Secrets (the session, the data key) are kept by Rust in the platform's
//! store ([`secrets`]), never handed to the shell.

mod account;
mod account_methods;
mod account_model;
mod account_provider;
mod account_runtime;
mod account_settings;
mod actions;
mod approvals;
mod approver_model;
pub mod background;
mod backup;
mod backups;
pub mod biometrics;
mod board;
pub mod boundary;
mod builtin_themes;
pub mod capture;
mod comments_model;
pub mod diagnostics;
mod discovery;
mod due;
mod editing;
mod errors;
mod gallery;
mod gallery_model;
mod gallery_net;
mod gallery_store;
mod history;
mod icons;
pub mod import;
mod journal;
mod library;
mod library_admin;
mod linking;
mod linking_model;
mod locks;
mod matching;
mod model;
mod model_history;
mod notes;
pub mod passkey_bridge;
mod passkey_flow;
mod preview;
mod provider_loopback;
mod publish;
mod publish_model;
mod push;
mod reminders;
mod renderer;
mod replace;
mod schedule;
pub mod secrets;
mod sharing;
mod sharing_model;
mod shortcuts;
mod sidebar;
mod stream;
mod sync_model;
mod sync_words;
mod syncing;
mod tasks;
mod theme_tokens;
mod undo;

pub use account::AccountHandle;
pub use account_model::*;
pub use account_settings::AccountSettings;
pub use approvals::Approvals;
pub use backups::BACKUPS_SUMMARY;
pub use comments_model::*;
pub use discovery::{Discovery, IndexerStart};
pub use editing::EditorSession;
pub use errors::KansoError;
pub use gallery::Gallery;
pub use gallery_model::*;
pub use library::Library;
pub use linking::Linking;
pub use linking_model::*;
pub use locks::LockSession;
pub use model::*;
pub use model_history::*;
pub use publish::Publisher;
pub use publish_model::*;
pub use reminders::SCHEDULE_LIMIT;
pub use renderer::Renderer;
pub use replace::replace_apply_label;
pub use sharing::SharedNotes;
pub use sharing_model::*;
pub use shortcuts::shortcut_catalog;
pub use stream::Sink;
pub use sync_model::*;
pub use syncing::SyncHandle;

/// Where secrets live: the seam the Keychain, the Keystore and the tests'
/// memory store stand behind.
pub use kanso_engine::{MemorySecrets, Secrets};
/// The server a library syncs with (the sync client's `Backend`), so a test
/// or the background entry can pass its own.
pub use kanso_sync::Backend;
