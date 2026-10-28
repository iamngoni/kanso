//! Related notes: notes about the same things as the open one, found by
//! comparing embeddings made on this device.
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
//! local API's `kanso-related-notes` read them without the model. Locked
//! notes and notes tagged `#ai-exclude` are never embedded.
//!
//! While the model is loaded the store borrows it to turn a search into a
//! vector, so the palette (and the local API's `kanso-search-by-meaning`)
//! also finds notes about what was typed, not only notes with its words.
//!
//! What runs it is in `indexer.rs`; the words each state is told in are
//! here, so the desktop's Settings and the phones' say the same.

use std::fs;
use std::path::Path;
use std::time::Duration;

use kanso_store::{Embedder, Store};

pub use super::model_fetch::percent;
use super::model_fetch::{Model, ModelError, ModelFile};
use crate::account::words::format_count;

/// The model, as the store names its embeddings: the Hugging Face repository
/// and the revision its files are pinned to.
pub const MODEL_ID: &str = "minishlab/potion-base-8M@bf8b0566";
/// The repository's commit the files below come from.
const REVISION: &str = "bf8b056651a2c21b8d2565580b8569da283cab23";
const REPOSITORY: &str = "minishlab/potion-base-8M";
/// The model's files at [`REVISION`]. The SHA-256s were taken from the
/// files themselves; for the safetensors file it is also the hash Git LFS
/// records for it.
pub const FILES: [ModelFile; 3] = [
    ModelFile {
        name: "config.json",
        size: 202,
        sha256: "2a6ac0e9aaa356a68a5688070db78fc3a464fefe85d2f06a1905ce3718687553",
    },
    ModelFile {
        name: "tokenizer.json",
        size: 683_666,
        sha256: "e67e803f624fb4d67dea1c730d06e1067e1b14d830e2c2202569e3ef0f70bb50",
    },
    ModelFile {
        name: "model.safetensors",
        size: 30_236_760,
        sha256: "f65d0f325faadc1e121c319e2faa41170d3fa07d8c89abd48ca5358d9a223de2",
    },
];

/// The model, kept in the data folder's `models/potion-base-8M`.
pub const MODEL: Model = Model {
    repository: REPOSITORY,
    revision: REVISION,
    folder: "models/potion-base-8M",
    files: &FILES,
};

/// How many notes are embedded at once, and the pause after each batch.
pub const BATCH: usize = 16;
pub const BATCH_PAUSE: Duration = Duration::from_millis(40);
/// How long notes stay still before changed ones are embedded again.
pub const SETTLE: Duration = Duration::from_millis(1500);
/// How often the download's progress is shown.
pub const PROGRESS_TICK: Duration = Duration::from_millis(150);

/// Where related notes stand.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Status {
    #[default]
    Off,
    /// Fetching the model: bytes so far, of all.
    Downloading { done: u64, total: u64 },
    /// Checking and loading the model.
    Loading,
    /// Embedding notes: how many are done, of all.
    Indexing { done: usize, total: usize },
    /// Every note is embedded.
    Ready { notes: usize },
    /// Something went wrong; Retry starts again.
    Failed(String),
}

/// The model, loaded from `dir` once every file there checks out.
pub struct StaticEmbedder {
    model: model2vec_rs::model::StaticModel,
    dim: usize,
}

impl StaticEmbedder {
    pub fn load(dir: &Path) -> Result<StaticEmbedder, ModelError> {
        if let Some(file) = MODEL.missing(dir)?.first() {
            return Err(ModelError::Checksum(file.name));
        }
        let read = |name: &str| fs::read(dir.join(name));
        // One thread for tokenizing: the indexer runs beside the app, not
        // over it.
        tokenizers::utils::parallelism::set_parallelism(false);
        let model = model2vec_rs::model::StaticModel::from_bytes(
            read("tokenizer.json")?,
            read("model.safetensors")?,
            read("config.json")?,
            None,
        )
        .map_err(|err| ModelError::Load(err.to_string()))?;
        let dim = model.encode_single("kanso").len();
        Ok(StaticEmbedder { model, dim })
    }
}

impl Embedder for StaticEmbedder {
    fn model(&self) -> &str {
        MODEL_ID
    }

    fn dim(&self) -> usize {
        self.dim
    }

    fn embed(&self, texts: &[String]) -> Vec<Vec<f32>> {
        self.model.encode_with_args(texts, Some(256), BATCH)
    }
}

/// Embeds the notes `store` is waiting on, one batch: returns how many
/// are left and how many notes there are that can be embedded.
pub fn embed_batch(store: &Store, embedder: &dyn Embedder) -> kanso_store::Result<(usize, usize)> {
    let queue = store.embedding_queue(embedder.model())?;
    let total = queue.total;
    let waiting = queue.pending.len();
    let batch: Vec<_> = queue.pending.into_iter().take(BATCH).collect();
    let texts: Vec<String> = batch.iter().map(|p| p.text.clone()).collect();
    let vectors = embedder.embed(&texts);
    let rows: Vec<(String, String, Vec<f32>)> = batch
        .into_iter()
        .zip(vectors)
        .map(|(p, v)| (p.id, p.hash, v))
        .collect();
    let kept = rows.len();
    store.put_embeddings(embedder.model(), &rows)?;
    Ok((waiting - kept, total))
}

/// Turns related notes off in `store`: the lent model is taken back and
/// every embedding is forgotten. The downloaded model stays for next time.
pub fn forget(store: &Store) -> kanso_store::Result<()> {
    store.lend_embedder(None);
    store.set_embedding_model(None)
}

/// How close two notes' embeddings are, in words, for the Related tab.
pub fn similarity_label(score: f32) -> &'static str {
    match score {
        s if s >= 0.75 => "Very close",
        s if s >= 0.55 => "Close",
        s if s >= 0.35 => "Related",
        _ => "Loosely related",
    }
}

/// "12.4 MB".
pub fn megabytes(bytes: u64) -> String {
    format!("{:.1}\u{a0}MB", bytes as f64 / 1_000_000.)
}

/// The status row's title and line for related notes in `status`; none
/// while it is off.
pub fn related_words(status: &Status) -> Option<(String, String)> {
    let count = |n: usize| format_count(n as u64);
    Some(match status {
        Status::Off => return None,
        Status::Downloading { done, total } => (
            format!("Downloading the model… {}%", percent(*done, *total)),
            format!("{} of {}", megabytes(*done), megabytes(*total)),
        ),
        Status::Loading => ("Getting ready…".to_owned(), "Checking the model".to_owned()),
        Status::Indexing { done, total } => (
            "Indexing notes…".to_owned(),
            format!("{} of {}", count(*done), count(*total)),
        ),
        Status::Ready { notes } => (
            "Ready".to_owned(),
            format!(
                "{} indexed. New and changed notes are indexed as you write.",
                plural(*notes, "note", "notes")
            ),
        ),
        Status::Failed(message) => ("Related notes stopped".to_owned(), message.clone()),
    })
}

/// "1 note" or "2,841 notes".
fn plural(n: usize, one: &str, many: &str) -> String {
    format!(
        "{} {}",
        format_count(n as u64),
        if n == 1 { one } else { many }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::model_fetch::{download, tests::Memory};
    use proptest::prelude::*;
    use std::collections::HashMap;
    use std::sync::atomic::AtomicBool;

    #[test]
    fn the_pinned_files_are_on_their_revision_over_https() {
        for file in FILES {
            let url = MODEL.file_url(file.name);
            assert!(url.starts_with("https://huggingface.co/minishlab/potion-base-8M/resolve/"));
            assert!(url.contains(REVISION));
            assert_eq!(file.sha256.len(), 64);
            assert!(file.sha256.bytes().all(|b| b.is_ascii_hexdigit()));
        }
        assert!(MODEL_ID.ends_with(&REVISION[..8]));
    }

    /// A tampered file is never kept and never loaded: the download
    /// refuses bytes that aren't the pinned ones, and a file swapped in the
    /// folder afterwards fails the check before the model loads.
    #[test]
    fn a_tampered_model_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let fake: HashMap<String, Vec<u8>> = FILES
            .iter()
            .map(|f| (MODEL.file_url(f.name), vec![b'x'; f.size as usize]))
            .collect();
        let cancel = AtomicBool::new(false);
        let err = download(
            &Memory::new(fake),
            &MODEL,
            dir.path(),
            &mut |_, _| {},
            &cancel,
        )
        .unwrap_err();
        assert!(matches!(err, ModelError::Checksum("config.json")));
        // Nothing was kept, not even the part file.
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
        // A file put there by hand, the right size but other bytes.
        for file in FILES {
            fs::write(dir.path().join(file.name), vec![b'y'; file.size as usize]).unwrap();
        }
        assert_eq!(MODEL.missing(dir.path()).unwrap().len(), 3);
        assert!(matches!(
            StaticEmbedder::load(dir.path()),
            Err(ModelError::Checksum(_))
        ));
    }

    #[test]
    fn related_words_examples() {
        assert_eq!(related_words(&Status::Off), None);
        assert_eq!(
            related_words(&Status::Downloading {
                done: 12_400_000,
                total: 31_121_000
            }),
            Some((
                "Downloading the model… 39%".into(),
                "12.4\u{a0}MB of 31.1\u{a0}MB".into()
            ))
        );
        assert_eq!(
            related_words(&Status::Indexing {
                done: 120,
                total: 2_000
            })
            .unwrap()
            .1,
            "120 of 2,000"
        );
        assert!(
            related_words(&Status::Ready { notes: 1 })
                .unwrap()
                .1
                .starts_with("1 note indexed")
        );
        assert!(
            related_words(&Status::Ready { notes: 2841 })
                .unwrap()
                .1
                .starts_with("2,841 notes indexed")
        );
    }

    proptest! {
        /// Closer scores never read as further apart.
        #[test]
        fn prop_similarity_words_follow_the_score(a in -1f32..1., b in -1f32..1.) {
            let rank = |s: f32| ["Loosely related", "Related", "Close", "Very close"].iter().position(|w| *w == similarity_label(s)).unwrap();
            if a <= b { prop_assert!(rank(a) <= rank(b)); }
        }

        /// Every state but Off says something, and a failure says why.
        #[test]
        fn prop_every_state_reads(done in 0u64..50_000_000, total in 1u64..50_000_000, n in 0usize..100_000, why in "[a-z ]{1,30}") {
            for status in [
                Status::Downloading { done, total },
                Status::Loading,
                Status::Indexing { done: n / 2, total: n },
                Status::Ready { notes: n },
                Status::Failed(why.clone()),
            ] {
                let (title, detail) = related_words(&status).unwrap();
                prop_assert!(!title.is_empty() && !detail.is_empty());
                if let Status::Failed(_) = status {
                    prop_assert_eq!(&detail, &why);
                }
            }
        }

        /// Sizes read with one decimal and never more.
        #[test]
        fn prop_megabytes_have_one_decimal(bytes in 0u64..10_000_000_000) {
            const UNIT: &str = "\u{a0}MB";
            let text = megabytes(bytes);
            let number = text.trim_end_matches(UNIT);
            prop_assert!(text.ends_with(UNIT));
            prop_assert_eq!(number.split('.').nth(1).map(str::len), Some(1));
        }
    }
}
