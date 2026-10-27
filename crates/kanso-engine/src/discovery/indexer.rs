//! The indexer: what keeps notes embedded while related notes are on, with
//! no runtime or window, so a phone runs it on a thread of its own.
//!
//! [`run`] fetches the model if it is missing (telling how far it is),
//! loads it, sets the store's embedding model, lends it to the store so
//! searches by meaning work, then embeds the notes that are waiting a batch
//! at a time with a short pause between, and when none is waiting, waits for
//! a note to change and for the changes to settle. It ends when `cancel` is
//! set, or with the error that stopped it. [`Indexer::start`] runs it on a
//! thread and turns an error into [`Status::Failed`].
//!
//! The desktop runs the same steps on its own executor
//! (`kanso-app/src/related.rs`), calling the pieces in [`super::related`].

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use kanso_store::{Embedder, Store};

use super::model_fetch::{Fetch, HttpFetch, Model, ModelError, download};
use super::related::{
    BATCH_PAUSE, MODEL, PROGRESS_TICK, SETTLE, StaticEmbedder, Status, embed_batch, forget,
};

/// How often a waiting indexer looks at the cancel flag.
const IDLE_TICK: Duration = Duration::from_millis(250);

/// Loads a model from the folder its files are in.
pub type Load = dyn Fn(&Path) -> Result<Arc<dyn Embedder>, ModelError> + Send + Sync;

/// The embedding model, loaded from `dir`.
pub fn load_static(dir: &Path) -> Result<Arc<dyn Embedder>, ModelError> {
    Ok(Arc::new(StaticEmbedder::load(dir)?))
}

/// Runs the indexer for `model` in `dir` on this thread until `cancel`: see
/// the module. `on_status` hears each state once, as it begins.
pub fn run_with(
    store: &Arc<Store>,
    model: &Model,
    dir: &Path,
    fetch: &dyn Fetch,
    load: &Load,
    cancel: &AtomicBool,
    on_status: &mut dyn FnMut(Status),
) -> Result<(), ModelError> {
    let mut last: Option<Status> = None;
    let mut say = |status: Status| {
        if last.as_ref() != Some(&status) {
            last = Some(status.clone());
            on_status(status);
        }
    };
    // Changes are heard from now on, so none is missed while the model loads.
    let changes = store.subscribe();
    let needed = model.missing(dir)?;
    if !needed.is_empty() {
        let total: u64 = needed.iter().map(|f| f.size).sum();
        say(Status::Downloading { done: 0, total });
        let mut shown = Instant::now();
        download(
            fetch,
            model,
            dir,
            &mut |done, total| {
                if shown.elapsed() >= PROGRESS_TICK {
                    shown = Instant::now();
                    say(Status::Downloading { done, total });
                }
            },
            cancel,
        )?;
    }
    if cancel.load(Ordering::Relaxed) {
        return Ok(());
    }
    say(Status::Loading);
    let embedder = load(dir)?;
    if cancel.load(Ordering::Relaxed) {
        return Ok(());
    }
    store
        .set_embedding_model(Some(embedder.model()))
        .map_err(|err| ModelError::Load(err.to_string()))?;
    // Searches by meaning turn their words into a vector with it.
    store.lend_embedder(Some(embedder.clone()));
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Ok(());
        }
        let (left, total) =
            embed_batch(store, &*embedder).map_err(|err| ModelError::Load(err.to_string()))?;
        if cancel.load(Ordering::Relaxed) {
            return Ok(());
        }
        if left > 0 {
            say(Status::Indexing {
                done: total.saturating_sub(left),
                total,
            });
            std::thread::sleep(BATCH_PAUSE);
            continue;
        }
        say(Status::Ready { notes: total });
        // Wait for a note to change, then for the changes to settle.
        let mut changed = false;
        while !changed {
            if cancel.load(Ordering::Relaxed) {
                return Ok(());
            }
            match changes.try_recv() {
                Ok(change) => {
                    changed = change.id.starts_with("note:") || change.id.starts_with("tag:")
                }
                Err(async_channel::TryRecvError::Empty) => std::thread::sleep(IDLE_TICK),
                Err(async_channel::TryRecvError::Closed) => return Ok(()),
            }
        }
        loop {
            sleep_unless(SETTLE, cancel);
            if cancel.load(Ordering::Relaxed) {
                return Ok(());
            }
            let mut more = false;
            while let Ok(change) = changes.try_recv() {
                more |= change.id.starts_with("note:") || change.id.starts_with("tag:");
            }
            if !more {
                break;
            }
        }
    }
}

/// Sleeps for `time`, in short steps that stop once `cancel` is set.
fn sleep_unless(time: Duration, cancel: &AtomicBool) {
    let end = Instant::now() + time;
    while Instant::now() < end && !cancel.load(Ordering::Relaxed) {
        std::thread::sleep(IDLE_TICK.min(end.saturating_duration_since(Instant::now())));
    }
}

/// [`run_with`] for the embedding model the apps use, fetched over HTTPS.
pub fn run(
    store: &Arc<Store>,
    data_dir: &Path,
    cancel: &AtomicBool,
    on_status: &mut dyn FnMut(Status),
) -> Result<(), ModelError> {
    run_with(
        store,
        &MODEL,
        &MODEL.dir(data_dir),
        &HttpFetch::default(),
        &load_static,
        cancel,
        on_status,
    )
}

/// An indexer running on a thread of its own.
pub struct Indexer {
    cancel: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Indexer {
    /// Starts [`run`] on a new thread; a failure is told as
    /// [`Status::Failed`] unless it was stopped. `data_dir` is the folder
    /// the model's folder is in.
    pub fn start(
        store: Arc<Store>,
        data_dir: PathBuf,
        on_status: impl FnMut(Status) + Send + 'static,
    ) -> Indexer {
        Indexer::start_with(
            store,
            MODEL,
            MODEL.dir(&data_dir),
            Arc::new(HttpFetch::default()),
            Arc::new(load_static),
            on_status,
        )
    }

    /// [`Indexer::start`] for another model, fetch and loader: the tests'.
    pub fn start_with(
        store: Arc<Store>,
        model: Model,
        dir: PathBuf,
        fetch: Arc<dyn Fetch>,
        load: Arc<Load>,
        mut on_status: impl FnMut(Status) + Send + 'static,
    ) -> Indexer {
        let cancel = Arc::new(AtomicBool::new(false));
        let stop = cancel.clone();
        let spawned = std::thread::Builder::new()
            .name("kanso-indexer".into())
            .spawn(move || {
                let result = run_with(&store, &model, &dir, &*fetch, &*load, &stop, &mut on_status);
                if let Err(err) = result
                    && !stop.load(Ordering::Relaxed)
                {
                    on_status(Status::Failed(err.to_string()));
                }
            });
        let thread = match spawned {
            Ok(thread) => Some(thread),
            Err(err) => {
                eprintln!("kanso: couldn't start the indexer: {err}");
                None
            }
        };
        Indexer { cancel, thread }
    }

    /// Whether the thread is still running.
    pub fn is_running(&self) -> bool {
        self.thread.as_ref().is_some_and(|t| !t.is_finished())
    }

    /// Tells the indexer to stop; it does within a moment.
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    /// Stops the indexer, waits for it to end, then forgets the embeddings:
    /// related notes are off.
    pub fn stop(mut self, store: &Store) -> kanso_store::Result<()> {
        self.cancel();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        forget(store)
    }
}

impl Drop for Indexer {
    /// Dropping an indexer stops it, without waiting.
    fn drop(&mut self) {
        self.cancel();
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use kanso_core::DocType;
    use kanso_store::{MeaningSearch, Origin, Put};
    use serde_json::json;
    use sha2::{Digest as _, Sha256};

    use super::super::model_fetch::{ModelFile, tests::Memory};
    use super::*;

    /// Counts letters: texts with the same letters point the same way.
    struct Letters;

    impl Embedder for Letters {
        fn model(&self) -> &str {
            "letters"
        }
        fn dim(&self) -> usize {
            26
        }
        fn embed(&self, texts: &[String]) -> Vec<Vec<f32>> {
            texts
                .iter()
                .map(|t| {
                    let mut v = vec![0f32; 26];
                    for c in t.to_ascii_lowercase().bytes() {
                        if c.is_ascii_lowercase() {
                            v[(c - b'a') as usize] += 1.;
                        }
                    }
                    v
                })
                .collect()
        }
    }

    /// A model of one small file, and what a server would send for it.
    fn tiny() -> (Model, Memory) {
        let bytes = b"fake weights".to_vec();
        let sha: &'static str = Box::leak(hex(&Sha256::digest(&bytes)).into_boxed_str());
        let files: &'static [ModelFile] = Box::leak(
            vec![ModelFile {
                name: "weights.bin",
                size: bytes.len() as u64,
                sha256: sha,
            }]
            .into_boxed_slice(),
        );
        let model = Model {
            repository: "o/m",
            revision: "r",
            folder: "models/m",
            files,
        };
        let served = Memory::new(HashMap::from([(model.file_url("weights.bin"), bytes)]));
        (model, served)
    }

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    fn note(store: &Store, title: &str, body: &str) -> String {
        let book = match store.books().unwrap().first() {
            Some(book) => book.id.clone(),
            None => {
                store
                    .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
                    .unwrap()
                    .id
            }
        };
        store
            .put(
                DocType::Note,
                json!({"title": title, "body": body, "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

    /// Statuses heard on a channel, until one satisfies `done`.
    fn wait_for(
        heard: &std::sync::mpsc::Receiver<Status>,
        done: impl Fn(&Status) -> bool,
    ) -> Vec<Status> {
        let mut seen = Vec::new();
        loop {
            let status = heard
                .recv_timeout(Duration::from_secs(20))
                .expect("a status");
            let finished = done(&status);
            seen.push(status);
            if finished {
                return seen;
            }
        }
    }

    // no-prop-test: a thread, a download and a wait for changes: the
    // download, the batching and the privacy rules have their own
    // properties.
    #[test]
    fn the_indexer_fetches_loads_embeds_and_follows_changes() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::open(dir.path().join("k.db")).unwrap());
        let first = note(&store, "Orchard", "fruit trees and pruning");
        note(&store, "Pruning", "fruit trees pruned in winter");
        let (model, served) = tiny();
        let models = dir.path().join("models/m");
        let (tx, heard) = std::sync::mpsc::channel();
        let tx = Mutex::new(tx);
        let indexer = Indexer::start_with(
            store.clone(),
            model,
            models.clone(),
            Arc::new(served),
            Arc::new(|_: &Path| Ok(Arc::new(Letters) as Arc<dyn Embedder>)),
            move |status| {
                let _ = tx.lock().unwrap().send(status);
            },
        );
        let seen = wait_for(&heard, |s| matches!(s, Status::Ready { .. }));
        assert!(matches!(
            seen[0],
            Status::Downloading { done: 0, total: 12 }
        ));
        assert!(seen.contains(&Status::Loading));
        assert_eq!(seen.last(), Some(&Status::Ready { notes: 2 }));
        assert!(models.join("weights.bin").exists());
        assert_eq!(store.embedding_model().unwrap().as_deref(), Some("letters"));
        assert!(matches!(
            store.search_by_meaning("fruit trees", 5).unwrap(),
            MeaningSearch::Found(found) if found.len() == 2
        ));
        assert_eq!(
            store
                .related_notes(&first, "letters", 5)
                .unwrap()
                .unwrap()
                .len(),
            1
        );

        // A note written while it waits is embedded once things settle.
        let third = note(&store, "Cider", "apples pressed into cider");
        let seen = wait_for(&heard, |s| *s == Status::Ready { notes: 3 });
        assert!(seen.len() <= 4);
        assert!(store.related_notes(&third, "letters", 5).unwrap().is_some());

        indexer.stop(&store).unwrap();
        assert_eq!(store.embedding_model().unwrap(), None);
        assert!(matches!(
            store.search_by_meaning("fruit", 5).unwrap(),
            MeaningSearch::Off
        ));
    }

    // no-prop-test: the failure paths, one each.
    #[test]
    fn a_failing_download_or_load_is_told_and_stops_the_indexer() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::open(dir.path().join("k.db")).unwrap());
        let (model, _) = tiny();
        // Nothing to fetch it from.
        let (tx, heard) = std::sync::mpsc::channel();
        let tx = Mutex::new(tx);
        let indexer = Indexer::start_with(
            store.clone(),
            model,
            dir.path().join("a"),
            Arc::new(Memory::new(HashMap::new())),
            Arc::new(|_: &Path| Ok(Arc::new(Letters) as Arc<dyn Embedder>)),
            move |status| {
                let _ = tx.lock().unwrap().send(status);
            },
        );
        let seen = wait_for(&heard, |s| matches!(s, Status::Failed(_)));
        assert!(matches!(seen.last(), Some(Status::Failed(why)) if why.contains("404")));
        assert_eq!(store.embedding_model().unwrap(), None);
        indexer.stop(&store).unwrap();

        // Fetched but won't load.
        let (model, served) = tiny();
        let (tx, heard) = std::sync::mpsc::channel();
        let tx = Mutex::new(tx);
        let indexer = Indexer::start_with(
            store.clone(),
            model,
            dir.path().join("b"),
            Arc::new(served),
            Arc::new(|_: &Path| Err(ModelError::Load("bad weights".into()))),
            move |status| {
                let _ = tx.lock().unwrap().send(status);
            },
        );
        let seen = wait_for(&heard, |s| matches!(s, Status::Failed(_)));
        assert!(matches!(seen.last(), Some(Status::Failed(why)) if why.contains("bad weights")));
        assert_eq!(store.embedding_model().unwrap(), None);
        indexer.stop(&store).unwrap();
    }

    // no-prop-test: cancelling before and during a download.
    #[test]
    fn a_cancelled_indexer_ends_quietly() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::open(dir.path().join("k.db")).unwrap());
        let (model, served) = tiny();
        let cancel = AtomicBool::new(true);
        let mut heard = Vec::new();
        let result = run_with(
            &store,
            &model,
            &dir.path().join("m"),
            &served,
            &|_: &Path| Ok(Arc::new(Letters) as Arc<dyn Embedder>),
            &cancel,
            &mut |status| heard.push(status),
        );
        assert!(matches!(result, Err(ModelError::Cancelled)));
        assert_eq!(store.embedding_model().unwrap(), None);
    }
}
