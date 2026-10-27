//! Related notes on the desktop: the switch in Settings, the state the
//! views read and the loop that keeps notes embedded. What related notes
//! are, the model, its download and the batches are the engine's
//! (`kanso_engine::discovery::related`), which the phones run the same
//! pieces of on a thread (`discovery::indexer`); this loop is the same
//! steps on the app's executor.
//!
//! Off until turned on in Settings. Turning it on downloads a small static
//! embedding model once (Model2Vec's potion-base-8M, MIT, about 30 MB) from
//! its pinned revision on Hugging Face into the data folder, and every file
//! is checked against the SHA-256 written here before it is kept or
//! loaded. The model only turns words into numbers on this device: nothing
//! about the notes is ever sent anywhere, and the download asks for the
//! model's files and nothing else.
//!
//! Once loaded, notes are embedded in the background a few at a time,
//! notes worked on most recently first, with pauses between batches so the
//! app stays smooth; a note is embedded again only when its words change
//! (the store keeps a hash of them). The store keeps the vectors
//! (`kanso-store/src/embed.rs`), so the inspector's Related tab and the
//! local API's `kanso-related-notes` read them without the model.
//!
//! While the model is loaded the store borrows it to turn a search into a
//! vector, so the palette (and the local API's `kanso-search-by-meaning`)
//! also finds notes about what was typed, not only notes with its words.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use gpui_kit::{App, Global, Task};
use kanso_store::{Embedder, Store};

use crate::workspace::AppState;

use kanso_engine::discovery::model_fetch::{HttpFetch, ModelError, download};
pub use kanso_engine::discovery::related::*;

/// The app's related-notes state.
#[derive(Default)]
pub struct Related {
    pub status: Status,
    /// Moves on whenever new embeddings are kept, so views ask again.
    pub generation: u64,
    task: Option<Task<()>>,
    cancel: Arc<AtomicBool>,
}

impl Global for Related {}

fn set_status(status: Status, cx: &mut App) {
    let related = cx.default_global::<Related>();
    if related.status != status {
        if matches!(status, Status::Indexing { .. } | Status::Ready { .. }) {
            related.generation += 1;
        }
        related.status = status;
        cx.refresh_windows();
    }
}

/// Starts related notes at launch when they were left on.
pub fn init(cx: &mut App) {
    cx.default_global::<Related>();
    if cx.global::<AppState>().config.discovery.related_notes {
        start(cx);
    }
}

/// Turns related notes on or off, from Settings.
pub fn set_enabled(on: bool, cx: &mut App) {
    AppState::update_config(cx, |config| config.discovery.related_notes = on);
    if on {
        start(cx);
    } else {
        stop(cx);
    }
}

/// Stops whatever is running and forgets the embeddings; the model stays
/// downloaded for next time.
fn stop(cx: &mut App) {
    let related = cx.default_global::<Related>();
    related.cancel.store(true, Ordering::Relaxed);
    related.task = None;
    let store = cx.global::<AppState>().store.clone();
    if let Err(err) = forget(&store) {
        eprintln!("kanso: couldn't turn related notes off: {err}");
    }
    set_status(Status::Off, cx);
}

/// Starts (or starts again, as Retry does): fetches the model if needed,
/// loads it, then keeps the notes embedded.
pub fn start(cx: &mut App) {
    let state = cx.global::<AppState>();
    let store = state.store.clone();
    let dir = MODEL.dir(&state.data_dir);
    let related = cx.default_global::<Related>();
    related.cancel.store(true, Ordering::Relaxed);
    let cancel = Arc::new(AtomicBool::new(false));
    related.cancel = cancel.clone();
    let changes = store.subscribe();
    let task = cx.spawn(async move |cx| {
        let result = run(store, dir, cancel.clone(), changes, cx).await;
        if let Err(err) = result
            && !cancel.load(Ordering::Relaxed)
        {
            cx.update(|cx| set_status(Status::Failed(err.to_string()), cx));
        }
    });
    cx.default_global::<Related>().task = Some(task);
}

/// The steps of `kanso_engine::discovery::indexer::run_with`, on the app's
/// executor: its pieces, with timers the executor owns.
async fn run(
    store: Arc<Store>,
    dir: PathBuf,
    cancel: Arc<AtomicBool>,
    changes: async_channel::Receiver<kanso_store::Change>,
    cx: &mut gpui_kit::AsyncApp,
) -> Result<(), ModelError> {
    // The model: fetched once, its progress shown as it comes.
    let needed = {
        let dir = dir.clone();
        cx.background_executor()
            .spawn(async move { MODEL.missing(&dir) })
            .await?
    };
    if !needed.is_empty() {
        let total: u64 = needed.iter().map(|f| f.size).sum();
        cx.update(|cx| set_status(Status::Downloading { done: 0, total }, cx));
        let progress = Arc::new(Mutex::new((0u64, total)));
        let fetched = {
            let (dir, cancel, progress) = (dir.clone(), cancel.clone(), progress.clone());
            cx.background_executor().spawn(async move {
                download(
                    &HttpFetch::default(),
                    &MODEL,
                    &dir,
                    &mut |done, total| *progress.lock().unwrap() = (done, total),
                    &cancel,
                )
            })
        };
        let mut fetched = std::pin::pin!(fetched);
        loop {
            let tick = cx.background_executor().timer(PROGRESS_TICK);
            match futures_lite_select(&mut fetched, tick).await {
                Some(result) => {
                    result?;
                    break;
                }
                None => {
                    let (done, total) = *progress.lock().unwrap();
                    cx.update(|cx| set_status(Status::Downloading { done, total }, cx));
                }
            }
        }
    }
    cx.update(|cx| set_status(Status::Loading, cx));
    let embedder: Arc<dyn Embedder> = {
        let dir = dir.clone();
        Arc::new(
            cx.background_executor()
                .spawn(async move { StaticEmbedder::load(&dir) })
                .await?,
        )
    };
    store
        .set_embedding_model(Some(MODEL_ID))
        .map_err(|err| ModelError::Load(err.to_string()))?;
    // Searches by meaning turn their words into a vector with it.
    store.lend_embedder(Some(embedder.clone()));
    // A note or a tag changing can change what may be embedded.
    let counts = |id: &str| id.starts_with("note:") || id.starts_with("tag:");
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Ok(());
        }
        let (store2, embedder2) = (store.clone(), embedder.clone());
        let (left, total) = cx
            .background_executor()
            .spawn(async move { embed_batch(&store2, &*embedder2) })
            .await
            .map_err(|err| ModelError::Load(err.to_string()))?;
        if cancel.load(Ordering::Relaxed) {
            return Ok(());
        }
        if left == 0 {
            cx.update(|cx| set_status(Status::Ready { notes: total }, cx));
            // Wait for a note to change, then for the changes to settle.
            loop {
                let Ok(change) = changes.recv().await else {
                    return Ok(());
                };
                if counts(&change.id) {
                    break;
                }
            }
            loop {
                cx.background_executor().timer(SETTLE).await;
                let mut more = false;
                while let Ok(change) = changes.try_recv() {
                    more |= counts(&change.id);
                }
                if !more {
                    break;
                }
            }
        } else {
            let done = total.saturating_sub(left);
            cx.update(|cx| set_status(Status::Indexing { done, total }, cx));
            cx.background_executor().timer(BATCH_PAUSE).await;
        }
    }
}

/// Waits for `work` or `tick`, whichever comes first: `Some` with the
/// work's result, or `None` when the tick came first.
pub(crate) async fn futures_lite_select<T>(
    work: &mut std::pin::Pin<&mut impl std::future::Future<Output = T>>,
    tick: impl std::future::Future,
) -> Option<T> {
    use std::task::Poll;
    let mut tick = std::pin::pin!(tick);
    std::future::poll_fn(move |cx| {
        if let Poll::Ready(value) = work.as_mut().poll(cx) {
            return Poll::Ready(Some(value));
        }
        if tick.as_mut().poll(cx).is_ready() {
            return Poll::Ready(None);
        }
        Poll::Pending
    })
    .await
}
