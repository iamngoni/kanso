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
//! - [`SyncHandle`] (`library.sync()`) signs in to a server, syncs and
//!   reports its status.
//! - [`Renderer`] (`library.renderer()`) draws a note's preview, highlights
//!   source, fits a theme (the built-in ones are in, as the desktop has
//!   them, and source can be colored in any of them) and draws math and
//!   diagrams.
//! - [`Discovery`] (`library.discovery()`) finds notes by meaning and by
//!   links: related notes on a thread of the library's, tags and twins
//!   suggested by the nearest notes, search by meaning, the note graph
//!   laid out, and the words in images read through the phone's reader.
//!   A locked note or one tagged `#ai-exclude` is never embedded.
//! - [`background`] is the sync that runs with no app: the code behind the
//!   C entry `kanso_background_sync` that `mobile/rust` exports.
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

pub mod background;
mod backup;
pub mod boundary;
mod builtin_themes;
pub mod diagnostics;
mod discovery;
mod errors;
mod library;
mod locks;
mod matching;
mod model;
mod notes;
mod renderer;
pub mod secrets;
mod stream;
mod syncing;
mod theme_tokens;

pub use discovery::{Discovery, IndexerStart};
pub use errors::KansoError;
pub use library::Library;
pub use locks::LockSession;
pub use model::*;
pub use renderer::Renderer;
pub use stream::Sink;
pub use syncing::SyncHandle;

/// Where secrets live: the seam the Keychain, the Keystore and the tests'
/// memory store stand behind.
pub use kanso_engine::{MemorySecrets, Secrets};
/// The server a library syncs with (the sync client's `Backend`), so a test
/// or the background entry can pass its own.
pub use kanso_sync::Backend;
