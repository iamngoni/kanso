//! Transcribing a recording: an audio file dropped into a note, or one
//! already attached, turned into text by the on-device Parakeet model that
//! voice input uses (`voice.rs`), and put in the note under it.
//!
//! A dropped file of up to 10 MB is attached first, as a link, so the
//! recording stays with its words; a longer one is only read, and its
//! transcript goes at the end of the note. One recording is transcribed at
//! a time, on a thread of its own, a piece at a time (`kanso-voice`'s
//! `file`), so the app stays smooth however long it is; the editor shows
//! how far it has got, with Cancel. The first time, Kanso asks before
//! downloading the model, as voice input does.

use std::ops::Range;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;
use kanso_core::links::{FILE_LINK_PREFIX, LINK_SCHEME};
use kanso_core::model::MAX_ATTACHMENT_SIZE;
use kanso_core::{DocType, Note};
use kanso_engine::recording_note::{self, Segment};
use kanso_store::{Origin, Put, Store};
use kanso_voice::VoiceError;
use kanso_voice::file::{self, Progress};

use crate::editor::NoteEditor;
use crate::markdown_edit::Edit;
use crate::voice::{self, ModelState};
use crate::workspace::AppState;

actions!(kanso, [TranscribeAudio]);

/// How often the editor's progress line is drawn.
const TICK: Duration = Duration::from_millis(200);

// MARK: Where the words go

/// The Markdown for an attached recording: a link to it, by name.
pub fn audio_link(file_id: &str, name: &str) -> String {
    let name: String = name
        .chars()
        .map(|c| if matches!(c, '[' | ']') { ' ' } else { c })
        .collect();
    format!("[{}]({LINK_SCHEME}{file_id})", name.trim())
}

/// The attachments `line` links to (not shown as images), by file ID with
/// its `file:` prefix. Pure.
pub fn linked_files(line: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = line;
    let mut offset = 0;
    while let Some(at) = rest.find(FILE_LINK_PREFIX) {
        let start = offset + at;
        let id_start = start + LINK_SCHEME.len();
        let id: String = line[id_start..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '-' | '_'))
            .collect();
        // `![…](…)` is an image; `[…](…)` a link.
        let image = line[..start]
            .rfind("](")
            .filter(|close| *close + 2 == start)
            .and_then(|close| line[..close].rfind('['))
            .is_some_and(|open| open > 0 && line.as_bytes()[open - 1] == b'!');
        if id.len() > "file:".len() && !image && !found.contains(&id) {
            found.push(id);
        }
        offset = id_start;
        rest = &line[id_start..];
    }
    found
}

/// The edit that puts `transcript` in `text` as a paragraph of its own:
/// under the line that links to `file_id`, or, without one, at the end.
/// The caret (`selection`) stays on the same text. Nothing for an empty
/// transcript. Pure.
pub fn transcript_edit(
    text: &str,
    file_id: Option<&str>,
    transcript: &str,
    selection: Range<usize>,
) -> Option<Edit> {
    let words = transcript.trim();
    if words.is_empty() {
        return None;
    }
    // A marked transcript replaces the marked one already in the note.
    let marked = words.starts_with(recording_note::TRANSCRIPT_OPEN);
    let replaced = file_id
        .filter(|_| marked)
        .and_then(|id| recording_note::transcript_range(text, id));
    if let Some(range) = replaced {
        let shift = |offset: usize| {
            let offset = offset.min(text.len());
            if offset <= range.start {
                offset
            } else if offset <= range.end {
                range.start
            } else {
                offset - range.len() + words.len()
            }
        };
        return Some(Edit {
            selection: shift(selection.start)..shift(selection.end),
            range,
            text: words.to_owned(),
        });
    }
    let below = file_id.and_then(|id| {
        let mut start = 0;
        for line in text.split_inclusive('\n') {
            let end = start + line.trim_end_matches(['\n', '\r']).len();
            if linked_files(line).iter().any(|f| f == id) {
                return Some(end);
            }
            start += line.len();
        }
        None
    });
    // Under the link's line, a blank line either side; or at the end, in
    // place of the white space the note ends with.
    let (range, inserted) = match below {
        Some(at) => {
            let after = &text[at..];
            let tail =
                if after.is_empty() || after.starts_with("\n\n") || after.starts_with("\r\n\r\n") {
                    ""
                } else {
                    "\n"
                };
            (at..at, format!("\n\n{words}{tail}"))
        }
        None => {
            let at = text.trim_end().len();
            let lead = if at == 0 { "" } else { "\n\n" };
            (at..text.len(), format!("{lead}{words}\n"))
        }
    };
    let shift = |offset: usize| {
        let offset = offset.min(text.len());
        if offset <= range.start {
            offset
        } else if offset <= range.end {
            range.start
        } else {
            offset - range.len() + inserted.len()
        }
    };
    let selection = shift(selection.start)..shift(selection.end);
    Some(Edit {
        range,
        text: inserted,
        selection,
    })
}

// MARK: The work

/// What to transcribe.
#[derive(Clone, Debug)]
pub enum Source {
    /// An audio attachment already in the library.
    Attachment {
        file_id: String,
        name: String,
        content_type: String,
        /// Present when the open note had to decrypt the attachment first.
        bytes: Option<Vec<u8>>,
    },
    /// A recording made in Kanso, kept as several attachments (its parts, in
    /// playing order) that are read as one timeline. Every timestamp links
    /// to the first part.
    Recording {
        file_ids: Vec<String>,
        duration: Duration,
    },
    /// A file on disk; `file_id` when it was attached too.
    Path {
        path: PathBuf,
        file_id: Option<String>,
        name: String,
    },
}

impl Source {
    fn name(&self) -> &str {
        match self {
            Source::Attachment { name, .. } | Source::Path { name, .. } => name,
            Source::Recording { .. } => "Recording",
        }
    }

    fn file_id(&self) -> Option<&str> {
        match self {
            Source::Recording { file_ids, .. } => file_ids.first().map(String::as_str),
            Source::Attachment { file_id, .. } => Some(file_id),
            Source::Path { file_id, .. } => file_id.as_deref(),
        }
    }
}

struct Job {
    note_id: String,
    source: Source,
    progress: Arc<Mutex<Progress>>,
    cancel: Arc<AtomicBool>,
    editor: WeakEntity<NoteEditor>,
    window: AnyWindowHandle,
    /// Waiting for the voice model to download.
    waiting: bool,
    _task: Option<Task<()>>,
}

/// The recording being transcribed, if any.
#[derive(Default)]
pub struct Transcription {
    job: Option<Job>,
}

impl Global for Transcription {}

/// What the editor shows for note `note_id`'s transcription.
#[derive(Clone, Debug, PartialEq)]
pub struct Shown {
    pub name: String,
    /// How far, 0 to 1, when the file's length is known.
    pub share: Option<f32>,
    /// Waiting for the model to download.
    pub waiting: bool,
}

/// What the editor showing `note_id` says about a transcription into it.
pub fn shown_for(note_id: &str, cx: &App) -> Option<Shown> {
    let job = cx.try_global::<Transcription>()?.job.as_ref()?;
    (job.note_id == note_id).then(|| Shown {
        name: job.source.name().to_owned(),
        share: job.progress.lock().unwrap().share(),
        waiting: job.waiting,
    })
}

/// A transcription into the note `editor` shows, `share` of the way, as
/// the snapshots draw it, with no model running.
#[cfg(test)]
pub fn show_for_snapshot(
    editor: &Entity<NoteEditor>,
    window: AnyWindowHandle,
    name: &str,
    share: f32,
    cx: &mut App,
) {
    let Some(note_id) = editor.read(cx).note_id().map(str::to_owned) else {
        return;
    };
    let total = Duration::from_secs(600);
    let job = Job {
        note_id,
        source: Source::Attachment {
            file_id: "file:snapshot".into(),
            name: name.into(),
            content_type: "audio/wav".into(),
            bytes: None,
        },
        progress: Arc::new(Mutex::new(Progress {
            done: total.mul_f32(share),
            total: Some(total),
        })),
        cancel: Arc::default(),
        editor: editor.downgrade(),
        window,
        waiting: false,
        _task: None,
    };
    cx.default_global::<Transcription>().job = Some(job);
}

/// Stops the transcription; nothing goes into the note.
// no-prop-test: glue.
pub fn cancel(cx: &mut App) {
    if let Some(job) = cx.default_global::<Transcription>().job.take() {
        job.cancel.store(true, Ordering::Relaxed);
        forget_if_recording(&job, cx);
    }
    cx.refresh_windows();
}

/// A recording made in Kanso that is no longer being transcribed is no
/// longer owed a transcript at the next launch.
fn forget_if_recording(job: &Job, cx: &mut App) {
    if matches!(job.source, Source::Recording { .. }) {
        crate::recording::clear_pending(cx);
    }
}

/// Transcribes `source` into the note open in `editor`, asking first to
/// download the model when it isn't here.
// no-prop-test: glue; `transcript_edit` and `kanso_voice::file` have the properties.
pub fn start(editor: Entity<NoteEditor>, source: Source, window: &mut Window, cx: &mut App) {
    let Some(note_id) = editor.read(cx).note_id().map(str::to_owned) else {
        return;
    };
    start_for(note_id, editor, source, window, cx);
}

/// Whether a recording is being transcribed.
pub fn busy(cx: &App) -> bool {
    cx.try_global::<Transcription>()
        .is_some_and(|t| t.job.is_some())
}

/// [`start`] into the note `note_id`, which `editor` is about to show (a
/// note just opened may not be the editor's yet).
// no-prop-test: glue.
pub fn start_for(
    note_id: String,
    editor: Entity<NoteEditor>,
    source: Source,
    window: &mut Window,
    cx: &mut App,
) {
    if cx.default_global::<Transcription>().job.is_some() {
        window.push_notification(
            Notification::info(
                "One recording is transcribed at a time. Cancel it, or wait for it to finish.",
            ),
            cx,
        );
        return;
    }
    let job = Job {
        note_id,
        source,
        progress: Arc::default(),
        cancel: Arc::default(),
        editor: editor.downgrade(),
        window: window.window_handle(),
        waiting: false,
        _task: None,
    };
    let model = cx
        .try_global::<voice::Voice>()
        .map(|voice| voice.model.clone())
        .unwrap_or_default();
    match model {
        ModelState::Ready => run(job, cx),
        ModelState::Downloading { .. } => wait_for_model(job, cx),
        _ => {
            let size = crate::settings_discovery::megabytes(voice::MODEL.total_size());
            crate::dialogs::confirm(
                "Transcribe on this device?",
                format!(
                    "Kanso downloads its speech model once (about {size}) and turns the recording into text on {}. Nothing in it leaves this device. Voice input uses the same model.",
                    crate::settings_discovery::this_device()
                ),
                "Download and Transcribe",
                window,
                cx,
                {
                    let job = Arc::new(Mutex::new(Some(job)));
                    move |_, cx| {
                        if let Some(job) = job.lock().unwrap().take() {
                            voice::start_download(cx);
                            wait_for_model(job, cx);
                        }
                    }
                },
            );
        }
    }
    cx.refresh_windows();
}

/// Holds `job` until the model has downloaded, then runs it.
// no-prop-test: glue.
fn wait_for_model(mut job: Job, cx: &mut App) {
    job.waiting = true;
    let task = cx.spawn(async move |cx| {
        loop {
            cx.background_executor().timer(TICK).await;
            let model = cx.update(|cx| {
                cx.try_global::<voice::Voice>()
                    .map(|voice| voice.model.clone())
                    .unwrap_or_default()
            });
            match model {
                ModelState::Downloading { .. } | ModelState::Unknown => {
                    cx.update(|cx| cx.refresh_windows());
                }
                ModelState::Ready => {
                    cx.update(|cx| {
                        if let Some(mut job) = cx.default_global::<Transcription>().job.take() {
                            job.waiting = false;
                            run(job, cx);
                        }
                    });
                    break;
                }
                ModelState::Missing | ModelState::Failed(_) => {
                    cx.update(|cx| {
                        let job = cx.default_global::<Transcription>().job.take();
                        if let Some(job) = job {
                            forget_if_recording(&job, cx);
                            notify(
                                &job,
                                Notification::error(
                                    "The speech model didn't download, so the recording wasn't transcribed.",
                                ),
                                cx,
                            );
                        }
                        cx.refresh_windows();
                    });
                    break;
                }
            }
        }
    });
    job._task = Some(task);
    cx.default_global::<Transcription>().job = Some(job);
}

/// Transcribes on a thread of its own, drawing progress as it goes.
// no-prop-test: glue over `kanso_voice::file::transcribe`.
fn run(mut job: Job, cx: &mut App) {
    let model_dir = voice::MODEL.dir(&cx.global::<AppState>().data_dir);
    let store = cx.global::<AppState>().store.clone();
    let keys = crate::attachments::FileKeys::of(cx);
    let (progress, cancel) = (job.progress.clone(), job.cancel.clone());
    let source = job.source.clone();
    let (done_tx, done_rx) = async_channel::bounded(1);
    let spawned = std::thread::Builder::new()
        .name("kanso-transcribe".into())
        .spawn(move || {
            let result = match &source {
                Source::Recording { file_ids, duration } => read_parts(&store, &keys, file_ids)
                    .and_then(|parts| {
                        file::transcribe_parts(
                            parts,
                            Some(*duration),
                            &model_dir,
                            &cancel,
                            &mut |at| *progress.lock().unwrap() = at,
                        )
                    }),
                _ => read_source(&store, &keys, &source).and_then(|(media, ext)| {
                    file::transcribe_timed(media, ext.as_deref(), &model_dir, &cancel, &mut |at| {
                        *progress.lock().unwrap() = at;
                    })
                }),
            };
            let _ = done_tx.try_send(result);
        });
    if let Err(err) = spawned {
        notify(
            &job,
            Notification::error(format!("Couldn't start transcribing: {err}")),
            cx,
        );
        return;
    }
    let cancel = job.cancel.clone();
    let task = cx.spawn(async move |cx| {
        let result = loop {
            let tick = cx.background_executor().timer(TICK);
            let mut next = std::pin::pin!(done_rx.recv());
            match crate::related::futures_lite_select(&mut next, tick).await {
                Some(result) => break result.unwrap_or(Err(VoiceError::Cancelled)),
                None => cx.update(|cx| cx.refresh_windows()),
            }
        };
        cx.update(|cx| {
            if cancel.load(Ordering::Relaxed) {
                return;
            }
            let Some(job) = cx.default_global::<Transcription>().job.take() else {
                return;
            };
            forget_if_recording(&job, cx);
            match result {
                Ok(pieces) => insert(&job, &transcript_text(job.source.file_id(), &pieces), cx),
                Err(err) => {
                    if let Some(words) = voice::error_words(&err) {
                        notify(&job, Notification::error(words), cx);
                    }
                }
            }
            cx.refresh_windows();
        });
    });
    job._task = Some(task);
    cx.default_global::<Transcription>().job = Some(job);
}

/// What goes in the note for `pieces`: the timestamped, marked transcript
/// when the recording is attached (so each paragraph can seek it), else
/// plain words. Pure.
pub fn transcript_text(file_id: Option<&str>, pieces: &[file::Timed]) -> String {
    let segments: Vec<Segment> = pieces
        .iter()
        .map(|p| Segment {
            start_secs: p.start.as_secs() as u32,
            text: p.text.clone(),
        })
        .collect();
    match file_id.and_then(|id| recording_note::render_transcript(id, &segments)) {
        Some(block) => block,
        None => kanso_voice::text::join_pieces(pieces.iter().map(|p| p.text.as_str())),
    }
}

/// The parts of a recording made in Kanso, in order, as the decoder reads
/// them.
fn read_parts(
    store: &Store,
    keys: &crate::attachments::FileKeys,
    file_ids: &[String],
) -> Result<Vec<file::Part>, VoiceError> {
    use crate::attachments::{Opened, open_with};
    file_ids
        .iter()
        .map(|id| match open_with(store, id, keys) {
            Opened::Opened(file) => Ok((file::from_bytes(file.bytes), Some("opus".to_owned()))),
            Opened::Locked => Err(VoiceError::Audio(
                "the recording is in a locked note: unlock it first".into(),
            )),
            Opened::Missing => Err(VoiceError::Audio("a part of the recording is gone".into())),
        })
        .collect()
}

/// The recording to read, and its extension as a hint.
fn read_source(
    store: &Store,
    keys: &crate::attachments::FileKeys,
    source: &Source,
) -> Result<(Box<dyn file::MediaSource>, Option<String>), VoiceError> {
    match source {
        Source::Attachment {
            file_id,
            content_type,
            bytes,
            ..
        } => {
            let bytes = match bytes {
                Some(bytes) => bytes.clone(),
                None => match crate::attachments::open_with(store, file_id, keys) {
                    crate::attachments::Opened::Opened(file) => file.bytes,
                    crate::attachments::Opened::Locked => {
                        return Err(VoiceError::Audio(
                            "the recording is in a locked note: unlock it first".into(),
                        ));
                    }
                    crate::attachments::Opened::Missing => {
                        return Err(VoiceError::Audio("the attachment is gone".into()));
                    }
                },
            };
            let ext = file::extension(content_type).map(str::to_owned);
            Ok((file::from_bytes(bytes), ext))
        }
        Source::Path { path, .. } => {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(str::to_ascii_lowercase);
            Ok((file::from_path(path)?, ext))
        }
        Source::Recording { .. } => Err(VoiceError::Audio(
            "a recording in parts is read part by part".into(),
        )),
    }
}

/// Puts the transcript in the note: through the editor while it shows the
/// note, so it undoes as one step, or else straight into the store.
// no-prop-test: glue over `transcript_edit`.
fn insert(job: &Job, text: &str, cx: &mut App) {
    let file_id = job.source.file_id().map(str::to_owned);
    let text = text.to_owned();
    let editor = job.editor.upgrade();
    let shown = editor
        .as_ref()
        .is_some_and(|e| e.read(cx).note_id() == Some(job.note_id.as_str()));
    if let (Some(editor), true) = (editor, shown) {
        let put = job.window.update(cx, |_, window, cx| {
            editor.update(cx, |editor, cx| {
                editor.insert_transcript_below(file_id.as_deref(), &text, window, cx)
            })
        });
        if matches!(put, Ok(true)) {
            return;
        }
    }
    let store = cx.global::<AppState>().store.clone();
    if let Err(err) = insert_into_store(&store, &job.note_id, file_id.as_deref(), &text) {
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        notify(
            job,
            Notification::error(format!(
                "Couldn't put the transcript in the note ({err}), so it's on the clipboard."
            )),
            cx,
        );
    }
}

/// [`transcript_edit`] on the note as stored.
fn insert_into_store(
    store: &Store,
    note_id: &str,
    file_id: Option<&str>,
    text: &str,
) -> Result<(), String> {
    let note: Note = store
        .note(note_id)
        .map_err(|e| e.to_string())?
        .ok_or("the note is gone")?;
    if note.is_locked() {
        return Err("the note is locked".into());
    }
    let Some(edit) = transcript_edit(&note.body, file_id, text, 0..0) else {
        return Ok(());
    };
    let mut body = note.body.clone();
    body.replace_range(edit.range, &edit.text);
    store
        .put(
            DocType::Note,
            serde_json::json!({"_id": note.id, "_rev": note.rev, "body": body}),
            Put::Merge,
            Origin::Ui,
        )
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Attaches the audio file at `path` when it is small enough: its file ID
/// and the link to write, or `None` for a file over the limit.
pub fn attach_audio(
    store: &Store,
    path: &std::path::Path,
) -> Result<Option<(String, String)>, String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default();
    let content_type = file::content_type(ext).ok_or("That isn't an audio file Kanso reads.")?;
    let size = std::fs::metadata(path).map_err(|e| e.to_string())?.len();
    if size > MAX_ATTACHMENT_SIZE as u64 {
        return Ok(None);
    }
    let bytes = std::fs::read(path).map_err(|e| format!("Couldn't read the recording: {e}"))?;
    let name: String = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("recording")
        .chars()
        .take(120)
        .collect();
    use base64::Engine as _;
    let doc = serde_json::json!({
        "name": name,
        "contentType": content_type,
        "publicIn": [],
        "_attachments": {"index": {
            "content_type": content_type,
            "data": base64::engine::general_purpose::STANDARD.encode(&bytes),
        }},
    });
    let id = store
        .put(DocType::File, doc, Put::Upsert, Origin::Ui)
        .map_err(|e| format!("Couldn't attach the recording: {e}"))?
        .id;
    let link = audio_link(&id, &name);
    Ok(Some((id, link)))
}

/// Whether `path` is an audio file Kanso reads, by its extension.
pub fn is_audio(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| file::content_type(e).is_some())
}

fn notify(job: &Job, notification: Notification, cx: &mut App) {
    job.window
        .update(cx, |_, window, cx| {
            window.push_notification(notification, cx)
        })
        .ok();
}

#[cfg(test)]
mod tests {
    use super::{
        Edit, attach_audio, audio_link, insert_into_store, is_audio, linked_files, transcript_edit,
    };
    use kanso_core::DocType;
    use kanso_store::{Origin, Put, Store};
    use proptest::prelude::*;

    fn apply(text: &str, edit: &Edit) -> String {
        format!(
            "{}{}{}",
            &text[..edit.range.start],
            edit.text,
            &text[edit.range.end..]
        )
    }

    #[test]
    fn links_to_recordings_are_found_and_images_are_not() {
        let link = audio_link("file:abc123", "Memo [1].m4a");
        assert_eq!(link, "[Memo  1 .m4a](inkdrop://file:abc123)");
        assert_eq!(linked_files(&link), ["file:abc123"]);
        assert_eq!(
            linked_files("![](inkdrop://file:img1) and [a](inkdrop://file:a2)"),
            ["file:a2"]
        );
        assert!(linked_files("[note](inkdrop://note:x)").is_empty());
    }

    #[test]
    fn the_transcript_goes_under_its_recording() {
        let text = "Intro\n\n[memo.m4a](inkdrop://file:m1)\nNext line";
        let edit = transcript_edit(text, Some("file:m1"), " Hello there. ", 0..0).unwrap();
        assert_eq!(
            apply(text, &edit),
            "Intro\n\n[memo.m4a](inkdrop://file:m1)\n\nHello there.\n\nNext line"
        );
        // No recording linked: at the end, after a blank line.
        let edit = transcript_edit("Intro\n\n", Some("file:zz"), "Words", 2..2).unwrap();
        assert_eq!(apply("Intro\n\n", &edit), "Intro\n\nWords\n");
        assert_eq!(edit.selection, 2..2);
        assert_eq!(transcript_edit("x", None, "  ", 0..0), None);
        let edit = transcript_edit("", None, "Words", 0..0).unwrap();
        assert_eq!(apply("", &edit), "Words\n");
    }

    #[test]
    fn the_store_takes_every_format_kanso_reads() {
        for (_, content_type) in kanso_voice::file::FORMATS {
            assert!(kanso_core::model::SUPPORTED_AUDIO_TYPES.contains(&content_type));
        }
    }

    /// A transcript lands in a note that isn't open, under its recording.
    #[test]
    fn a_transcript_reaches_a_note_that_isnt_open() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("k.db")).unwrap();
        let book = store
            .put(
                DocType::Book,
                serde_json::json!({"name": "B"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let wav = dir.path().join("memo.wav");
        std::fs::write(&wav, b"RIFF").unwrap();
        let (file_id, link) = attach_audio(&store, &wav).unwrap().unwrap();
        let note = store
            .put(
                DocType::Note,
                serde_json::json!({"title": "t", "body": format!("{link}\nafter"), "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        insert_into_store(&store, &note, Some(&file_id), "Spoken words.").unwrap();
        assert_eq!(
            store.note(&note).unwrap().unwrap().body,
            format!("{link}\n\nSpoken words.\n\nafter")
        );
        let crate::attachments::Opened::Opened(file) = crate::attachments::open_with(
            &store,
            &file_id,
            &crate::attachments::FileKeys::default(),
        ) else {
            panic!("the audio opens");
        };
        assert_eq!(file.content_type, "audio/wav");
        assert!(is_audio(&wav) && !is_audio(std::path::Path::new("a.png")));
    }

    #[test]
    fn a_marked_transcript_replaces_the_old_one_and_only_it() {
        use super::{file::Timed, transcript_text};
        use std::time::Duration;
        let piece = |secs, text: &str| Timed {
            start: Duration::from_secs(secs),
            text: text.into(),
        };
        let first = transcript_text(Some("file:m1"), &[piece(0, "One."), piece(20, "Two.")]);
        assert!(first.contains("[00:20](kanso-audio://file:m1?t=20) Two."));
        let text = format!("[a](inkdrop://file:m1)\n\nMine\n\n{first}\n\nTail");
        let second = transcript_text(Some("file:m1"), &[piece(0, "New.")]);
        let edit = transcript_edit(&text, Some("file:m1"), &second, 0..0).unwrap();
        assert_eq!(
            apply(&text, &edit),
            format!("[a](inkdrop://file:m1)\n\nMine\n\n{second}\n\nTail")
        );
        // Without a recording linked there are no timestamps.
        assert_eq!(
            transcript_text(None, &[piece(0, "A."), piece(9, "B.")]),
            "A. B."
        );
        // Without markers in the note, a marked one goes under the link.
        let edit =
            transcript_edit("[a](inkdrop://file:m1)", Some("file:m1"), &second, 0..0).unwrap();
        assert!(apply("[a](inkdrop://file:m1)", &edit).ends_with(&second));
    }

    fn check_placement(
        text: &str,
        file: usize,
        words: &str,
        caret: usize,
    ) -> Result<(), TestCaseError> {
        let id = format!("file:f{file}");
        let caret = caret.min(text.len());
        let edit = transcript_edit(text, Some(&id), words, caret..caret).unwrap();
        let out = apply(text, &edit);
        // What was there stays, in order: only white space at the very
        // end may be swapped.
        prop_assert!(out.starts_with(&text[..edit.range.start]));
        prop_assert!(out.ends_with(&text[edit.range.end..]));
        prop_assert!(text[edit.range.clone()].trim().is_empty());
        prop_assert!(out.contains(words));
        let linked = text.lines().any(|l| linked_files(l).contains(&id));
        if linked {
            let before = &out[..out.find(words).unwrap()];
            let link_line = before.trim_end().lines().last().unwrap_or("");
            prop_assert!(linked_files(link_line).contains(&id));
        } else {
            prop_assert!(out.trim_end().ends_with(words));
        }
        // The caret is still before the same text.
        let at = edit.selection.start;
        prop_assert!(out.is_char_boundary(at));
        if caret <= edit.range.start {
            prop_assert_eq!(at, caret);
        } else if caret > edit.range.end {
            prop_assert_eq!(&out[at..], &text[caret..]);
        } else {
            // In the white space the note ended with: before the transcript.
            prop_assert_eq!(at, edit.range.start);
        }
        Ok(())
    }

    fn text() -> impl Strategy<Value = String> {
        proptest::string::string_regex("([a-z ]{0,8}|\\[r\\]\\(inkdrop://file:f[0-3]\\)|!\\[\\]\\(inkdrop://file:f[0-3]\\))(\n{1,2}([a-z ]{0,8}|\\[r\\]\\(inkdrop://file:f[0-3]\\))){0,5}\n{0,2}")
            .unwrap()
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(
                proptest::test_runner::FileFailurePersistence::WithSource("regressions"),
            )),
            ..ProptestConfig::default()
        })]

        /// Nothing written is lost or moved: the note's text is all still
        /// there around the transcript, the transcript sits right under the
        /// recording's line when the note links it (else at the end), and
        /// the caret stays on the same text.
        #[test]
        fn prop_the_transcript_never_disturbs_the_note(
            text in text(),
            file in 0usize..4,
            words in "[A-Z][a-z]{0,6}( [a-z]{1,6}){0,4}\\.",
            caret in 0usize..80,
        ) {
            check_placement(&text, file, &words, caret)?;
        }
    }
}
