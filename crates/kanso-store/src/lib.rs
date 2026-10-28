//! Kanso's document store.
//!
//! Every document is kept as its JSON, in the local-server API's shape, plus indexed columns
//! derived from it. Writes are serialized through one connection inside
//! `BEGIN IMMEDIATE`; reads use a small pool of connections. Each committed
//! write is broadcast to subscribers as a [`Change`].

mod backup;
pub mod collab;
mod derived;
mod doc;
mod embed;
mod graph;
mod history;
mod meaning;
mod ocr;
mod plugin;
mod query_view;
mod read;
mod replace;
mod roles;
mod schema;
mod sync;
mod tasks;
mod validate;
mod write;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::Connection;

pub use backup::{BackupName, BackupPolicy, backup_name};
pub use collab::model::{
    CommentRecord, Contact, Outgoing, SealedKey, SharedNote, SharedStatus, StoredNotice,
};
pub use collab::{LOCKED_SHARED, SharedText};
pub use derived::{Backlink, DERIVED_VERSION, DERIVED_VERSION_KEY, HeadingHit};
pub use doc::{Attachment, Doc};
pub use embed::{
    Embedder, EmbeddingQueue, MODEL_KEY as EMBED_MODEL_KEY, PendingEmbedding, RelatedNote,
    content_hash, cosine, embed_text, top_k,
};
pub use graph::{GraphLink, GraphNote};
pub use meaning::model::{MeaningSearch, Merged, TagSuggestion};
pub use meaning::rank::{
    DUPLICATE_SIMILARITY, MEANING_SIMILARITY, TAG_NEIGHBOUR_SIMILARITY, blend_meaning, merged_body,
    merged_tags, suggest_tags,
};
pub use ocr::{OCR_TYPES, UnreadImage};
pub use plugin::{
    PLUGIN_DATA_FIELD, PluginCreate, PluginEdit, PluginReport, PluginWriter, plugin_data,
    set_plugin_data,
};
pub use query_view::QueryNoteRow;
pub use read::{NoteSummary, Revision, SearchParams, Sort, StatusCounts};
pub use replace::{
    NoteMatches, PREVIEW_MATCHES_PER_NOTE, Planned, ReplaceReport, Skipped, UndoReport,
};
pub use roles::{DAILY_TEMPLATE_FIELD, DailyNote, FromTemplate, NewNote};
pub use sync::{ApplyOutcome, SyncDoc, SyncGuard, SyncState};
pub use tasks::{CheckOff, ReminderTask, TaskQuery, TaskRow};
pub use write::{Put, WriteResult};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("not found")]
    NotFound,
    #[error("Document update conflict")]
    Conflict,
    #[error("{0}")]
    Invalid(String),
    /// A guarded sync write found the library bound to another account (or
    /// none): its [`SyncGuard`] didn't match `meta`. Nothing was written.
    /// See docs/ACCOUNTS.md › Sync with encryption › The guard.
    #[error("the library is no longer bound to the account this sync started with")]
    GuardFailed,
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T, E = StoreError> = std::result::Result<T, E>;

/// Who made a write. Decides revision snapshots and sync bookkeeping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    /// The desktop UI. Snapshots are throttled so autosave doesn't flood history.
    Ui,
    /// The local HTTP API (MCP clients, scripts). Every write is snapshotted.
    Api,
    /// The Cloudflare sync client applying a remote change.
    Sync,
    /// The PouchDB library importer.
    Import,
    /// `kanso-collab` writing a shared note's merged text. Snapshots are
    /// throttled as for the UI.
    Collab,
    /// An edit an agent suggested and the person applied in Kanso's review
    /// queue. Every write is snapshotted, as for the API, and the version it
    /// replaces names the agent ([`Store::put_by_agent`]).
    Agent,
    /// A plugin's batch ([`Store::apply_plugin_batch`]). Every write is
    /// snapshotted, unthrottled, with the plugin's name, so history can say
    /// which plugin changed what.
    Plugin,
}

impl Origin {
    pub fn as_str(self) -> &'static str {
        match self {
            Origin::Ui => "ui",
            Origin::Api => "api",
            Origin::Sync => "sync",
            Origin::Import => "import",
            Origin::Collab => "collab",
            Origin::Agent => "agent",
            Origin::Plugin => "plugin",
        }
    }
}

/// A committed write, broadcast to subscribers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    pub id: String,
    pub rev: String,
    pub seq: i64,
    pub deleted: bool,
    pub origin: Origin,
}

const READER_POOL: usize = 4;

pub struct Store {
    path: PathBuf,
    writer: Mutex<Connection>,
    readers: Mutex<Vec<Connection>>,
    subscribers: Mutex<Vec<async_channel::Sender<Change>>>,
    /// A note's forgotten text may still be in the write-ahead log: a reader
    /// kept the last write from emptying it, so the next write tries again.
    wal_to_empty: std::sync::atomic::AtomicBool,
    /// The app's embedding model while it is loaded, for turning a search
    /// into a vector ([`Store::lend_embedder`]).
    embedder: std::sync::RwLock<Option<std::sync::Arc<dyn Embedder>>>,
}

impl Store {
    /// Opens (creating if needed) the database at `path` and migrates it.
    pub fn open(path: impl AsRef<Path>) -> Result<Store> {
        let path = path.as_ref().to_path_buf();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let mut writer = schema::connect(&path)?;
        schema::migrate(&writer)?;
        derived::rebuild_if_stale(&mut writer)?;
        ocr::rebuild_if_stale(&mut writer)?;
        Ok(Store {
            path,
            writer: Mutex::new(writer),
            readers: Mutex::new(Vec::new()),
            subscribers: Mutex::new(Vec::new()),
            wal_to_empty: Default::default(),
            embedder: Default::default(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// A channel that receives every committed change from now on.
    pub fn subscribe(&self) -> async_channel::Receiver<Change> {
        let (tx, rx) = async_channel::unbounded();
        self.subscribers
            .lock()
            .expect("subscribers poisoned")
            .push(tx);
        rx
    }

    fn broadcast(&self, changes: &[Change]) {
        if changes.is_empty() {
            return;
        }
        let mut subscribers = self.subscribers.lock().expect("subscribers poisoned");
        subscribers.retain(|tx| changes.iter().all(|c| tx.try_send(c.clone()).is_ok()));
    }

    /// Runs `f` on a pooled read connection.
    fn read<T>(&self, f: impl FnOnce(&Connection) -> Result<T>) -> Result<T> {
        let conn = self.readers.lock().expect("readers poisoned").pop();
        let conn = match conn {
            Some(conn) => conn,
            None => schema::connect(&self.path)?,
        };
        let result = f(&conn);
        let mut readers = self.readers.lock().expect("readers poisoned");
        if readers.len() < READER_POOL {
            readers.push(conn);
        }
        result
    }

    /// Runs `f` inside one `BEGIN IMMEDIATE` transaction on the writer and
    /// broadcasts the changes it reports once the transaction commits.
    fn write<T>(
        &self,
        f: impl FnOnce(&rusqlite::Transaction, &mut Vec<Change>) -> Result<T>,
    ) -> Result<T> {
        use std::sync::atomic::Ordering;
        let mut conn = self.writer.lock().expect("writer poisoned");
        let tx = rusqlite::Transaction::new(&mut conn, rusqlite::TransactionBehavior::Immediate)?;
        let mut changes = Vec::new();
        let written = f(&tx, &mut changes).and_then(|value| Ok(tx.commit().map(|()| value)?));
        let forgot = write::forgot_text();
        let value = written?;
        if forgot || self.wal_to_empty.load(Ordering::Relaxed) {
            self.wal_to_empty
                .store(!empty_wal(&conn), Ordering::Relaxed);
        }
        drop(conn);
        self.broadcast(&changes);
        Ok(value)
    }

    /// Reads a value from the `meta` table.
    pub fn meta(&self, key: &str) -> Result<Option<String>> {
        self.read(|conn| schema::get_meta(conn, key))
    }

    /// Writes a value to the `meta` table.
    pub fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        self.write(|tx, _| schema::set_meta(tx, key, value))
    }
}

/// Moves the write-ahead log into the database and empties it, so a note's
/// forgotten text doesn't linger there (freed pages are overwritten, as
/// `secure_delete` is on). Whether it could: not while a reader holds it.
fn empty_wal(conn: &Connection) -> bool {
    conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| {
        row.get::<_, i64>(0)
    })
    .is_ok_and(|busy| busy == 0)
}

/// An exclusive lock on a data directory, held for the process lifetime.
/// The holder owns the database writer and the API port.
pub struct DataDirLock {
    _file: std::fs::File,
}

/// Takes the data-directory lock, or returns `None` when another process holds it.
pub fn lock_data_dir(dir: &Path) -> Result<Option<DataDirLock>> {
    std::fs::create_dir_all(dir)?;
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(dir.join(".lock"))?;
    match file.try_lock() {
        Ok(()) => Ok(Some(DataDirLock { _file: file })),
        Err(std::fs::TryLockError::WouldBlock) => Ok(None),
        Err(std::fs::TryLockError::Error(err)) => Err(err.into()),
    }
}
