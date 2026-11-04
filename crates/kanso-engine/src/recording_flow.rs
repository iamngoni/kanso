//! A recording becoming a note (issue #38, step 4): the line that links the
//! audio, the clock on the recording bar, the words of the crash banner,
//! and saving a finished recording into the library. The desktop's Record
//! sheet and the phones' recorder both end in [`save_recording`].
//!
//! A recording is saved under IDs made from its session, so saving again
//! after a failure or a crash finds what is already stored and adds
//! nothing twice. Nothing is written into a locked note or notebook: the
//! audio blob is not sealed by the lock, so a locked note refuses audio
//! until that exists (the issue's answer to open question 10).

use std::path::Path;

use base64::Engine as _;
use kanso_core::DocType;
use kanso_core::links::LINK_SCHEME;
use kanso_core::model::MAX_ATTACHMENT_SIZE;
use kanso_core::roles::Role;
use kanso_store::{Origin, Put, Store};
use serde_json::json;

pub use crate::recording_flow_model::*;
use crate::recording_note;

/// What every recording part is stored as.
pub const AUDIO_TYPE: &str = "audio/ogg";

/// `00:42:17`: the recording bar's clock, audio time. Pure.
pub fn clock(seconds: u64) -> String {
    format!(
        "{:02}:{:02}:{:02}",
        seconds / 3600,
        seconds / 60 % 60,
        seconds % 60
    )
}

/// `52 min`, `40 sec`, `2 h 5 min`: how long, in the audio line. Pure.
pub fn length_label(seconds: u64) -> String {
    if seconds < 60 {
        return format!("{seconds} sec");
    }
    let minutes = seconds / 60;
    if minutes < 120 {
        format!("{minutes} min")
    } else if minutes.is_multiple_of(60) {
        format!("{} h", minutes / 60)
    } else {
        format!("{} h {} min", minutes / 60, minutes % 60)
    }
}

/// The recording's first line: `[Audio · 52 min](inkdrop://file:abc)`, or
/// for several parts `[Audio · part 1](…) · [part 2](…)`. The player takes
/// every link on this line as one recording. Pure.
pub fn audio_line(file_ids: &[String], duration_secs: u64) -> String {
    match file_ids {
        [] => String::new(),
        [only] => format!(
            "[Audio · {}]({LINK_SCHEME}{only})",
            length_label(duration_secs)
        ),
        [first, rest @ ..] => {
            let mut line = format!("[Audio · part 1]({LINK_SCHEME}{first})");
            for (ix, id) in rest.iter().enumerate() {
                line.push_str(&format!(" · [part {}]({LINK_SCHEME}{id})", ix + 2));
            }
            line
        }
    }
}

/// `body` with `line` as a paragraph at its end, keeping what is there.
/// Pure.
pub fn append_to_body(body: &str, line: &str) -> String {
    let kept = body.trim_end();
    if kept.is_empty() {
        format!("{line}\n")
    } else {
        format!("{kept}\n\n{line}\n")
    }
}

/// The banner shown after a crash: "Kanso closed while recording at 14:05
/// (52 min)." `at` is the local start time, such as `14:05`. Pure.
pub fn recovery_text(at: Option<&str>, seconds: u64) -> String {
    match at {
        Some(at) => format!(
            "Kanso closed while recording at {at} ({}).",
            length_label(seconds)
        ),
        None => format!("Kanso closed while recording ({}).", length_label(seconds)),
    }
}

/// The local date and time a note's title carries: `23 Oct 2026, 14:05`.
pub fn when_text(ms: i64) -> String {
    use chrono::TimeZone as _;
    chrono::Local
        .timestamp_millis_opt(ms)
        .single()
        .map_or_else(String::new, |d| d.format("%-d %b %Y, %H:%M").to_string())
}

/// The local time of day the banner names: `14:05`.
pub fn time_of_day(ms: i64) -> Option<String> {
    use chrono::TimeZone as _;
    chrono::Local
        .timestamp_millis_opt(ms)
        .single()
        .map(|d| d.format("%H:%M").to_string())
}

/// The ID of part `index` (from 0) of session `session_id`.
pub fn part_file_id(session_id: &str, index: usize) -> String {
    format!("file:rec-{session_id}-{}", index + 1)
}

/// The ID of the note a recording from `session_id` makes.
pub fn note_id(session_id: &str) -> String {
    format!("note:rec-{session_id}")
}

fn attach_part(
    store: &Store,
    session_id: &str,
    index: usize,
    path: &Path,
    origin: Origin,
) -> Result<String, FlowError> {
    let id = part_file_id(session_id, index);
    if store.get(&id)?.is_some() {
        return Ok(id);
    }
    let size = std::fs::metadata(path)
        .map_err(|e| FlowError::Read(e.to_string()))?
        .len();
    if size > MAX_ATTACHMENT_SIZE as u64 {
        return Err(FlowError::TooLarge);
    }
    let bytes = std::fs::read(path).map_err(|e| FlowError::Read(e.to_string()))?;
    let name = format!("Recording part {}.opus", index + 1);
    let doc = json!({
        "_id": id,
        "name": name,
        "contentType": AUDIO_TYPE,
        "publicIn": [],
        "_attachments": {"index": {
            "content_type": AUDIO_TYPE,
            "data": base64::engine::general_purpose::STANDARD.encode(&bytes),
        }},
    });
    store.put(DocType::File, doc, Put::Upsert, origin)?;
    Ok(id)
}

/// Attaches the parts of `recorded` and puts them in the note: a new one
/// titled `Recording — …` (`Meeting — …` with system audio) holding the
/// audio line, or the line at the end of an existing one. Refuses a locked
/// note or notebook before storing anything.
// no-prop-test: the line, clock and labels have the properties below; the
// store calls are exercised by the example tests.
pub fn save_recording(
    store: &Store,
    recorded: &Recorded,
    destination: &Destination,
    origin: Origin,
) -> Result<SavedRecording, FlowError> {
    if recorded.parts.is_empty() {
        return Err(FlowError::Empty);
    }
    let locked_books = store.locked_books()?;
    let (existing, book) = match destination {
        Destination::ThisNote { note_id } => {
            let note = store.note(note_id)?.ok_or(FlowError::Missing)?;
            if note.is_locked() || locked_books.contains(&note.book_id) {
                return Err(FlowError::Locked);
            }
            (Some(note), None)
        }
        Destination::NewNote { book_id } => {
            let book = match book_id {
                Some(book) => book.clone(),
                None => store.role_book(Role::Inbox, origin)?,
            };
            if locked_books.contains(&book) {
                return Err(FlowError::Locked);
            }
            (None, Some(book))
        }
    };
    let mut file_ids = Vec::new();
    for (index, path) in recorded.parts.iter().enumerate() {
        file_ids.push(attach_part(
            store,
            &recorded.session_id,
            index,
            path,
            origin,
        )?);
    }
    let line = audio_line(&file_ids, recorded.duration_secs);
    let (note_id, created) = match (existing, book) {
        (Some(note), _) => {
            // Saving twice must not link the recording twice.
            if !note.body.contains(&file_ids[0]) {
                store.put(
                    DocType::Note,
                    json!({
                        "_id": note.id,
                        "_rev": note.rev,
                        "body": append_to_body(&note.body, &line),
                    }),
                    Put::Merge,
                    origin,
                )?;
            }
            (note.id, false)
        }
        (None, Some(book)) => {
            let id = note_id(&recorded.session_id);
            let made = store.get(&id)?.is_none();
            if made {
                store.put(
                    DocType::Note,
                    json!({
                        "_id": id,
                        "title": recording_note::title(recorded.mic_only, &recorded.when),
                        "body": format!("{line}\n"),
                        "bookId": book,
                    }),
                    Put::Upsert,
                    origin,
                )?;
            }
            (id, made)
        }
        (None, None) => return Err(FlowError::Empty),
    };
    Ok(SavedRecording {
        note_id,
        file_ids,
        created,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn ids(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("file:a{i}")).collect()
    }

    #[test]
    fn the_clock_runs_in_hours() {
        assert_eq!(clock(0), "00:00:00");
        assert_eq!(clock(42 * 60 + 17), "00:42:17");
        assert_eq!(clock(3723), "01:02:03");
    }

    #[test]
    fn lengths_read_naturally() {
        assert_eq!(length_label(40), "40 sec");
        assert_eq!(length_label(52 * 60 + 10), "52 min");
        assert_eq!(length_label(2 * 3600), "2 h");
        assert_eq!(length_label(2 * 3600 + 5 * 60), "2 h 5 min");
    }

    #[test]
    fn the_audio_line_matches_the_design() {
        assert_eq!(
            audio_line(&ids(1), 52 * 60),
            "[Audio · 52 min](inkdrop://file:a0)"
        );
        assert_eq!(
            audio_line(&ids(2), 3600),
            "[Audio · part 1](inkdrop://file:a0) · [part 2](inkdrop://file:a1)"
        );
        assert_eq!(audio_line(&[], 5), "");
    }

    #[test]
    fn a_line_goes_after_what_is_there() {
        assert_eq!(append_to_body("", "L"), "L\n");
        assert_eq!(append_to_body("Hello\n\n\n", "L"), "Hello\n\nL\n");
    }

    #[test]
    fn the_banner_names_the_time_and_length() {
        assert_eq!(
            recovery_text(Some("14:05"), 52 * 60),
            "Kanso closed while recording at 14:05 (52 min)."
        );
        assert_eq!(
            recovery_text(None, 30),
            "Kanso closed while recording (30 sec)."
        );
    }

    #[test]
    fn the_defaults_follow_the_source() {
        assert_eq!(
            TranscribeWhen::default_for(WhatToRecord::Microphone),
            TranscribeWhen::WhileRecording
        );
        assert_eq!(
            TranscribeWhen::default_for(WhatToRecord::MicrophoneAndSystem),
            TranscribeWhen::AfterStopping
        );
        let plan = RecordPlan::new();
        assert_eq!(plan.destination, Destination::NewNote { book_id: None });
    }

    fn store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        (dir, store)
    }

    fn recorded(dir: &Path, parts: usize) -> Recorded {
        let files = (0..parts)
            .map(|i| {
                let path = dir.join(format!("part-{i}.opus"));
                std::fs::write(&path, [i as u8; 64]).unwrap();
                path
            })
            .collect();
        Recorded {
            session_id: "abc-1".into(),
            parts: files,
            duration_secs: 3120,
            mic_only: false,
            when: "23 Oct 2026, 14:05".into(),
        }
    }

    #[test]
    fn a_new_note_holds_the_audio_line_and_saving_twice_adds_nothing() {
        let (dir, store) = store();
        let rec = recorded(dir.path(), 2);
        let dest = Destination::NewNote { book_id: None };
        let saved = save_recording(&store, &rec, &dest, Origin::Ui).unwrap();
        assert!(saved.created);
        assert_eq!(saved.file_ids, vec!["file:rec-abc-1-1", "file:rec-abc-1-2"]);
        let note = store.note(&saved.note_id).unwrap().unwrap();
        assert_eq!(note.title, "Meeting — 23 Oct 2026, 14:05");
        assert_eq!(
            note.body,
            "[Audio · part 1](inkdrop://file:rec-abc-1-1) · [part 2](inkdrop://file:rec-abc-1-2)\n"
        );
        let again = save_recording(&store, &rec, &dest, Origin::Ui).unwrap();
        assert!(!again.created);
        assert_eq!(again.note_id, saved.note_id);
        assert_eq!(store.note(&saved.note_id).unwrap().unwrap().body, note.body);
    }

    #[test]
    fn this_note_gets_the_line_after_its_text() {
        let (dir, store) = store();
        let book = store.role_book(Role::Inbox, Origin::Ui).unwrap();
        let made = store
            .put(
                DocType::Note,
                json!({"title": "Memo", "body": "My prose", "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        let rec = recorded(dir.path(), 1);
        let dest = Destination::ThisNote {
            note_id: made.id.clone(),
        };
        let saved = save_recording(&store, &rec, &dest, Origin::Ui).unwrap();
        assert!(!saved.created);
        let body = store.note(&made.id).unwrap().unwrap().body;
        assert_eq!(
            body,
            "My prose\n\n[Audio · 52 min](inkdrop://file:rec-abc-1-1)\n"
        );
        save_recording(&store, &rec, &dest, Origin::Ui).unwrap();
        assert_eq!(store.note(&made.id).unwrap().unwrap().body, body);
    }

    #[test]
    fn a_missing_note_or_no_audio_is_refused() {
        let (dir, store) = store();
        let rec = recorded(dir.path(), 1);
        let gone = Destination::ThisNote {
            note_id: "note:nope".into(),
        };
        assert!(matches!(
            save_recording(&store, &rec, &gone, Origin::Ui),
            Err(FlowError::Missing)
        ));
        let none = Recorded {
            parts: vec![],
            ..rec
        };
        let dest = Destination::NewNote { book_id: None };
        assert!(matches!(
            save_recording(&store, &none, &dest, Origin::Ui),
            Err(FlowError::Empty)
        ));
    }

    proptest! {

        #[test]
        fn prop_the_clock_is_hh_mm_ss_and_round_trips(secs in 0u64..400_000) {
            let text = clock(secs);
            let parts: Vec<u64> = text.split(':').map(|p| p.parse().unwrap()).collect();
            prop_assert_eq!(parts.len(), 3);
            prop_assert!(parts[1] < 60 && parts[2] < 60);
            prop_assert_eq!(parts[0] * 3600 + parts[1] * 60 + parts[2], secs);
        }

        #[test]
        fn prop_every_part_is_linked_once_on_the_first_line(n in 1usize..40, secs in 0u64..100_000) {
            let ids = ids(n);
            let line = audio_line(&ids, secs);
            prop_assert!(!line.contains('\n'));
            for id in &ids {
                prop_assert_eq!(line.matches(&format!("({LINK_SCHEME}{id})")).count(), 1);
            }
        }

        #[test]
        fn prop_appending_keeps_the_body(body in "\\PC{0,200}", n in 1usize..4) {
            let line = audio_line(&ids(n), 60);
            let out = append_to_body(&body, &line);
            prop_assert!(out.starts_with(body.trim_end()));
            prop_assert!(out.trim_end().ends_with(&line));
        }
    }
}
