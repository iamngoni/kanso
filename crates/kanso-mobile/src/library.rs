//! An open library: its store, its lock session, its sync state and the
//! secrets it signs in with. The note calls are in `notes.rs`; the lock
//! session, sync and renderer handles have their own modules.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex, Weak};
use std::time::Duration;

use kanso_engine::lock::Session as KeySession;
use kanso_engine::sync::SyncState;
use kanso_store::Store;
use kanso_sync::{Backend, HttpBackend, KdfParams};

use crate::backup;
use crate::boundary::{guard, held};
use crate::errors::KansoError;
use crate::locks::LockSession;
use crate::model::{LockEvent, SyncStatus};
use crate::renderer::Renderer;
use crate::secrets;
use crate::stream::Listeners;
use crate::syncing::SyncHandle;
use crate::themes::Themes;
use crate::{Secrets, boundary};

/// How often the auto-lock timer looks at the clock.
const IDLE_CHECK: Duration = Duration::from_secs(5);

/// What every open library is: shared by the handles it hands out.
pub(crate) struct Inner {
    pub(crate) path: PathBuf,
    pub(crate) store: Arc<Store>,
    pub(crate) secrets: Arc<dyn Secrets>,
    pub(crate) backend: Arc<dyn Backend>,
    /// The lock keys typed this session.
    pub(crate) keys: Mutex<KeySession>,
    pub(crate) lock_events: Listeners<LockEvent>,
    /// What the last sync said.
    pub(crate) sync_state: Mutex<SyncState>,
    pub(crate) status_listeners: Listeners<SyncStatus>,
    /// Held for the length of a sync: one runs at a time.
    pub(crate) sync_gate: Mutex<()>,
    /// Set to stop the sync that is running.
    pub(crate) sync_cancel: Mutex<Option<Arc<std::sync::atomic::AtomicBool>>>,
}

/// The libraries open in this process, by their database path. A background
/// sync that starts while the app has the library open reuses it, so the two
/// never sync one library at once.
static OPEN: LazyLock<Mutex<HashMap<PathBuf, Weak<Inner>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// A library opened for reading and writing. Cheap to clone: clones share
/// the library.
#[derive(Clone)]
pub struct Library {
    pub(crate) inner: Arc<Inner>,
}

impl Library {
    /// Opens (creating if needed) the library whose database is the file
    /// `path`, with the phone's secret store and the real server client.
    ///
    /// On Android this refuses until the app has handed over its `Context`
    /// (`KansoNative.init`): a request before that would reach a
    /// certificate verifier that isn't ready.
    ///
    /// A library already open at `path` is returned as it is.
    pub fn open(path: &Path) -> Result<Library, KansoError> {
        guard(|| {
            ready_check()?;
            Library::open_with(path, secrets::platform(), Arc::new(HttpBackend::new()))
        })
    }

    /// [`Library::open`] with the secret store and server client given: the
    /// background entry's, and the tests'.
    pub fn open_with(
        path: &Path,
        secrets: Arc<dyn Secrets>,
        backend: Arc<dyn Backend>,
    ) -> Result<Library, KansoError> {
        guard(|| open_inner(path, secrets, backend, None))
    }

    /// [`Library::open_with`], with cheap key derivation settings for new
    /// lock passwords, so tests derive in milliseconds.
    #[cfg(any(test, feature = "test-support"))]
    pub fn open_for_test(
        path: &Path,
        secrets: Arc<dyn Secrets>,
        backend: Arc<dyn Backend>,
    ) -> Result<Library, KansoError> {
        guard(|| open_inner(path, secrets, backend, Some(KdfParams::TEST)))
    }

    /// The library already open at `path` in this process, if any.
    pub fn existing(path: &Path) -> Option<Library> {
        let key = registry_key(path).ok()?;
        let open = held(&OPEN);
        open.get(&key)
            .and_then(Weak::upgrade)
            .map(|inner| Library { inner })
    }

    /// The database file this library is.
    pub fn path(&self) -> &Path {
        &self.inner.path
    }

    /// The lock session: the lock password's keys.
    pub fn locks(&self) -> LockSession {
        LockSession {
            inner: self.inner.clone(),
        }
    }

    /// Sign-in and sync.
    pub fn sync(&self) -> SyncHandle {
        SyncHandle {
            inner: self.inner.clone(),
        }
    }

    /// Drawing: the preview, highlighting, themes, math and diagrams.
    pub fn renderer(&self) -> Renderer {
        Renderer {
            inner: self.inner.clone(),
        }
    }

    /// The themes: built-in, the person's own and the gallery's, and
    /// importing more.
    pub fn themes(&self) -> Themes {
        Themes {
            inner: self.inner.clone(),
        }
    }

    /// Calls `sink` with the IDs of the notes (and notebooks, and tags) that
    /// changed, in batches, as they are written: by this phone, by a sync,
    /// by the background. Until the sink returns `false` or the library is
    /// closed.
    ///
    /// A thread waits on the store's change channel. It holds no reference
    /// to the library, so closing the library ends it.
    pub fn changes(&self, mut sink: impl FnMut(Vec<String>) -> bool + Send + 'static) {
        let receiver = self.inner.store.subscribe();
        let spawned = std::thread::Builder::new()
            .name("kanso-changes".into())
            .spawn(move || {
                while let Ok(first) = receiver.recv_blocking() {
                    let mut ids = vec![first.id];
                    while let Ok(next) = receiver.try_recv() {
                        if !ids.contains(&next.id) {
                            ids.push(next.id);
                        }
                    }
                    if !sink(ids) {
                        return;
                    }
                }
            });
        if let Err(err) = spawned {
            eprintln!("kanso: couldn't start the change stream: {err}");
        }
    }
}

/// Android needs its `Context` before any request; every platform needs
/// nothing else.
fn ready_check() -> Result<(), KansoError> {
    kanso_tls::ready().map_err(|err| KansoError::invalid(err.to_string()))?;
    #[cfg(target_os = "android")]
    if !secrets::android::installed() {
        return Err(KansoError::invalid(
            "the Keystore isn't set up yet (KansoNative.init)",
        ));
    }
    Ok(())
}

/// The key a path is registered under: its folder made real, so two ways of
/// writing one path meet.
fn registry_key(path: &Path) -> std::io::Result<PathBuf> {
    let name = path
        .file_name()
        .ok_or_else(|| std::io::Error::other("a library path names a file"))?;
    let dir = path.parent().filter(|dir| !dir.as_os_str().is_empty());
    let dir = dir.unwrap_or_else(|| Path::new("."));
    Ok(dir.canonicalize()?.join(name))
}

fn open_inner(
    path: &Path,
    secrets: Arc<dyn Secrets>,
    backend: Arc<dyn Backend>,
    params: Option<KdfParams>,
) -> Result<Library, KansoError> {
    if let Some(dir) = path.parent().filter(|dir| !dir.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir).map_err(|err| KansoError::storage(err.to_string()))?;
    }
    // On every open, so whatever opens the library keeps it out of backups.
    let folder = path
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    backup::exclude(folder).map_err(KansoError::storage)?;
    let key = registry_key(path).map_err(|err| KansoError::storage(err.to_string()))?;
    let mut open = held(&OPEN);
    if let Some(inner) = open.get(&key).and_then(Weak::upgrade) {
        return Ok(Library { inner });
    }
    let store = Arc::new(Store::open(&key)?);
    let mut keys = KeySession::default();
    if let Some(params) = params {
        keys.set_params(params);
    }
    let inner = Arc::new(Inner {
        path: key.clone(),
        store,
        secrets,
        backend,
        keys: Mutex::new(keys),
        lock_events: Listeners::default(),
        sync_state: Mutex::new(SyncState::Off),
        status_listeners: Listeners::default(),
        sync_gate: Mutex::new(()),
        sync_cancel: Mutex::new(None),
    });
    open.retain(|_, inner| inner.strong_count() > 0);
    open.insert(key, Arc::downgrade(&inner));
    drop(open);
    spawn_auto_lock(Arc::downgrade(&inner));
    Ok(Library { inner })
}

/// The timer that closes locked notes once the auto-lock time has passed. It
/// holds the library weakly and ends when the library is closed.
fn spawn_auto_lock(library: Weak<Inner>) {
    let spawned = std::thread::Builder::new()
        .name("kanso-auto-lock".into())
        .spawn(move || {
            loop {
                std::thread::sleep(IDLE_CHECK);
                let Some(inner) = library.upgrade() else {
                    return;
                };
                let session = LockSession { inner };
                // A bug in the check must not end the timer.
                let _ = boundary::guard(|| {
                    session.lock_if_idle(std::time::Instant::now());
                    Ok(())
                });
            }
        });
    if let Err(err) = spawned {
        eprintln!("kanso: couldn't start the auto-lock timer: {err}");
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use kanso_sync::{MemorySecrets, MemoryServer};

    use super::*;

    pub(crate) fn open(dir: &Path) -> Library {
        Library::open_for_test(
            &dir.join("library.db"),
            Arc::new(MemorySecrets::new()),
            Arc::new(MemoryServer::new()),
        )
        .unwrap()
    }

    // no-prop-test: opening is a fixed sequence; the registry's identity is
    // the property and it is checked directly.
    #[test]
    fn a_library_open_twice_is_one_library() {
        let dir = tempfile::tempdir().unwrap();
        let first = open(dir.path());
        let second = Library::open_for_test(
            &dir.path().join("./library.db"),
            Arc::new(MemorySecrets::new()),
            Arc::new(MemoryServer::new()),
        )
        .unwrap();
        assert!(Arc::ptr_eq(&first.inner, &second.inner));
        let found = Library::existing(&dir.path().join("library.db")).unwrap();
        assert!(Arc::ptr_eq(&first.inner, &found.inner));
        drop((first, second, found));
        assert!(Library::existing(&dir.path().join("library.db")).is_none());
    }

    // no-prop-test: three writes and the stream's two sides; the store's
    // change batching is the store's.
    #[test]
    fn the_change_stream_reports_writes_until_its_sink_is_done() {
        use std::sync::atomic::AtomicBool;

        let dir = tempfile::tempdir().unwrap();
        let library = open(dir.path());
        let (tx, rx) = std::sync::mpsc::channel();
        let calls = Arc::new(AtomicUsize::new(0));
        let done = Arc::new(AtomicBool::new(false));
        let (counted, finished) = (calls.clone(), done.clone());
        library.changes(move |ids| {
            counted.fetch_add(1, Ordering::SeqCst);
            tx.send(ids).unwrap();
            !finished.load(Ordering::SeqCst)
        });
        let note = |title: &str| {
            library
                .create(crate::NewNote {
                    title: title.into(),
                    ..Default::default()
                })
                .unwrap()
                .id
        };
        let first = note("First");
        let mut seen = Vec::new();
        while !seen.contains(&first) {
            seen.extend(rx.recv_timeout(Duration::from_secs(5)).unwrap());
        }
        // The sink is told this is the last it will hear: it returns false
        // after the next batch.
        done.store(true, Ordering::SeqCst);
        let second = note("Second");
        while !seen.contains(&second) {
            seen.extend(rx.recv_timeout(Duration::from_secs(5)).unwrap());
        }
        let heard = calls.load(Ordering::SeqCst);
        note("Third");
        std::thread::sleep(Duration::from_millis(300));
        assert_eq!(
            calls.load(Ordering::SeqCst),
            heard,
            "a finished sink hears no more"
        );
    }
}
