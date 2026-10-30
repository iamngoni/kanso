//! Voice input: the editor's mic. Press it, speak, press it again (or
//! Escape to drop it), and what was said goes into the note at the caret
//! as one step that undoes at once. On this device the words show at the
//! caret while they are said, as text being composed, and the transcript
//! takes their place.
//!
//! Kanso listens one of two ways, picked the first time the mic is pressed
//! and changed in Settings › Editor:
//! - on this device, with NVIDIA's Parakeet speech model (`kanso-voice`),
//!   downloaded once (about 670 MB) from its pinned revision and checked
//!   like related notes' model (`model_fetch.rs`). Nothing said leaves the
//!   device; the microphone is on only while the pill says "Listening".
//! - with the system's own dictation (`dictation.rs`), where there is one.
//!
//! The rules for pressing, cancelling and a transcript coming back are in
//! `voice_state.rs`; this keeps the microphone, the model and the editor
//! the words go to.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;
use kanso_voice::permission::{self, Permission};
use kanso_voice::{Dictation, VoiceError};

use crate::config::VoiceEngine;
use crate::editor::NoteEditor;
use crate::model_fetch::{HttpFetch, Model, ModelError, ModelFile, download};
use crate::voice_state::{Effect, Event, Phase};
use crate::workspace::AppState;

/// Parakeet's files, as `model_fetch` fetches them.
const FILES: [ModelFile; 5] = {
    let theirs = kanso_voice::model::FILES;
    let mut out = [ModelFile {
        name: "",
        size: 0,
        sha256: "",
    }; 5];
    let mut i = 0;
    while i < out.len() {
        out[i] = ModelFile {
            name: theirs[i].name,
            size: theirs[i].size,
            sha256: theirs[i].sha256,
        };
        i += 1;
    }
    out
};

/// The speech model, kept in the data folder's
/// `models/parakeet-tdt-0.6b-v3`.
pub const MODEL: Model = Model {
    repository: kanso_voice::model::REPOSITORY,
    revision: kanso_voice::model::REVISION,
    folder: kanso_voice::model::FOLDER,
    files: &FILES,
};

/// How often the pill's level and time are drawn while listening, and the
/// download's progress while fetching.
const TICK: Duration = Duration::from_millis(80);

/// Where the speech model stands.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum ModelState {
    /// Not looked at yet.
    #[default]
    Unknown,
    Missing,
    /// Fetching it: bytes so far, of all.
    Downloading {
        done: u64,
        total: u64,
    },
    Ready,
    /// The download stopped; what arrived is kept to carry on from.
    Failed(String),
}

/// The app's voice input.
#[derive(Default)]
pub struct Voice {
    pub phase: Phase,
    pub model: ModelState,
    sessions: u64,
    dictation: Option<Dictation>,
    /// The editor the words go to, and its window.
    target: Option<(WeakEntity<NoteEditor>, AnyWindowHandle)>,
    ticker: Option<Task<()>>,
    /// The version of the live words last shown in the editor.
    shown: u64,
    transcribing: Option<Task<()>>,
    download: Option<Task<()>>,
    cancel_download: Arc<AtomicBool>,
    /// The level and time the snapshots draw, with no microphone.
    #[cfg(test)]
    meter: Option<(f32, Duration)>,
}

impl Global for Voice {}

impl Voice {
    /// Whether `editor` is the one listening or transcribing.
    pub fn is_for(&self, editor: &Entity<NoteEditor>) -> bool {
        self.phase.is_active()
            && self
                .target
                .as_ref()
                .is_some_and(|(target, _)| target.entity_id() == editor.entity_id())
    }

    /// How loud it is now, 0 to 1, and how long it has listened.
    pub fn level_and_elapsed(&self) -> Option<(f32, Duration)> {
        #[cfg(test)]
        if let (Some(meter), Phase::Listening { .. }) = (self.meter, self.phase) {
            return Some(meter);
        }
        let dictation = self.dictation.as_ref()?;
        Some((dictation.level(), dictation.elapsed()))
    }

    /// Voice input as the snapshots draw it: `phase` in `editor`, with
    /// `meter` for the pill and `model` for Settings.
    #[cfg(test)]
    pub fn for_snapshot(
        phase: Phase,
        editor: &Entity<NoteEditor>,
        window: AnyWindowHandle,
        meter: (f32, Duration),
        model: ModelState,
    ) -> Voice {
        Voice {
            phase,
            model,
            target: Some((editor.downgrade(), window)),
            meter: Some(meter),
            ..Voice::default()
        }
    }
}

/// Looks at whether the speech model is downloaded, in the background.
// no-prop-test: glue; `Model::missing` has the tests.
pub fn init(cx: &mut App) {
    cx.set_global(Voice::default());
    refresh_model(cx);
}

fn model_dir(cx: &App) -> std::path::PathBuf {
    MODEL.dir(&cx.global::<AppState>().data_dir)
}

// no-prop-test: glue over `Model::missing`.
fn refresh_model(cx: &mut App) {
    let dir = model_dir(cx);
    cx.spawn(async move |cx| {
        let missing = cx
            .background_executor()
            .spawn(async move { MODEL.missing(&dir) })
            .await;
        cx.update(|cx| {
            let voice = cx.global_mut::<Voice>();
            if !matches!(voice.model, ModelState::Downloading { .. }) {
                voice.model = match missing {
                    Ok(files) if files.is_empty() => ModelState::Ready,
                    Ok(_) => ModelState::Missing,
                    Err(err) => ModelState::Failed(err.to_string()),
                };
            }
            cx.refresh_windows();
        });
    })
    .detach();
}

/// The mic, pressed in `editor`: starts or stops listening, or first asks
/// how to listen.
// no-prop-test: glue; `Phase::next` has the rules and their properties.
pub fn press(editor: Entity<NoteEditor>, window: &mut Window, cx: &mut App) {
    let engine = cx.global::<AppState>().config.voice.engine;
    match engine {
        VoiceEngine::System if crate::dictation::AVAILABLE => {
            editor.update(cx, |editor, cx| editor.dictate(window, cx));
        }
        VoiceEngine::Ask => ask(editor, window, cx),
        _ => match cx.global::<Voice>().model.clone() {
            ModelState::Ready => step(Event::Press, Some((editor, window)), cx),
            ModelState::Downloading { .. } => window.push_notification(
                Notification::info("Voice input is ready once the model has downloaded."),
                cx,
            ),
            _ => {
                start_download(cx);
                window.push_notification(
                    Notification::info(
                        "Downloading the voice model. Press the mic again once it's done.",
                    ),
                    cx,
                );
            }
        },
    }
}

/// Escape, or the pill's cancel: stops listening and drops what was said.
// no-prop-test: glue over `Phase::next`.
pub fn cancel(cx: &mut App) {
    step(Event::Cancel, None, cx);
}

/// The first press: on this device, or the system's dictation.
// no-prop-test: dialog glue; the snapshots show it.
fn ask(editor: Entity<NoteEditor>, window: &mut Window, cx: &mut App) {
    let mut choices = vec![crate::dialogs::Described {
        label: "On this device".into(),
        description: format!(
            "Kanso downloads a speech model once (about {}) and turns what you say into text on {}. Nothing you say leaves it.",
            crate::settings_discovery::megabytes(MODEL.total_size()),
            crate::settings_discovery::this_device(),
        )
        .into(),
    }];
    if crate::dictation::AVAILABLE {
        choices.push(crate::dialogs::Described {
            label: system_dictation_name().into(),
            description: "Your system's own dictation, nothing to download. It types where the cursor is, as set up in your system's settings.".into(),
        });
    }
    crate::dialogs::choose(
        "Turn on voice input?",
        "Speak, and what you say goes into the note. You can change this in Settings › Editor.",
        choices,
        window,
        cx,
        move |picked, window, cx| {
            let engine = match picked {
                Some(0) => VoiceEngine::OnDevice,
                Some(_) => VoiceEngine::System,
                None => return,
            };
            AppState::update_config(cx, |config| config.voice.engine = engine);
            press(editor.clone(), window, cx);
        },
    );
}

/// What the system's dictation is called here.
pub fn system_dictation_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "Windows voice typing"
    } else {
        "macOS Dictation"
    }
}

/// Moves voice input on by `event`, doing what the rules say. `from` is
/// the editor (and its window) a press came from.
// no-prop-test: glue; `Phase::next` has the rules and their properties.
fn step(event: Event, from: Option<(Entity<NoteEditor>, &mut Window)>, cx: &mut App) {
    let voice = cx.global::<Voice>();
    let (phase, effect) = voice.phase.next(event, voice.sessions + 1);
    // Listening needs an editor for the words to go to.
    if matches!(effect, Effect::Listen(_)) && from.is_none() {
        return;
    }
    cx.global_mut::<Voice>().phase = phase;
    match effect {
        Effect::None => {}
        Effect::Listen(session) => {
            let Some((editor, window)) = from else {
                return;
            };
            let voice = cx.global_mut::<Voice>();
            voice.sessions = session;
            voice.target = Some((editor.downgrade(), window.window_handle()));
            voice.shown = 0;
            listen(cx);
        }
        Effect::Transcribe => transcribe(cx),
        Effect::Drop => {
            let voice = cx.global_mut::<Voice>();
            if let Some(dictation) = voice.dictation.take() {
                dictation.cancel();
            }
            voice.transcribing = None;
            voice.ticker = None;
            drop_live(cx);
        }
        // The transcript itself goes in where it arrives (`transcribe`).
        Effect::Insert => {}
        Effect::ShowError => {}
    }
    cx.refresh_windows();
}

/// Turns the microphone on, asking for it first where the system does.
// no-prop-test: microphone glue; no observable oracle without a device.
fn listen(cx: &mut App) {
    match permission::status() {
        Permission::Granted => open_microphone(cx),
        Permission::Denied => fail(VoiceError::MicrophoneDenied, cx),
        Permission::Undetermined => {
            let (tx, rx) = async_channel::bounded(1);
            permission::request(move |granted| {
                tx.try_send(granted).ok();
            });
            cx.spawn(async move |cx| {
                let granted = rx.recv().await.unwrap_or(false);
                cx.update(|cx| {
                    if !matches!(cx.global::<Voice>().phase, Phase::Listening { .. }) {
                        return;
                    }
                    if granted {
                        open_microphone(cx);
                    } else {
                        fail(VoiceError::MicrophoneDenied, cx);
                    }
                });
            })
            .detach();
        }
    }
}

// no-prop-test: microphone glue.
fn open_microphone(cx: &mut App) {
    match Dictation::start(&model_dir(cx)) {
        Ok(dictation) => {
            cx.global_mut::<Voice>().dictation = Some(dictation);
            let ticker = cx.spawn(async move |cx| {
                loop {
                    cx.background_executor().timer(TICK).await;
                    let stopped = cx.update(|cx| {
                        let voice = cx.global::<Voice>();
                        let stopped = voice.dictation.as_ref().is_some_and(|d| !d.is_recording());
                        show_live(cx);
                        cx.refresh_windows();
                        stopped
                    });
                    // The device went away, or the half hour ran out: what
                    // was heard is still transcribed.
                    if stopped {
                        cx.update(|cx| step(Event::Press, None, cx));
                        break;
                    }
                }
            });
            cx.global_mut::<Voice>().ticker = Some(ticker);
        }
        Err(err) => fail(err, cx),
    }
}

/// The microphone couldn't start: says why, in the editor's window.
// no-prop-test: glue.
fn fail(err: VoiceError, cx: &mut App) {
    step(Event::MicFailed, None, cx);
    show_error(&err, cx);
}

/// Turns the microphone off and puts what it heard in the note.
// no-prop-test: glue; `voice_edit` and `Phase::next` have the properties.
fn transcribe(cx: &mut App) {
    let voice = cx.global_mut::<Voice>();
    voice.ticker = None;
    let Phase::Transcribing { session } = voice.phase else {
        return;
    };
    let Some(dictation) = voice.dictation.take() else {
        return;
    };
    let done = dictation.stop();
    let transcribing = cx.spawn(async move |cx| {
        let result = done.recv().await.unwrap_or(Err(VoiceError::Cancelled));
        cx.update(|cx| {
            let ok = result.is_ok();
            let current = cx.global::<Voice>().phase;
            let (_, effect) = current.next(Event::Transcribed { session, ok }, 0);
            step(Event::Transcribed { session, ok }, None, cx);
            match (effect, result) {
                (Effect::Insert, Ok(text)) => insert(&text, cx),
                (Effect::ShowError, Err(err)) => {
                    drop_live(cx);
                    show_error(&err, cx);
                }
                _ => drop_live(cx),
            }
        });
    });
    cx.global_mut::<Voice>().transcribing = Some(transcribing);
}

/// Shows the words heard so far in the target editor, when there are new
/// ones.
// no-prop-test: glue over `Dictation::live` and `NoteEditor::show_live_transcript`.
fn show_live(cx: &mut App) {
    let voice = cx.global::<Voice>();
    let Some((version, words)) = voice.dictation.as_ref().map(Dictation::live) else {
        return;
    };
    if version == voice.shown {
        return;
    }
    cx.global_mut::<Voice>().shown = version;
    on_target(cx, move |editor, window, cx| {
        editor.show_live_transcript(&words, window, cx)
    });
}

/// Takes the words heard so far back out of the target editor.
// no-prop-test: glue over `NoteEditor::drop_live_transcript`.
fn drop_live(cx: &mut App) {
    on_target(cx, |editor, window, cx| {
        editor.drop_live_transcript(window, cx)
    });
}

/// Runs `f` on the editor the words go to, in its window, if both remain,
/// as soon as the editor is free.
fn on_target(
    cx: &mut App,
    f: impl FnOnce(&mut NoteEditor, &mut Window, &mut Context<NoteEditor>) + 'static,
) {
    let Some((editor, window)) = cx.global::<Voice>().target.clone() else {
        return;
    };
    // Once the editor is free: Escape cancels from inside its own key
    // handling, and an editor can't be updated inside its own update.
    cx.defer(move |cx| {
        let _ = window.update(cx, |_, window, cx| {
            editor.update(cx, |editor, cx| f(editor, window, cx))
        });
    });
}

/// Puts `text` in the target editor at its caret; if that editor has gone,
/// onto the clipboard, so nothing said is lost.
// no-prop-test: glue over `NoteEditor::insert_transcript`.
fn insert(text: &str, cx: &mut App) {
    let Some((editor, window)) = cx.global::<Voice>().target.clone() else {
        return;
    };
    let text = text.to_owned();
    let put = window.update(cx, |_, window, cx| {
        editor
            .update(cx, |editor, cx| editor.insert_transcript(&text, window, cx))
            .unwrap_or(false)
    });
    if matches!(put, Ok(true)) {
        crate::metrics::record(cx, crate::metrics::Feature::VoiceInput);
    } else {
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        notify(
            Notification::info("The note was closed, so what you said is on the clipboard."),
            cx,
        );
    }
}

/// What went wrong, in words, and whether it is worth saying (a cancel
/// isn't).
pub fn error_words(err: &VoiceError) -> Option<String> {
    Some(match err {
        VoiceError::Cancelled => return None,
        VoiceError::NothingHeard => {
            "Kanso didn't hear anything. Check the microphone and try again.".into()
        }
        VoiceError::NoMicrophone => "There's no microphone to listen with.".into(),
        VoiceError::MicrophoneDenied => {
            if cfg!(target_os = "macos") {
                "Kanso isn't allowed to use the microphone. Turn it on in System Settings › Privacy & Security › Microphone.".into()
            } else {
                "Kanso isn't allowed to use the microphone. Turn it on in your system's privacy settings.".into()
            }
        }
        VoiceError::Device(why) => format!("The microphone stopped working: {why}"),
        VoiceError::Model(why) => format!("The voice model couldn't run: {why}"),
        VoiceError::Audio(why) => format!(
            "Kanso can't read that recording ({why}). It reads M4A, MP3, WAV, AIFF, CAF, FLAC and Ogg Vorbis."
        ),
    })
}

// no-prop-test: glue over `error_words`.
fn show_error(err: &VoiceError, cx: &mut App) {
    if let Some(words) = error_words(err) {
        notify(Notification::error(words), cx);
    }
}

/// Shows `notification` in the window voice input was last used in.
// no-prop-test: glue.
fn notify(notification: Notification, cx: &mut App) {
    if let Some((_, window)) = cx.global::<Voice>().target.clone() {
        window
            .update(cx, |_, window, cx| {
                window.push_notification(notification, cx)
            })
            .ok();
    }
}

/// Fetches the speech model, showing its progress, carrying on from any
/// part already fetched.
// no-prop-test: glue over `model_fetch::download`, which has the properties.
pub fn start_download(cx: &mut App) {
    let dir = model_dir(cx);
    let voice = cx.global_mut::<Voice>();
    if matches!(voice.model, ModelState::Downloading { .. }) {
        return;
    }
    voice.cancel_download.store(true, Ordering::Relaxed);
    let cancel = Arc::new(AtomicBool::new(false));
    voice.cancel_download = cancel.clone();
    voice.model = ModelState::Downloading {
        done: 0,
        total: MODEL.total_size(),
    };
    let progress = Arc::new(std::sync::Mutex::new((0u64, MODEL.total_size())));
    let task = cx.spawn({
        let progress = progress.clone();
        async move |cx| {
            let mut fetched = std::pin::pin!(cx.background_executor().spawn({
                let (cancel, progress) = (cancel.clone(), progress.clone());
                async move {
                    download(
                        &HttpFetch::default(),
                        &MODEL,
                        &dir,
                        &mut |done, total| *progress.lock().unwrap() = (done, total),
                        &cancel,
                    )
                }
            }));
            let result = loop {
                let tick = cx.background_executor().timer(TICK * 3);
                match crate::related::futures_lite_select(&mut fetched, tick).await {
                    Some(result) => break result,
                    None => {
                        let (done, total) = *progress.lock().unwrap();
                        cx.update(|cx| {
                            cx.global_mut::<Voice>().model =
                                ModelState::Downloading { done, total };
                            cx.refresh_windows();
                        });
                    }
                }
            };
            cx.update(|cx| {
                let voice = cx.global_mut::<Voice>();
                voice.model = match result {
                    Ok(()) => ModelState::Ready,
                    Err(ModelError::Cancelled) => ModelState::Missing,
                    Err(err) => ModelState::Failed(err.to_string()),
                };
                voice.download = None;
                cx.refresh_windows();
            });
        }
    });
    cx.global_mut::<Voice>().download = Some(task);
    cx.refresh_windows();
}

/// Stops a download; what arrived is kept for next time.
// no-prop-test: glue.
pub fn cancel_download(cx: &mut App) {
    cx.global::<Voice>()
        .cancel_download
        .store(true, Ordering::Relaxed);
}

/// Deletes the speech model from this device.
// no-prop-test: glue over `model_fetch::remove`, which has the tests.
pub fn remove_model(cx: &mut App) -> std::io::Result<()> {
    cancel_download(cx);
    cancel(cx);
    let dir = model_dir(cx);
    crate::model_fetch::remove(&dir)?;
    cx.global_mut::<Voice>().model = ModelState::Missing;
    cx.refresh_windows();
    Ok(())
}

/// "0:07", "12:30": how long the pill has listened.
pub fn elapsed_label(elapsed: Duration) -> String {
    let secs = elapsed.as_secs();
    format!("{}:{:02}", secs / 60, secs % 60)
}

#[cfg(test)]
mod tests {
    use super::{MODEL, VoiceError, elapsed_label, error_words};
    use proptest::prelude::*;
    use std::time::Duration;

    #[test]
    fn the_model_is_parakeet_on_its_pinned_revision() {
        assert_eq!(MODEL.files.len(), kanso_voice::model::FILES.len());
        for (ours, theirs) in MODEL.files.iter().zip(kanso_voice::model::FILES) {
            assert_eq!(
                (ours.name, ours.size, ours.sha256),
                (theirs.name, theirs.size, theirs.sha256)
            );
            assert_eq!(
                MODEL.file_url(ours.name),
                kanso_voice::model::file_url(ours.name)
            );
        }
        assert_eq!(MODEL.total_size(), kanso_voice::model::total_size());
        assert!(MODEL.file_url("x").starts_with("https://huggingface.co/"));
    }

    #[test]
    fn errors_say_what_to_do() {
        assert_eq!(error_words(&VoiceError::Cancelled), None);
        assert!(
            error_words(&VoiceError::MicrophoneDenied)
                .unwrap()
                .contains("Microphone")
                || !cfg!(target_os = "macos")
        );
        assert_eq!(elapsed_label(Duration::from_secs(7)), "0:07");
        assert_eq!(elapsed_label(Duration::from_secs(750)), "12:30");
    }

    proptest! {
        /// The time reads back as the seconds it was made from.
        #[test]
        fn prop_elapsed_label_round_trips(secs in 0u64..360_000) {
            let label = elapsed_label(Duration::from_secs(secs));
            let (m, s) = label.split_once(':').unwrap();
            prop_assert_eq!(s.len(), 2);
            prop_assert_eq!(m.parse::<u64>().unwrap() * 60 + s.parse::<u64>().unwrap(), secs);
        }
    }
}
