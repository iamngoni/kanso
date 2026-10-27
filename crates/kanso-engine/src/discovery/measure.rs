//! What the embedding model costs on a device: how big it is on disk, how
//! long it takes to load, how much memory it holds once loaded and how many
//! notes it embeds a second. The phones run it from a debug screen, to
//! decide whether related notes can stay on by default there.
//!
//! The memory is read through a function the caller gives (`resident`), as
//! the platforms differ.

use std::path::Path;
use std::time::Instant;

use kanso_store::Embedder;

use super::indexer::Load;
use super::model_fetch::{Model, ModelError};

/// How many sample notes are embedded for the speed.
pub const SAMPLE_NOTES: usize = 200;

/// What was measured.
#[derive(Clone, Debug, PartialEq)]
pub struct Measurement {
    /// The model's files on disk, in bytes.
    pub model_bytes: u64,
    /// Checking and loading the model, in milliseconds.
    pub load_ms: u64,
    /// The process's resident memory before and after loading, in bytes,
    /// when the platform says.
    pub resident_before: Option<u64>,
    pub resident_after: Option<u64>,
    /// How many sample notes were embedded.
    pub notes: usize,
    /// Notes embedded a second.
    pub notes_per_second: f64,
}

/// A few hundred words of notes' kind, made the same way every time: each
/// `n`th text is a short paragraph of the words in an order of its own.
pub fn sample_texts(count: usize) -> Vec<String> {
    const WORDS: [&str; 48] = [
        "garden",
        "meeting",
        "budget",
        "recipe",
        "travel",
        "project",
        "draft",
        "invoice",
        "reading",
        "winter",
        "orchard",
        "notes",
        "planning",
        "kitchen",
        "journal",
        "budgeting",
        "sync",
        "release",
        "bridge",
        "design",
        "review",
        "weekend",
        "photos",
        "music",
        "health",
        "training",
        "schedule",
        "launch",
        "feedback",
        "roadmap",
        "sketch",
        "research",
        "ideas",
        "contract",
        "family",
        "studio",
        "market",
        "harvest",
        "report",
        "writing",
        "backup",
        "update",
        "meal",
        "routine",
        "client",
        "workshop",
        "outline",
        "summary",
    ];
    (0..count)
        .map(|n| {
            let mut state = (n as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            let mut next = || {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                state
            };
            let title = WORDS[(next() % 48) as usize];
            let body: Vec<&str> = (0..60).map(|_| WORDS[(next() % 48) as usize]).collect();
            format!("{title} {n}\n{}", body.join(" "))
        })
        .collect()
}

/// The size of the files of `model` in `dir`, for those there.
pub fn bytes_on_disk(model: &Model, dir: &Path) -> u64 {
    model
        .files
        .iter()
        .filter_map(|file| std::fs::metadata(dir.join(file.name)).ok())
        .map(|meta| meta.len())
        .sum()
}

/// Loads the model in `dir` with `load` and embeds `texts`, timing both.
/// The model must be downloaded: a file missing is the error `load` gives.
pub fn measure(
    model: &Model,
    dir: &Path,
    load: &Load,
    resident: &dyn Fn() -> Option<u64>,
    texts: &[String],
) -> Result<Measurement, ModelError> {
    let model_bytes = bytes_on_disk(model, dir);
    let resident_before = resident();
    let started = Instant::now();
    let embedder = load(dir)?;
    let load_ms = started.elapsed().as_millis() as u64;
    let resident_after = resident();
    let started = Instant::now();
    let vectors = embed_all(&*embedder, texts);
    let seconds = started.elapsed().as_secs_f64();
    Ok(Measurement {
        model_bytes,
        load_ms,
        resident_before,
        resident_after,
        notes: vectors,
        notes_per_second: if seconds > 0. {
            vectors as f64 / seconds
        } else {
            0.
        },
    })
}

/// Embeds `texts` as the indexer does, a batch at a time; how many vectors.
fn embed_all(embedder: &dyn Embedder, texts: &[String]) -> usize {
    texts
        .chunks(super::related::BATCH)
        .map(|batch| embedder.embed(batch).len())
        .sum()
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Arc;

    use proptest::prelude::*;

    use super::super::model_fetch::ModelFile;
    use super::*;

    struct Fixed;

    impl Embedder for Fixed {
        fn model(&self) -> &str {
            "fixed"
        }
        fn dim(&self) -> usize {
            2
        }
        fn embed(&self, texts: &[String]) -> Vec<Vec<f32>> {
            texts.iter().map(|t| vec![t.len() as f32, 1.]).collect()
        }
    }

    const FILES: [ModelFile; 1] = [ModelFile {
        name: "weights.bin",
        size: 5,
        sha256: "",
    }];
    const MODEL: Model = Model {
        repository: "o/m",
        revision: "r",
        folder: "m",
        files: &FILES,
    };

    #[test]
    fn a_model_is_measured_with_a_fake() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("weights.bin"), b"12345").unwrap();
        let readings = std::sync::Mutex::new(vec![100u64, 250]);
        let resident = || readings.lock().unwrap().drain(..1).next();
        let texts = sample_texts(SAMPLE_NOTES);
        let measured = measure(
            &MODEL,
            dir.path(),
            &|_: &Path| Ok(Arc::new(Fixed) as Arc<dyn Embedder>),
            &resident,
            &texts,
        )
        .unwrap();
        assert_eq!(measured.model_bytes, 5);
        assert_eq!(measured.notes, SAMPLE_NOTES);
        assert_eq!(
            (measured.resident_before, measured.resident_after),
            (Some(100), Some(250))
        );
        assert!(measured.notes_per_second >= 0.);
    }

    #[test]
    fn a_model_that_will_not_load_is_the_error() {
        let dir = tempfile::tempdir().unwrap();
        let result = measure(
            &MODEL,
            dir.path(),
            &|_: &Path| Err(ModelError::Checksum("weights.bin")),
            &|| None,
            &[],
        );
        assert!(matches!(result, Err(ModelError::Checksum("weights.bin"))));
    }

    proptest! {
        /// The samples are the same every time, and as many as asked for,
        /// each different from the next.
        #[test]
        fn prop_sample_texts_are_stable(count in 0usize..300) {
            let a = sample_texts(count);
            prop_assert_eq!(a.len(), count);
            prop_assert_eq!(&a, &sample_texts(count));
            for pair in a.windows(2) {
                prop_assert_ne!(&pair[0], &pair[1]);
            }
        }

        /// Every sample is embedded, whatever the count.
        #[test]
        fn prop_every_sample_is_embedded(count in 0usize..100) {
            let texts = sample_texts(count);
            prop_assert_eq!(embed_all(&Fixed, &texts), count);
        }
    }
}
