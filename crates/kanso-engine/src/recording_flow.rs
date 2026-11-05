//! A recording becoming a note (issue #38, step 4): the line that links the
//! audio, the clock on the recording bar, the words of the crash banner,
//! and saving a finished recording into the library. The desktop's Record
//! sheet and the phones' recorder both end in [`save_recording`].
//!
//! A recording is saved under IDs made from its session, so saving again
//! after a failure or a crash finds what is already stored and adds
//! nothing twice. A locked note takes a recording only sealed, with the
//! lock key in hand ([`crate::recording_seal`]; the issue's answer to open
//! question 10): without the key it refuses, and nothing is written into it.

use std::path::Path;

use base64::Engine as _;
use kanso_core::DocType;
use kanso_core::links::LINK_SCHEME;
use kanso_core::model::MAX_ATTACHMENT_SIZE;
use kanso_core::roles::Role;
use kanso_store::{Origin, Put, Store};
use kanso_sync::lock::LockKey;
use serde_json::json;

pub use crate::recording_flow_model::*;
use crate::recording_note;
use crate::recording_seal::{Target, open_target};

/// What a system audio track's session ID adds to its recording's.
pub const SYSTEM_SUFFIX: &str = "-sys";

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
    track_line("Audio", file_ids, duration_secs)
}

/// The line of a system audio track: `[System audio · 52 min](…)`. It is a
/// line of its own, so the player treats it as a second recording rather
/// than more of the microphone's. Pure.
pub fn system_line(file_ids: &[String], duration_secs: u64) -> String {
    track_line("System audio", file_ids, duration_secs)
}

fn track_line(label: &str, file_ids: &[String], duration_secs: u64) -> String {
    match file_ids {
        [] => String::new(),
        [only] => format!(
            "[{label} · {}]({LINK_SCHEME}{only})",
            length_label(duration_secs)
        ),
        [first, rest @ ..] => {
            let mut line = format!("[{label} · part 1]({LINK_SCHEME}{first})");
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

/// The session ID of the system audio track recorded beside session `id`.
pub fn system_session_id(id: &str) -> String {
    format!("{id}{SYSTEM_SUFFIX}")
}

/// The session `id` is the system track of, when it is one.
pub fn system_track_of(id: &str) -> Option<&str> {
    id.strip_suffix(SYSTEM_SUFFIX)
        .filter(|base| !base.is_empty())
}

/// The ID of the note a recording from `session_id` makes.
pub fn note_id(session_id: &str) -> String {
    format!("note:rec-{session_id}")
}

pub(crate) fn attach_part(
    store: &Store,
    session_id: &str,
    index: usize,
    path: &Path,
    label: &str,
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
    let name = format!("{label} part {}.opus", index + 1);
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

/// Refuses, before anything is recorded, a note the recording couldn't be
/// saved into: one that is gone, or locked (or in a locked notebook) with no
/// key to seal the audio under. A new note is checked when it is saved (its
/// notebook is made then).
pub fn check_destination(store: &Store, destination: &Destination) -> Result<(), FlowError> {
    check_destination_with(store, destination, None)
}

/// [`check_destination`], with the lock `key` the session holds: a locked
/// note that `key` opens is allowed, and takes the audio sealed.
pub fn check_destination_with(
    store: &Store,
    destination: &Destination,
    key: Option<&LockKey>,
) -> Result<(), FlowError> {
    match destination {
        Destination::ThisNote { note_id } => open_target(store, note_id, key).map(|_| ()),
        Destination::NewNote { .. } => Ok(()),
    }
}

/// [`save_recording`], but a recording whose note was deleted or locked
/// while it ran (or before a crash was recovered) goes into a new note
/// instead, so the audio is never kept out of the library for good.
// no-prop-test: glue over `save_recording`; the example test covers it.
pub fn save_recording_or_new(
    store: &Store,
    recorded: &Recorded,
    destination: &Destination,
    origin: Origin,
) -> Result<SavedRecording, FlowError> {
    save_recording_or_new_with(store, recorded, destination, None, origin)
}

/// [`save_recording_or_new`], with the lock `key` the session holds, which
/// lets a locked note take the audio sealed. A note that is locked and has no
/// key sends the audio to a new note, as one that is gone does.
// no-prop-test: glue over `save_recording_with`; the example test covers it.
pub fn save_recording_or_new_with(
    store: &Store,
    recorded: &Recorded,
    destination: &Destination,
    key: Option<&LockKey>,
    origin: Origin,
) -> Result<SavedRecording, FlowError> {
    match save_recording_with(store, recorded, destination, key, origin) {
        Err(FlowError::Locked | FlowError::Missing | FlowError::NotSealable)
            if matches!(destination, Destination::ThisNote { .. }) =>
        {
            let new = Destination::NewNote { book_id: None };
            save_recording_with(store, recorded, &new, None, origin)
        }
        other => other,
    }
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
    save_recording_with(store, recorded, destination, None, origin)
}

/// [`save_recording`], with the lock `key` the session holds: a locked note
/// that `key` opens takes the parts sealed under it (never as plain bytes),
/// and its text is re-sealed with the audio line in it.
// no-prop-test: see `save_recording`; sealing has its own example and
// property tests below.
pub fn save_recording_with(
    store: &Store,
    recorded: &Recorded,
    destination: &Destination,
    key: Option<&LockKey>,
    origin: Origin,
) -> Result<SavedRecording, FlowError> {
    if recorded.parts.is_empty() {
        return Err(FlowError::Empty);
    }
    let (existing, book) = match destination {
        Destination::ThisNote { note_id } => (Some(open_target(store, note_id, key)?), None),
        Destination::NewNote { book_id } => {
            let book = match book_id {
                Some(book) => book.clone(),
                None => store.role_book(Role::Inbox, origin)?,
            };
            if store.locked_books()?.contains(&book) {
                return Err(FlowError::Locked);
            }
            (None, Some(book))
        }
    };
    let mut existing = existing;
    let mut file_ids = Vec::new();
    for (index, path) in recorded.parts.iter().enumerate() {
        let id = match existing.as_mut() {
            Some(target) => target.attach(
                store,
                &recorded.session_id,
                index,
                path,
                "Recording",
                origin,
            )?,
            None => attach_part(
                store,
                &recorded.session_id,
                index,
                path,
                "Recording",
                origin,
            )?,
        };
        file_ids.push(id);
    }
    let line = audio_line(&file_ids, recorded.duration_secs);
    let (note_id, created) = match (existing, book) {
        (Some(target), _) => {
            let id = target.id().to_owned();
            // Saving twice must not link the recording twice.
            if !target.body().contains(&file_ids[0]) {
                target.append(store, &line, origin)?;
            }
            (id, false)
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

/// Attaches the system audio track of a recording and links it on a line of
/// its own at the end of the note `note_id`, which the microphone's (or, for
/// system audio alone, nothing else) saved. `session_id` is the track's own
/// session ID. Saving twice links it once. Refuses a locked note.
// no-prop-test: the line has the properties above; the store calls are
// exercised by the example tests.
pub fn save_system_track(
    store: &Store,
    note_id: &str,
    session_id: &str,
    parts: &[std::path::PathBuf],
    duration_secs: u64,
    origin: Origin,
) -> Result<SavedTrack, FlowError> {
    save_system_track_with(
        store,
        note_id,
        session_id,
        parts,
        duration_secs,
        None,
        origin,
    )
}

/// [`save_system_track`], with the lock `key` the session holds: a locked
/// note that `key` opens takes the track sealed.
// no-prop-test: see `save_system_track`.
#[allow(clippy::too_many_arguments)]
pub fn save_system_track_with(
    store: &Store,
    note_id: &str,
    session_id: &str,
    parts: &[std::path::PathBuf],
    duration_secs: u64,
    key: Option<&LockKey>,
    origin: Origin,
) -> Result<SavedTrack, FlowError> {
    if parts.is_empty() {
        return Err(FlowError::Empty);
    }
    let mut target: Target = open_target(store, note_id, key)?;
    let mut file_ids = Vec::new();
    for (index, path) in parts.iter().enumerate() {
        file_ids.push(target.attach(store, session_id, index, path, "System audio", origin)?);
    }
    if !target.body().contains(&file_ids[0]) {
        let line = system_line(&file_ids, duration_secs);
        target.append(store, &line, origin)?;
    }
    Ok(SavedTrack { file_ids })
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

    #[test]
    fn a_locked_note_is_refused_up_front_and_its_recording_lands_in_a_new_note() {
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
        let dest = Destination::ThisNote {
            note_id: made.id.clone(),
        };
        assert!(check_destination(&store, &dest).is_ok());
        let gone = Destination::ThisNote {
            note_id: "note:nope".into(),
        };
        assert!(matches!(
            check_destination(&store, &gone),
            Err(FlowError::Missing)
        ));
        // The note went away while recording: the audio still lands.
        let rec = recorded(dir.path(), 1);
        let saved = save_recording_or_new(&store, &rec, &gone, Origin::Ui).unwrap();
        assert!(saved.created);
        assert_eq!(saved.note_id, note_id("abc-1"));
        // Saving again (a second Recover) adds nothing.
        let again = save_recording_or_new(&store, &rec, &gone, Origin::Ui).unwrap();
        assert!(!again.created);
        assert_eq!(again.note_id, saved.note_id);
    }

    #[test]
    fn a_system_track_is_a_line_of_its_own_and_saving_twice_adds_nothing() {
        let (dir, store) = store();
        let rec = recorded(dir.path(), 1);
        let dest = Destination::NewNote { book_id: None };
        let saved = save_recording(&store, &rec, &dest, Origin::Ui).unwrap();
        let sys_id = system_session_id("abc-1");
        assert_eq!(system_track_of(&sys_id), Some("abc-1"));
        let parts: Vec<_> = (0..2)
            .map(|i| {
                let path = dir.path().join(format!("sys-{i}.opus"));
                std::fs::write(&path, [9u8; 32]).unwrap();
                path
            })
            .collect();
        let track =
            save_system_track(&store, &saved.note_id, &sys_id, &parts, 3120, Origin::Ui).unwrap();
        assert_eq!(
            track.file_ids,
            vec!["file:rec-abc-1-sys-1", "file:rec-abc-1-sys-2"]
        );
        let body = store.note(&saved.note_id).unwrap().unwrap().body;
        assert_eq!(
            body,
            "[Audio · 52 min](inkdrop://file:rec-abc-1-1)\n\n[System audio · part 1](inkdrop://file:rec-abc-1-sys-1) · [part 2](inkdrop://file:rec-abc-1-sys-2)\n"
        );
        // Each line is a recording of its own for the player.
        let lines = crate::recording_player::link_lines(&body);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[1].file_ids.len(), 2);
        save_system_track(&store, &saved.note_id, &sys_id, &parts, 3120, Origin::Ui).unwrap();
        assert_eq!(store.note(&saved.note_id).unwrap().unwrap().body, body);
        let file = store.get("file:rec-abc-1-sys-1").unwrap().unwrap();
        assert_eq!(file.fields["name"], "System audio part 1.opus");
    }

    #[test]
    fn a_plan_needs_an_app_before_it_starts() {
        let mut plan = RecordPlan::new();
        assert_eq!(plan.problem(), None);
        plan.what = WhatToRecord::MicrophoneAndSystem;
        assert!(plan.problem().is_some());
        plan.system = Some(SystemSource::App {
            bundle_id: "us.zoom.xos".into(),
            name: "zoom.us".into(),
        });
        assert_eq!(plan.problem(), None);
        assert_eq!(plan.bar_label(), "Microphone · zoom.us");
        plan.what = WhatToRecord::SystemOnly;
        assert_eq!(plan.bar_label(), "zoom.us");
        plan.system = Some(SystemSource::Everything);
        assert_eq!(plan.bar_label(), "All system audio");
    }

    proptest! {

        #[test]
        fn prop_a_system_session_id_names_its_recording(id in "[a-z0-9]{1,8}-[a-z0-9]{1,8}") {
            let sys = system_session_id(&id);
            prop_assert_eq!(system_track_of(&sys), Some(id.as_str()));
            prop_assert_eq!(system_track_of(&id), None);
            // Valid as a folder name and as part of a file ID the player reads.
            prop_assert!(sys.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'));
            let file = part_file_id(&sys, 0);
            let url = format!("kanso-audio://{file}?t=3");
            prop_assert!(crate::recording_note::parse_moment(&url).is_some());
        }

        #[test]
        fn prop_a_system_line_links_every_part_once_on_one_line(n in 1usize..40, secs in 0u64..100_000) {
            let ids = ids(n);
            let line = system_line(&ids, secs);
            prop_assert!(!line.contains('\n'));
            prop_assert!(line.starts_with("[System audio"));
            for id in &ids {
                prop_assert_eq!(line.matches(&format!("({LINK_SCHEME}{id})")).count(), 1);
            }
        }

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
