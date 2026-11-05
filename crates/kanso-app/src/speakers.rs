//! Telling speakers apart (issue #38, step 13): the two small models that
//! work out who said what in a recording, about 28 MB, downloaded once
//! when the person turns the setting on and kept in the data folder.
//!
//! The work itself runs inside the transcription (`transcribe.rs`): when
//! the setting is on and both models are here, a finished transcript gets
//! "Speaker 1", "Speaker 2" before its paragraphs. This keeps the models:
//! where they are, whether they are downloaded, fetching them and removing
//! them. Nothing leaves the device but the download.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use gpui_kit::*;
use kanso_voice::diarize::{self, Pinned};

use crate::model_fetch::{HttpFetch, Model, ModelError, ModelFile, download};
use crate::voice::ModelState;
use crate::workspace::AppState;

/// The segmentation model's one file, as `model_fetch` fetches it.
static SEGMENTATION_FILES: [ModelFile; 1] = [file_of(diarize::SEGMENTATION)];
/// The embedding model's one file.
static EMBEDDING_FILES: [ModelFile; 1] = [file_of(diarize::EMBEDDING)];

const fn file_of(pinned: Pinned) -> ModelFile {
    ModelFile {
        name: pinned.name,
        size: pinned.size,
        sha256: pinned.sha256,
    }
}

const fn model_of(pinned: Pinned, files: &'static [ModelFile]) -> Model {
    Model {
        repository: pinned.repository,
        revision: pinned.revision,
        folder: pinned.folder,
        files,
    }
}

/// Who is speaking where, in the data folder's
/// `models/diarize-segmentation-pyannote-3-0`.
pub static SEGMENTATION: Model = model_of(diarize::SEGMENTATION, &SEGMENTATION_FILES);
/// What each voice sounds like, in `models/diarize-embedding-wespeaker-resnet34`.
pub static EMBEDDING: Model = model_of(diarize::EMBEDDING, &EMBEDDING_FILES);

/// How often the download's progress is drawn.
const TICK: Duration = Duration::from_millis(240);

/// Where the speaker models stand.
#[derive(Default)]
pub struct Speakers {
    pub model: ModelState,
    download: Option<Task<()>>,
    cancel: Arc<AtomicBool>,
}

impl Global for Speakers {}

impl Speakers {
    /// The models as the settings snapshots draw them.
    #[cfg(test)]
    pub fn for_snapshot(model: ModelState) -> Speakers {
        Speakers {
            model,
            ..Speakers::default()
        }
    }
}

/// Both models' folders under `data_dir`: segmentation, then embedding.
pub fn dirs(data_dir: &std::path::Path) -> (PathBuf, PathBuf) {
    (SEGMENTATION.dir(data_dir), EMBEDDING.dir(data_dir))
}

/// How many bytes both models download.
pub fn total_size() -> u64 {
    SEGMENTATION.total_size() + EMBEDDING.total_size()
}

/// The folders to hand the speaker models, when the setting is on and both
/// are downloaded.
pub fn ready(cx: &App) -> Option<(PathBuf, PathBuf)> {
    let on = cx.global::<AppState>().config.voice.speakers;
    let here = cx
        .try_global::<Speakers>()
        .is_some_and(|s| s.model == ModelState::Ready);
    (on && here).then(|| dirs(&cx.global::<AppState>().data_dir))
}

/// Looks at whether the models are downloaded, in the background.
// no-prop-test: glue; `Model::missing` has the tests.
pub fn init(cx: &mut App) {
    cx.set_global(Speakers::default());
    let (segmentation, embedding) = dirs(&cx.global::<AppState>().data_dir);
    cx.spawn(async move |cx| {
        let missing = cx
            .background_executor()
            .spawn(async move {
                let a = SEGMENTATION.missing(&segmentation)?;
                let b = EMBEDDING.missing(&embedding)?;
                Ok::<_, std::io::Error>(a.is_empty() && b.is_empty())
            })
            .await;
        cx.update(|cx| {
            let speakers = cx.global_mut::<Speakers>();
            if !matches!(speakers.model, ModelState::Downloading { .. }) {
                speakers.model = match missing {
                    Ok(true) => ModelState::Ready,
                    Ok(false) => ModelState::Missing,
                    Err(err) => ModelState::Failed(err.to_string()),
                };
            }
            cx.refresh_windows();
        });
    })
    .detach();
}

/// Fetches both models, showing their progress together, carrying on from
/// any part already fetched.
// no-prop-test: glue over `model_fetch::download`, which has the properties.
pub fn start_download(cx: &mut App) {
    let (segmentation, embedding) = dirs(&cx.global::<AppState>().data_dir);
    let speakers = cx.default_global::<Speakers>();
    if matches!(speakers.model, ModelState::Downloading { .. }) {
        return;
    }
    speakers.cancel.store(true, Ordering::Relaxed);
    let cancel = Arc::new(AtomicBool::new(false));
    speakers.cancel = cancel.clone();
    speakers.model = ModelState::Downloading {
        done: 0,
        total: total_size(),
    };
    let progress = Arc::new(std::sync::Mutex::new((0u64, total_size())));
    let task = cx.spawn({
        let progress = progress.clone();
        async move |cx| {
            let mut fetched = std::pin::pin!(cx.background_executor().spawn({
                let (cancel, progress) = (cancel.clone(), progress.clone());
                async move {
                    let fetch = HttpFetch::default();
                    let all = total_size();
                    // What the first model already holds counts as done.
                    let first = SEGMENTATION.total_size();
                    download(
                        &fetch,
                        &SEGMENTATION,
                        &segmentation,
                        &mut |done, total| {
                            let held = first.saturating_sub(total);
                            *progress.lock().unwrap() = (held + done, all);
                        },
                        &cancel,
                    )?;
                    download(
                        &fetch,
                        &EMBEDDING,
                        &embedding,
                        &mut |done, total| {
                            let held = EMBEDDING.total_size().saturating_sub(total);
                            *progress.lock().unwrap() = (first + held + done, all);
                        },
                        &cancel,
                    )
                }
            }));
            let result = loop {
                let tick = cx.background_executor().timer(TICK);
                match crate::related::futures_lite_select(&mut fetched, tick).await {
                    Some(result) => break result,
                    None => {
                        let (done, total) = *progress.lock().unwrap();
                        cx.update(|cx| {
                            cx.global_mut::<Speakers>().model =
                                ModelState::Downloading { done, total };
                            cx.refresh_windows();
                        });
                    }
                }
            };
            cx.update(|cx| {
                let speakers = cx.global_mut::<Speakers>();
                speakers.model = match result {
                    Ok(()) => ModelState::Ready,
                    Err(ModelError::Cancelled) => ModelState::Missing,
                    Err(err) => ModelState::Failed(err.to_string()),
                };
                speakers.download = None;
                cx.refresh_windows();
            });
        }
    });
    cx.global_mut::<Speakers>().download = Some(task);
    cx.refresh_windows();
}

/// Stops a download; what arrived is kept for next time.
// no-prop-test: glue.
pub fn cancel_download(cx: &mut App) {
    if let Some(speakers) = cx.try_global::<Speakers>() {
        speakers.cancel.store(true, Ordering::Relaxed);
    }
}

/// Deletes both models from this device.
// no-prop-test: glue over `model_fetch::remove`, which has the tests.
pub fn remove_models(cx: &mut App) -> std::io::Result<()> {
    cancel_download(cx);
    let (segmentation, embedding) = dirs(&cx.global::<AppState>().data_dir);
    crate::model_fetch::remove(&segmentation)?;
    crate::model_fetch::remove(&embedding)?;
    cx.global_mut::<Speakers>().model = ModelState::Missing;
    cx.refresh_windows();
    Ok(())
}

/// The settings row's title and line for `state`. Pure.
pub fn model_words(state: &ModelState) -> (String, String) {
    let size = crate::settings_discovery::megabytes(total_size());
    match state {
        ModelState::Unknown | ModelState::Missing => (
            "Speaker models".to_owned(),
            format!(
                "About {size}, downloaded once. They work out who is talking, on {}.",
                crate::settings_discovery::this_device()
            ),
        ),
        ModelState::Downloading { done, total } => (
            format!(
                "Downloading the speaker models… {}%",
                crate::model_fetch::percent(*done, *total)
            ),
            format!(
                "{} of {}",
                crate::settings_discovery::megabytes(*done),
                crate::settings_discovery::megabytes(*total)
            ),
        ),
        ModelState::Ready => (
            "Speaker models downloaded".to_owned(),
            format!(
                "{size} on {}. {}",
                crate::settings_discovery::this_device(),
                diarize::model::ATTRIBUTION
            ),
        ),
        ModelState::Failed(why) => (
            "The download stopped".to_owned(),
            format!("{why} What arrived is kept, so trying again carries on from there."),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{EMBEDDING, SEGMENTATION, model_words, total_size};
    use crate::voice::ModelState;
    use kanso_voice::diarize;
    use proptest::prelude::*;

    #[test]
    fn the_models_are_the_pinned_ones_and_about_28_megabytes() {
        assert_eq!(total_size(), 28_071_056);
        assert_eq!(SEGMENTATION.files[0].name, "model.int8.onnx");
        assert!(
            EMBEDDING
                .file_url("x")
                .contains(diarize::EMBEDDING.revision)
        );
        assert!(SEGMENTATION.folder != EMBEDDING.folder);
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 32,
            rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f),
            failure_persistence: Some(Box::new(proptest::test_runner::FileFailurePersistence::Off)),
            ..ProptestConfig::default()
        })]

        /// Every state says something, and a failure says why.
        #[test]
        fn prop_every_state_has_words(done in 0u64..30_000_000, why in "[a-z ]{1,20}") {
            for state in [
                ModelState::Unknown,
                ModelState::Missing,
                ModelState::Ready,
                ModelState::Downloading { done, total: 28_071_056 },
                ModelState::Failed(why.clone()),
            ] {
                let (title, detail) = model_words(&state);
                prop_assert!(!title.is_empty() && !detail.is_empty());
            }
            prop_assert!(model_words(&ModelState::Failed(why.clone())).1.contains(&why));
        }
    }
}
