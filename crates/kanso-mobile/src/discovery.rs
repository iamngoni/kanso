//! Discovery: finding notes by what they are about and how they link.
//!
//! Related notes turn on a small language model that runs on the phone: it
//! is downloaded once into `models/` beside the library (inside the folder
//! that is kept out of backups), and a thread embeds the notes a few at a
//! time, so the app stays smooth. The embeddings stay in the library's
//! database; nothing about a note is sent anywhere. Whether they are on is
//! whether the store has an embedding model set, so it survives the app
//! closing, and opening the library starts the thread again.
//!
//! A note that is locked or tagged `#ai-exclude` is never embedded (the
//! store sees to that), so it is never a related note, a tag suggestion, a
//! twin or a hit of a search by meaning, and an image only such a note
//! shows is never read. Locked notes are left out of the graph while the
//! lock password hasn't opened them.
//!
//! The rules, the words and the layout are the engine's
//! (`kanso_engine::discovery`), which the desktop uses too; this is the
//! phone's side: the thread, its status stream, and the calls the shell
//! makes.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use kanso_core::tags::is_ai_exclude;
use kanso_engine::discovery::graph::{self as layout, GraphFilter as Narrow, Scope};
use kanso_engine::discovery::indexer::{Indexer, load_static};
use kanso_engine::discovery::meaning::{self, Dismissed, MEANING_MIN_CHARS};
use kanso_engine::discovery::measure::{self, SAMPLE_NOTES};
use kanso_engine::discovery::model_fetch::Model;
use kanso_engine::discovery::ocr;
use kanso_engine::discovery::related::{MODEL, MODEL_ID, Status, related_words, similarity_label};
use kanso_store::{
    MEANING_SIMILARITY, MeaningSearch, Origin, RelatedNote, Store, StoreError, blend_meaning,
};

use crate::boundary::{guard, held};
use crate::errors::KansoError;
use crate::library::{Inner, Library};
use crate::model::{
    DiscoveryMeasure, GraphEdgeRow, GraphFilter, GraphLayout, GraphNodeRow, GraphRequest,
    ImageToRead, MeaningResult, NoteMeaning, NoteView, RelatedRow, RelatedState, RelatedStatus,
    SuggestedTag, TwinRow,
};
use crate::stream::Listeners;

/// How many hops the local graph may reach.
const MAX_HOPS: u8 = 3;

/// Starts the indexer: the library's store, the folder the data lives in
/// and where to tell the statuses. The phone's starts the real one; tests
/// start another with a model of their own.
pub type IndexerStart =
    dyn Fn(Arc<Store>, PathBuf, Box<dyn FnMut(Status) + Send>) -> Indexer + Send + Sync;

/// The real one: fetches the embedding model over HTTPS.
fn real_start(
    store: Arc<Store>,
    data_dir: PathBuf,
    on_status: Box<dyn FnMut(Status) + Send>,
) -> Indexer {
    Indexer::start(store, data_dir, on_status)
}

/// How libraries opened from now on start their indexer: the real one,
/// unless a test put another.
fn default_start() -> Arc<IndexerStart> {
    #[cfg(any(test, feature = "test-support"))]
    if let Some(start) = held(&TEST_START).clone() {
        return start;
    }
    Arc::new(real_start)
}

/// The indexer tests start in place of the real one.
#[cfg(any(test, feature = "test-support"))]
static TEST_START: std::sync::Mutex<Option<Arc<IndexerStart>>> = std::sync::Mutex::new(None);

/// Where related notes stand, and who listens.
struct Shared {
    state: std::sync::Mutex<RelatedState>,
    listeners: Listeners<RelatedState>,
    /// Which start the statuses being told belong to: a status from a
    /// thread that was stopped is not told.
    generation: AtomicU64,
}

/// A library's discovery state.
pub(crate) struct DiscoveryState {
    indexer: std::sync::Mutex<Option<Indexer>>,
    shared: Arc<Shared>,
    start: std::sync::Mutex<Arc<IndexerStart>>,
    /// Pairs of notes said not to be twins, until the library is closed.
    dismissed: std::sync::Mutex<Dismissed>,
}

impl Default for DiscoveryState {
    fn default() -> Self {
        DiscoveryState {
            indexer: std::sync::Mutex::new(None),
            shared: Arc::new(Shared {
                state: std::sync::Mutex::new(RelatedState::default()),
                listeners: Listeners::default(),
                generation: AtomicU64::new(0),
            }),
            start: std::sync::Mutex::new(default_start()),
            dismissed: std::sync::Mutex::new(Dismissed::default()),
        }
    }
}

impl Shared {
    fn set(&self, status: &Status) {
        let state = state_of(status);
        *held(&self.state) = state.clone();
        self.listeners.send(state);
    }
}

/// A status as the shell gets it, with the words it is told in.
fn state_of(status: &Status) -> RelatedState {
    let words = related_words(status);
    RelatedState {
        status: match status {
            Status::Off => RelatedStatus::Off,
            Status::Downloading { done, total } => RelatedStatus::Downloading {
                done: *done,
                total: *total,
            },
            Status::Loading => RelatedStatus::Loading,
            Status::Indexing { done, total } => RelatedStatus::Indexing {
                done: *done as u32,
                total: *total as u32,
            },
            Status::Ready { notes } => RelatedStatus::Ready {
                notes: *notes as u32,
            },
            Status::Failed(message) => RelatedStatus::Failed {
                message: message.clone(),
            },
        },
        title: words.as_ref().map(|(title, _)| title.clone()),
        detail: words.map(|(_, detail)| detail),
    }
}

/// The discovery calls of one library.
#[derive(Clone)]
pub struct Discovery {
    pub(crate) inner: Arc<Inner>,
}

impl Library {
    /// Related notes, notes close in meaning, tags and twins, the graph and
    /// reading the words in images.
    pub fn discovery(&self) -> Discovery {
        Discovery {
            inner: self.inner.clone(),
        }
    }
}

impl Discovery {
    fn library(&self) -> Library {
        Library {
            inner: self.inner.clone(),
        }
    }

    fn store(&self) -> &Store {
        &self.inner.store
    }

    /// The folder the library lives in, which the models folder is in. It is
    /// kept out of backups, and so is everything in it.
    fn data_dir(&self) -> PathBuf {
        data_dir(&self.inner.path)
    }

    /// Where the model's files are kept.
    pub fn model_dir(&self) -> PathBuf {
        MODEL.dir(&self.data_dir())
    }

    // MARK: Related notes on and off

    /// Whether related notes are on.
    pub fn related_enabled(&self) -> Result<bool, KansoError> {
        guard(|| Ok(self.store().embedding_model()?.is_some()))
    }

    /// Turns related notes on or off. On sets the store's embedding model
    /// (so it stays on across launches) and starts the thread that fetches
    /// the model, loads it and keeps notes embedded; turning it on while it
    /// has stopped with an error starts it again. Off stops the thread,
    /// forgets every embedding and keeps the downloaded model.
    pub fn set_related_notes(&self, on: bool) -> Result<(), KansoError> {
        guard(|| {
            if on {
                self.store().set_embedding_model(Some(MODEL_ID))?;
                self.ensure_running();
            } else {
                self.stop_indexer();
                self.inner.discovery.shared.set(&Status::Off);
            }
            Ok(())
        })
    }

    /// Starts the thread again if related notes were left on and none is
    /// running: opening a library does.
    pub(crate) fn resume(&self) {
        if matches!(self.store().embedding_model(), Ok(Some(_))) {
            self.ensure_running();
        }
    }

    fn ensure_running(&self) {
        let mut indexer = held(&self.inner.discovery.indexer);
        if indexer.as_ref().is_some_and(Indexer::is_running) {
            return;
        }
        let shared = self.inner.discovery.shared.clone();
        let generation = shared.generation.fetch_add(1, Ordering::SeqCst) + 1;
        let told = shared.clone();
        let on_status = Box::new(move |status: Status| {
            if told.generation.load(Ordering::SeqCst) == generation {
                told.set(&status);
            }
        });
        let start = held(&self.inner.discovery.start).clone();
        *indexer = Some(start(self.inner.store.clone(), self.data_dir(), on_status));
    }

    /// Stops the thread, waits for it and forgets the embeddings.
    fn stop_indexer(&self) {
        let running = held(&self.inner.discovery.indexer).take();
        // Nothing the stopped thread says after this is told.
        self.inner
            .discovery
            .shared
            .generation
            .fetch_add(1, Ordering::SeqCst);
        match running {
            Some(indexer) => {
                if let Err(err) = indexer.stop(self.store()) {
                    eprintln!("kanso: couldn't turn related notes off: {err}");
                }
            }
            None => {
                if let Err(err) = kanso_engine::discovery::related::forget(self.store()) {
                    eprintln!("kanso: couldn't turn related notes off: {err}");
                }
            }
        }
    }

    /// Where related notes stand now.
    pub fn related_state(&self) -> Result<RelatedState, KansoError> {
        guard(|| Ok(held(&self.inner.discovery.shared.state).clone()))
    }

    /// Calls `sink` with where related notes stand now and again each time
    /// it changes, until the sink returns `false`.
    pub fn related_stream(&self, mut sink: impl FnMut(RelatedState) -> bool + Send + 'static) {
        let shared = &self.inner.discovery.shared;
        if !sink(held(&shared.state).clone()) {
            return;
        }
        shared.listeners.add(sink);
    }

    // MARK: Related notes, tags and twins

    /// The notes closest in meaning to note `id`, closest first, at most
    /// `limit`, without the notes it links to or from. Nothing while related
    /// notes are off or the note isn't embedded yet, and never for a note
    /// that is locked or tagged `#ai-exclude`.
    pub fn related_notes(&self, id: &str, limit: u32) -> Result<Vec<RelatedRow>, KansoError> {
        guard(|| {
            let note = self.library().stored(id)?;
            let Some(model) = self.store().embedding_model()? else {
                return Ok(Vec::new());
            };
            if self.is_hidden(&note)? {
                return Ok(Vec::new());
            }
            let found = self.store().related_notes(id, &model, limit as usize)?;
            Ok(found.unwrap_or_default().iter().map(related_row).collect())
        })
    }

    /// Whether note `id` may not be read by the discovery features: locked
    /// (sealed, or in a locked notebook) or tagged `#ai-exclude`.
    fn is_hidden(&self, note: &kanso_core::Note) -> Result<bool, KansoError> {
        if note.is_locked() || self.store().note_is_locked(&note.id)? {
            return Ok(true);
        }
        for tag in &note.tags {
            if let Some(name) = self.tag_name(tag)?
                && is_ai_exclude(&name)
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn tag_name(&self, id: &str) -> Result<Option<String>, KansoError> {
        Ok(self.store().get(id)?.and_then(|doc| {
            doc.fields
                .get("name")
                .and_then(|name| name.as_str().map(str::to_owned))
        }))
    }

    /// What the embeddings say about note `id`: the tags its nearest notes
    /// have and it hasn't (at most five, heaviest first), and a note that
    /// says nearly the same, unless it was dismissed this session.
    pub fn note_meaning(&self, id: &str) -> Result<NoteMeaning, KansoError> {
        guard(|| {
            let note = self.library().stored(id)?;
            if self.is_hidden(&note)? {
                return Ok(NoteMeaning::default());
            }
            let (suggested, twin) = meaning::read(self.store(), id);
            let tags = meaning::suggested_tags(&suggested, &note.tags)
                .into_iter()
                .map(|s| SuggestedTag {
                    tag_id: s.tag_id,
                    name: s.name,
                    weight: s.weight,
                })
                .collect();
            let dismissed = held(&self.inner.discovery.dismissed).clone();
            let twin = twin
                .filter(|other| !dismissed.has(id, &other.id))
                .map(|other| TwinRow {
                    percent: (other.score.clamp(0., 1.) * 100.).round() as u32,
                    id: other.id,
                    title: other.title,
                });
            Ok(NoteMeaning { tags, twin })
        })
    }

    /// The two notes are different notes after all: not mentioned as twins
    /// again, from either side, until the library is closed.
    pub fn dismiss_twin(&self, id: &str, other_id: &str) -> Result<(), KansoError> {
        guard(|| {
            held(&self.inner.discovery.dismissed).add(id, other_id);
            Ok(())
        })
    }

    /// Merges note `other_id` into note `keep_id`: the kept note gets the
    /// other's words after its own and its tags, and the other goes to the
    /// trash; both versions as they were go into their history. Refused for
    /// a locked note ([`KansoError::Locked`]), and when either changed
    /// since it was read here ([`KansoError::Conflict`]). Returns the kept
    /// note.
    pub fn merge_notes(&self, keep_id: &str, other_id: &str) -> Result<NoteView, KansoError> {
        guard(|| {
            let library = self.library();
            let keep = library.stored(keep_id)?;
            let other = library.stored(other_id)?;
            for note in [&keep, &other] {
                if note.is_locked() || self.store().note_is_locked(&note.id)? {
                    return Err(KansoError::Locked);
                }
            }
            let merged = self.store().merge_notes(
                keep_id,
                keep.rev.as_deref().unwrap_or_default(),
                other_id,
                other.rev.as_deref().unwrap_or_default(),
                Origin::Ui,
            );
            match merged {
                Ok(_) => library.note(keep_id),
                Err(StoreError::Conflict) => Err(KansoError::Conflict {
                    current_rev: library
                        .stored(keep_id)
                        .ok()
                        .and_then(|note| note.rev)
                        .unwrap_or_default(),
                }),
                Err(err) => Err(err.into()),
            }
        })
    }

    // MARK: Search by meaning

    /// The words of `query` worth searching by meaning: plain words with
    /// at least [`MEANING_MIN_CHARS`] letters or digits, no qualifiers.
    pub fn meaning_query(query: &str) -> Option<String> {
        meaning::meaning_query(query)
    }

    /// The fewest letters and digits a search needs to go by meaning.
    pub const MIN_QUERY_CHARS: u32 = MEANING_MIN_CHARS as u32;

    /// The `k` notes closest in meaning to `query`, closest first, those
    /// at least as similar as the desktop's palette keeps. Off while related
    /// notes are, `NoModel` while the model is loading, and nothing found
    /// for a query that isn't plain words ([`Discovery::meaning_query`]).
    pub fn search_by_meaning(&self, query: &str, k: u32) -> Result<MeaningResult, KansoError> {
        guard(|| {
            let Some(words) = meaning::meaning_query(query) else {
                return Ok(match self.store().embedding_model()? {
                    None => MeaningResult::Off,
                    Some(_) => MeaningResult::Found { rows: Vec::new() },
                });
            };
            Ok(match self.store().search_by_meaning(&words, k as usize)? {
                MeaningSearch::Off => MeaningResult::Off,
                MeaningSearch::NoModel => MeaningResult::NoModel,
                MeaningSearch::Found(found) => {
                    let kept = blend_meaning(&[], found, MEANING_SIMILARITY, k as usize);
                    MeaningResult::Found {
                        rows: kept.iter().map(related_row).collect(),
                    }
                }
            })
        })
    }

    // MARK: The graph

    /// The graph for `request`, laid out the same way every time: the
    /// library's (all, one notebook, one tag) or the neighbourhood of a
    /// note. Locked notes and the links to them are left out while the lock
    /// password hasn't opened them; notes tagged `#ai-exclude` stay.
    pub fn graph(&self, request: &GraphRequest) -> Result<GraphLayout, KansoError> {
        guard(|| {
            let show_locked = held(&self.inner.keys).is_unlocked();
            let full = layout::read(self.store(), show_locked)?;
            let books: Vec<(String, Option<String>)> = self
                .store()
                .books()?
                .into_iter()
                .map(|book| (book.id, book.parent_book_id))
                .collect();
            let (scope, center, hops, filter) = match request {
                GraphRequest::Library { filter } => (
                    Scope::Library,
                    None,
                    1,
                    match filter {
                        GraphFilter::All => Narrow::All,
                        GraphFilter::Book { id } => Narrow::Book(id.clone()),
                        GraphFilter::Tag { id } => Narrow::Tag(id.clone()),
                    },
                ),
                GraphRequest::Local { center, hops } => (
                    Scope::Local,
                    Some(center.as_str()),
                    (*hops).clamp(1, MAX_HOPS),
                    Narrow::All,
                ),
            };
            let arranged = layout::arrange(full, scope, center, hops, &filter, &books);
            let opaque = |rgb: u32| 0xFF00_0000 | (rgb & 0x00FF_FFFF);
            let nodes = arranged
                .graph
                .nodes
                .iter()
                .enumerate()
                .map(|(ix, node)| GraphNodeRow {
                    id: node.id.clone(),
                    title: node.title.clone(),
                    notebook_id: node.book_id.clone(),
                    degree: arranged.graph.degree(ix) as u32,
                    x: arranged.pos[ix][0],
                    y: arranged.pos[ix][1],
                    radius: arranged.radii[ix],
                    color_light: opaque(layout::book_color(&node.book_id, false)),
                    color_dark: opaque(layout::book_color(&node.book_id, true)),
                    is_center: center == Some(node.id.as_str()),
                })
                .collect();
            let edges = arranged
                .graph
                .edges
                .iter()
                .map(|&(from, to)| GraphEdgeRow { from, to })
                .collect();
            Ok(GraphLayout {
                nodes,
                edges,
                link_count: arranged.graph.link_count() as u32,
            })
        })
    }

    // MARK: Words in images

    /// The next image to read, newest first, with its bytes; `None` when
    /// none is waiting. Only images a note shows that may be read are
    /// offered. Read it with the platform's reader and give the words to
    /// [`Discovery::ocr_keep`].
    pub fn ocr_next(&self) -> Result<Option<ImageToRead>, KansoError> {
        guard(|| {
            Ok(ocr::next_unread(self.store())?.map(|image| ImageToRead {
                id: image.id,
                md5: image.md5,
                bytes: image.bytes,
            }))
        })
    }

    /// Keeps the words read from image `id`'s bytes (`md5`) by `engine`;
    /// empty `text` for an image that has none or couldn't be read, so it
    /// isn't tried again until its bytes change. `false` when the file is
    /// gone or its bytes changed since: it comes up again.
    pub fn ocr_keep(
        &self,
        id: &str,
        md5: &str,
        text: &str,
        engine: &str,
    ) -> Result<bool, KansoError> {
        guard(|| Ok(ocr::keep(self.store(), id, md5, text, engine)?))
    }

    /// How many images are waiting to be read.
    pub fn ocr_waiting(&self) -> Result<u32, KansoError> {
        guard(|| Ok(ocr::waiting(self.store())? as u32))
    }

    // MARK: What the model costs

    /// Loads the downloaded model, embeds a few hundred sample notes and
    /// says how big it is on disk, how long it took to load, how much
    /// memory the app held before and after, and how fast it embeds. It
    /// doesn't touch the library. The model must be downloaded (turn
    /// related notes on first).
    pub fn measure(&self) -> Result<DiscoveryMeasure, KansoError> {
        guard(|| measure_model(&MODEL, &self.model_dir(), &|dir| load_static(dir)))
    }

    /// Makes every library opened from now on start `start` in place of
    /// the indexer that fetches the model, so a test can run one with a
    /// model of its own. `None` puts the real one back.
    #[cfg(any(test, feature = "test-support"))]
    pub fn use_indexer_for_test(start: Option<Arc<IndexerStart>>) {
        *held(&TEST_START) = start;
    }
}

/// The folder a library file is in, which its models folder is in.
fn data_dir(library: &Path) -> PathBuf {
    library
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

fn related_row(note: &RelatedNote) -> RelatedRow {
    RelatedRow {
        id: note.id.clone(),
        title: note.title.clone(),
        score: note.score,
        label: similarity_label(note.score).to_owned(),
    }
}

/// [`Discovery::measure`] for any model and loader.
pub(crate) fn measure_model(
    model: &Model,
    dir: &Path,
    load: &kanso_engine::discovery::indexer::Load,
) -> Result<DiscoveryMeasure, KansoError> {
    let missing = model
        .missing(dir)
        .map_err(|err| KansoError::storage(err.to_string()))?;
    if !missing.is_empty() {
        return Err(KansoError::invalid(
            "The model isn't downloaded yet. Turn on related notes first.",
        ));
    }
    let texts = measure::sample_texts(SAMPLE_NOTES);
    let measured = measure::measure(model, dir, load, &resident_bytes, &texts)
        .map_err(|err| KansoError::invalid(err.to_string()))?;
    Ok(DiscoveryMeasure {
        model_bytes: measured.model_bytes,
        load_ms: measured.load_ms,
        resident_before: measured.resident_before,
        resident_after: measured.resident_after,
        notes: measured.notes as u32,
        notes_per_second: measured.notes_per_second,
    })
}

/// The memory the app holds in RAM now, in bytes, where the phone says.
pub(crate) fn resident_bytes() -> Option<u64> {
    #[cfg(target_vendor = "apple")]
    {
        apple_resident()
    }
    #[cfg(not(target_vendor = "apple"))]
    {
        proc_resident(&std::fs::read_to_string("/proc/self/status").ok()?)
    }
}

/// `VmRSS` of a `/proc/self/status` (Android and other Linux), in bytes:
/// kilobytes there, whatever the size of a page.
#[cfg_attr(target_vendor = "apple", allow(dead_code))]
pub(crate) fn proc_resident(status: &str) -> Option<u64> {
    let line = status
        .lines()
        .find_map(|line| line.strip_prefix("VmRSS:"))?;
    let kilobytes: u64 = line.trim().strip_suffix("kB")?.trim().parse().ok()?;
    kilobytes.checked_mul(1024)
}

/// `resident_size` of Mach's `task_info`.
#[cfg(target_vendor = "apple")]
fn apple_resident() -> Option<u64> {
    /// `mach_task_basic_info`.
    #[repr(C)]
    #[derive(Default)]
    struct BasicInfo {
        virtual_size: u64,
        resident_size: u64,
        resident_size_max: u64,
        user_time: [i32; 2],
        system_time: [i32; 2],
        policy: i32,
        suspend_count: i32,
    }
    const MACH_TASK_BASIC_INFO: u32 = 20;
    unsafe extern "C" {
        static mach_task_self_: u32;
        fn task_info(task: u32, flavor: u32, info: *mut i32, count: *mut u32) -> i32;
    }
    let mut info = BasicInfo::default();
    let mut count = (size_of::<BasicInfo>() / size_of::<i32>()) as u32;
    // SAFETY: `info` is a `mach_task_basic_info` and `count` its size in
    // words, as the flavor wants; `mach_task_self_` is the process's own
    // task port, set before `main`.
    let status = unsafe {
        task_info(
            mach_task_self_,
            MACH_TASK_BASIC_INFO,
            (&raw mut info).cast(),
            &mut count,
        )
    };
    (status == 0).then_some(info.resident_size)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use kanso_engine::discovery::model_fetch::ModelFile;
    use kanso_store::Embedder;
    use proptest::prelude::*;

    use super::*;

    /// SHA-256 of "abc".
    const ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    const FILES: [ModelFile; 1] = [ModelFile {
        name: "f",
        size: 3,
        sha256: ABC,
    }];
    const TINY: Model = Model {
        repository: "o/m",
        revision: "r",
        folder: "models/m",
        files: &FILES,
    };

    struct Fixed;

    impl Embedder for Fixed {
        fn model(&self) -> &str {
            "fixed"
        }
        fn dim(&self) -> usize {
            2
        }
        fn embed(&self, texts: &[String]) -> Vec<Vec<f32>> {
            texts.iter().map(|_| vec![1., 0.]).collect()
        }
    }

    // no-prop-test: a fake model on disk, measured once; the arithmetic of
    // the measurement has its own properties in the engine.
    #[test]
    fn the_model_is_measured_once_it_is_downloaded() {
        let dir = tempfile::tempdir().unwrap();
        let load = |_: &Path| Ok(Arc::new(Fixed) as Arc<dyn Embedder>);
        assert!(matches!(
            measure_model(&TINY, dir.path(), &load),
            Err(KansoError::Invalid { .. })
        ));
        std::fs::write(dir.path().join("f"), b"abc").unwrap();
        let measured = measure_model(&TINY, dir.path(), &load).unwrap();
        assert_eq!(measured.model_bytes, 3);
        assert_eq!(measured.notes as usize, SAMPLE_NOTES);
        assert!(measured.notes_per_second >= 0.);
        // This process holds memory, and says so where the phone can.
        if cfg!(any(
            target_vendor = "apple",
            target_os = "linux",
            target_os = "android"
        )) {
            assert!(measured.resident_before.is_some_and(|bytes| bytes > 0));
            assert!(measured.resident_after.is_some_and(|bytes| bytes > 0));
        }
    }

    // no-prop-test: the lines of one file.
    #[test]
    fn resident_memory_is_read_from_the_proc_status() {
        let status = "Name:\tkanso\nVmPeak:\t  200 kB\nVmRSS:\t   1234 kB\nThreads:\t4\n";
        assert_eq!(proc_resident(status), Some(1234 * 1024));
        assert_eq!(proc_resident("Name:\tkanso\n"), None);
        assert_eq!(proc_resident("VmRSS:\tlots\n"), None);
    }

    proptest! {
        /// Any size in kilobytes reads back in bytes, however it is padded.
        #[test]
        fn prop_proc_status_sizes_read_in_bytes(kb in 0u64..1_000_000_000, pad in " {0,6}") {
            let status = format!("VmPeak:\t1 kB\nVmRSS:\t{pad}{kb} kB\nVmSwap:\t0 kB\n");
            prop_assert_eq!(proc_resident(&status), Some(kb * 1024));
        }

        /// Every status says its words when it has any, and carries its
        /// numbers across.
        #[test]
        fn prop_a_status_keeps_its_numbers(done in 0u64..100_000, total in 1u64..100_000, notes in 0usize..10_000) {
            let state = state_of(&Status::Downloading { done, total });
            prop_assert_eq!(state.status, RelatedStatus::Downloading { done, total });
            prop_assert!(state.title.is_some() && state.detail.is_some());
            let state = state_of(&Status::Ready { notes });
            prop_assert_eq!(state.status, RelatedStatus::Ready { notes: notes as u32 });
            let off = state_of(&Status::Off);
            prop_assert_eq!(off, RelatedState::default());
        }
    }
}
