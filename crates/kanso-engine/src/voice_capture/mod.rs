//! Voice capture (issue #193): what a finished recording becomes in the
//! library. The desktop's Quick Capture and the phones' voice notes, the
//! Action Button, the widgets and the Android tile all end in
//! [`save_voice_capture`].
//!
//! A capture always lands in the Inbox, never asks for a notebook or a
//! title, and keeps the transcript. It is saved once: the note's ID is made
//! from the capture's own, so a retry after a crash finds the note already
//! there. Nothing is written into a locked notebook.

mod model;
mod transcript;

use kanso_core::DocType;
use kanso_core::capture_mark::capture_value;
use kanso_core::model::{MAX_ATTACHMENT_SIZE, MAX_BODY_LEN};
use kanso_core::roles::Role;
use kanso_store::{Origin, Put, Store, StoreError};
use serde_json::json;

use crate::attach::file_markdown;
use crate::capture::{CaptureError, is_drop_id};

pub use model::{Engine, MEETING_AFTER_MS, SavedCapture, Segment, TidyState, VoiceCapture};
pub use transcript::{
    PAUSE_MS, clean_transcript, duration_label, transcript_from_segments, voice_title,
};

/// What a meeting is: a recording longer than [`MEETING_AFTER_MS`] when
/// `treat_long_as_meeting` (Settings › Voice) is on.
pub fn is_meeting(duration_ms: u64, treat_long_as_meeting: bool) -> bool {
    treat_long_as_meeting && duration_ms > MEETING_AFTER_MS
}

/// The ID of the audio file kept for capture `id` (`file:cap-<id>-audio`).
pub fn audio_file_id(id: &str) -> String {
    format!("file:cap-{id}-audio")
}

/// The ID of the note capture `id` makes.
pub fn note_id(id: &str) -> String {
    format!("note:cap-{id}")
}

/// The content type of an audio file a phone or the recorder writes, by
/// extension.
fn audio_type(extension: &str) -> Option<&'static str> {
    Some(match extension.to_ascii_lowercase().as_str() {
        "m4a" | "mp4" | "aac" => "audio/mp4",
        "caf" => "audio/x-caf",
        "wav" => "audio/wav",
        "mp3" => "audio/mpeg",
        "ogg" | "opus" => "audio/ogg",
        "3gp" => "audio/3gpp",
        _ => return None,
    })
}

/// Attaches the recording as `file:cap-<id>-audio` and returns the link to
/// it, or what to tell the person. A file already stored under that ID is
/// kept as it is.
fn attach_audio(store: &Store, id: &str, path: &std::path::Path) -> Result<String, String> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default();
    let content_type =
        audio_type(extension).ok_or("The recording isn't in a format Kanso keeps.")?;
    let size = std::fs::metadata(path)
        .map_err(|e| format!("Couldn't read the recording: {e}"))?
        .len();
    if size > MAX_ATTACHMENT_SIZE as u64 {
        return Err(
            "The recording is over 10 MB, so the audio wasn't kept. The transcript was.".to_owned(),
        );
    }
    let file_id = audio_file_id(id);
    let name = format!("Voice note.{}", extension.to_ascii_lowercase());
    if store.get(&file_id).map_err(|e| e.to_string())?.is_none() {
        let bytes = std::fs::read(path).map_err(|e| format!("Couldn't read the recording: {e}"))?;
        use base64::Engine as _;
        let doc = json!({
            "_id": file_id,
            "name": name,
            "contentType": content_type,
            "publicIn": [],
            "_attachments": {"index": {
                "content_type": content_type,
                "data": base64::engine::general_purpose::STANDARD.encode(&bytes),
            }},
        });
        store
            .put(DocType::File, doc, Put::Upsert, Origin::Ui)
            .map_err(|e| format!("Couldn't attach the recording: {e}"))?;
    }
    Ok(file_markdown(&file_id, &name))
}

/// Saves `cap` as a note in the Inbox (made if needed) and returns it.
///
/// The body is the cleaned transcript ([`clean_transcript`], with
/// paragraph breaks at pauses when the engine gave times) and the title its
/// first sentence. The note records `kansoCapture` (`tidy` is "pending"
/// when `tidy_pending`, else "none"). With `keep_audio` and a file, the
/// recording is attached as well; one over the attachment limit is left
/// out and named in [`SavedCapture::problems`].
///
/// `Ok(None)` when nothing was heard. A locked Inbox refuses
/// ([`CaptureError::Locked`]) before anything is written, and an ID that
/// isn't 32 lowercase hex characters is [`CaptureError::BadId`]. A capture
/// already saved is returned as it is.
pub fn save_voice_capture(
    store: &Store,
    cap: &VoiceCapture,
    keep_audio: bool,
    tidy_pending: bool,
    origin: Origin,
) -> Result<Option<SavedCapture>, CaptureError> {
    if !is_drop_id(&cap.id) {
        return Err(CaptureError::BadId);
    }
    let note = note_id(&cap.id);
    if store.get(&note)?.is_some() {
        return Ok(Some(SavedCapture {
            id: note,
            problems: Vec::new(),
            existed: true,
        }));
    }
    let text = if cap.segments.is_empty() {
        clean_transcript(&cap.transcript)
    } else {
        transcript_from_segments(&cap.segments)
    };
    if text.is_empty() {
        return Ok(None);
    }
    let inbox = store.role_book(Role::Inbox, origin)?;
    if store.locked_books()?.contains(&inbox) {
        return Err(CaptureError::Locked);
    }

    let mut problems = Vec::new();
    let mut body = text.clone();
    if keep_audio && let Some(path) = &cap.audio_file {
        match attach_audio(store, &cap.id, path) {
            Ok(link) => {
                body.push_str("\n\n");
                body.push_str(&link);
            }
            Err(problem) => problems.push(problem),
        }
    }
    if body.chars().count() > MAX_BODY_LEN {
        body = body.chars().take(MAX_BODY_LEN).collect();
        problems.push("The transcript was too long for a note and was cut.".to_owned());
    }
    let state = if tidy_pending {
        TidyState::Pending
    } else {
        TidyState::None
    };
    let engine = cap.engine;
    let put = store.put(
        DocType::Note,
        json!({
            "_id": note,
            "title": voice_title(&text),
            "body": body,
            "bookId": inbox,
            "kansoCapture": capture_value(cap.duration_ms, engine, state),
        }),
        Put::Upsert,
        origin,
    );
    match put {
        Ok(written) => Ok(Some(SavedCapture {
            id: written.id,
            problems,
            existed: false,
        })),
        // Two saves of the same capture raced: the other one won.
        Err(StoreError::Conflict) if store.get(&note)?.is_some() => Ok(Some(SavedCapture {
            id: note,
            problems: Vec::new(),
            existed: true,
        })),
        Err(err) => Err(err.into()),
    }
}

/// Saves text that was spoken into a window, as `save_capture` saves typed
/// text, with the capture's record added. For the desktop's Quick Capture,
/// where the person may have edited the words or tidied them before saving.
///
/// `id` is 32 lowercase hex characters (the note is `note:cap-<id>`, saved
/// once). The note goes in `book`, or the Inbox when that's `None` or gone.
/// A tidied text starts with its `# Title` line, which names the note;
/// untidied words are titled by their first sentence and kept whole.
/// `Ok(None)` for empty text. A locked notebook refuses.
#[allow(clippy::too_many_arguments)]
pub fn save_voice_text(
    store: &Store,
    id: &str,
    text: &str,
    book: Option<&str>,
    duration_ms: u64,
    engine: Engine,
    tidy: TidyState,
    origin: Origin,
) -> Result<Option<SavedCapture>, CaptureError> {
    if !is_drop_id(id) {
        return Err(CaptureError::BadId);
    }
    let note = note_id(id);
    if store.get(&note)?.is_some() {
        return Ok(Some(SavedCapture {
            id: note,
            problems: Vec::new(),
            existed: true,
        }));
    }
    let text = text.trim();
    if text.is_empty() {
        return Ok(None);
    }
    let (title, body) = if tidy == TidyState::Done {
        crate::capture::split_capture(text).expect("text is not empty")
    } else {
        (voice_title(text), text.to_owned())
    };
    let book = match book {
        Some(book) if store.get(book)?.is_some() => book.to_owned(),
        _ => store.role_book(Role::Inbox, origin)?,
    };
    if store.locked_books()?.contains(&book) {
        return Err(CaptureError::Locked);
    }
    let written = store.put(
        DocType::Note,
        json!({
            "_id": note,
            "title": title,
            "body": body,
            "bookId": book,
            "kansoCapture": capture_value(duration_ms, engine, tidy),
        }),
        Put::Upsert,
        origin,
    )?;
    Ok(Some(SavedCapture {
        id: written.id,
        problems: Vec::new(),
        existed: false,
    }))
}

/// A fresh capture ID: 32 lowercase hex characters.
pub fn new_capture_id() -> String {
    (0..32)
        .map(|_| char::from_digit(fastrand::u32(0..16), 16).expect("a hex digit"))
        .collect()
}

#[cfg(test)]
mod tests;
