//! Keeping a recording's audio private (issue #38, step 11): deleting the
//! audio once the transcript is safely in the note ("Keep transcript
//! only"), and the rules that keep audio out of what leaves the device.
//! Sweeping what a crash leaves behind is `kanso_audio::sweep`; sealing audio
//! into a locked note is `recording_seal`.
//!
//! Deleting is irreversible, so it is refused unless the note, read back
//! from the store, holds the recording's transcript: a transcript that
//! failed, was cancelled, or is still only in an editor's buffer leaves the
//! audio where it is.

use kanso_core::links::LINK_SCHEME;
use kanso_store::{FileDrop, Origin, Store};
use kanso_sync::lock::LockKey;

use crate::recording_flow::FlowError;
use crate::recording_note::{AUDIO_SCHEME, transcript_range};
pub use crate::recording_privacy_model::*;
pub use crate::recording_seal::edit_text;
use crate::recording_seal::open_target;

/// The line that stands where the audio was.
pub const DELETED_LINE: &str = "Audio deleted after transcription.";

/// What the delete sheet says about backups: the daily ones made before now
/// still hold the audio (`docs/PLAN.md:56`).
pub const BACKUP_WARNING: &str = "Daily backups made before now still hold the audio.";

/// Whether an attachment of `content_type` is audio, and so stays out of a
/// published notebook unless the owner opts in. Pure.
pub fn is_audio_type(content_type: &str) -> bool {
    content_type
        .get(..6)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("audio/"))
}

/// Whether an attachment may go into a published notebook: anything but
/// audio, and audio only when the owner opted in. Pure.
pub fn may_publish(content_type: &str, include_audio: bool) -> bool {
    include_audio || !is_audio_type(content_type)
}

/// Whether `line` is a recording's audio line (`[Audio · 52 min](…)` or
/// `[System audio · …](…)`) linking any of `ids`. Pure.
fn is_audio_line(line: &str, ids: &[String]) -> bool {
    let line = line.trim_start();
    (line.starts_with("[Audio") || line.starts_with("[System audio"))
        && ids
            .iter()
            .any(|id| line.contains(&format!("]({LINK_SCHEME}{id})")))
}

/// `[00:12](kanso-audio://file:x?t=12)` links to any of `ids` become the
/// bare `00:12`: the moment can no longer be played. Pure.
fn unlink_moments(text: &str, ids: &[String]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find("](") {
        let url_start = at + 2;
        let tail = &rest[url_start..];
        let ours = ids.iter().any(|id| {
            tail.starts_with(&format!("{AUDIO_SCHEME}{id}?"))
                || tail.starts_with(&format!("{AUDIO_SCHEME}{id})"))
        });
        let label_start = rest[..at]
            .rfind('[')
            .filter(|open| !rest[*open..at].contains('\n'));
        let close = tail.find(')');
        match (ours, label_start, close) {
            (true, Some(open), Some(close)) => {
                out.push_str(&rest[..open]);
                out.push_str(&rest[open + 1..at]);
                rest = &rest[url_start + close + 1..];
            }
            _ => {
                out.push_str(&rest[..url_start]);
                rest = tail;
            }
        }
    }
    out.push_str(rest);
    out
}

/// `body` once the audio `ids` are gone: their audio lines become one line
/// saying so, and their timestamps become plain text. Everything else, the
/// transcript's words above all, is kept as it is. Pure.
pub fn without_audio(body: &str, ids: &[String]) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut said = false;
    for line in body.split('\n') {
        if !is_audio_line(line, ids) {
            lines.push(line.to_owned());
        } else if !said {
            said = true;
            lines.push(DELETED_LINE.to_owned());
        } else if lines.last().is_some_and(String::is_empty) {
            // A second audio line (the system track) joins the first's gap.
            lines.pop();
        }
    }
    unlink_moments(&lines.join("\n"), ids)
}

/// Whether `body` holds the transcript of the recording whose first file is
/// `file_id`: its marked block, with at least one paragraph. Pure.
pub fn transcript_saved(body: &str, file_id: &str) -> bool {
    transcript_range(body, file_id).is_some()
}

/// Deletes the audio files `audio` of a recording once the note `note_id`
/// holds its transcript (the one linking `transcript_file`), as stored: a
/// locked note is opened with `key`. The note is rewritten first (its audio
/// line says the audio was deleted, and its ownership of the files is
/// released), then the files go, so a failure part way leaves audio that a
/// second call deletes, never a transcript lost.
///
/// Nothing is deleted unless the stored transcript is there, and a file
/// another live note still links (a copy of the note, a duplicate, a merged
/// note, a canvas card) is kept: only this note lets go of it.
pub fn delete_audio_after_transcript(
    store: &Store,
    note_id: &str,
    transcript_file: &str,
    audio: &[String],
    key: Option<&LockKey>,
    origin: Origin,
) -> Result<Dropped, DropError> {
    let target = open_target(store, note_id, key).map_err(|err| match err {
        FlowError::Missing => DropError::Missing,
        FlowError::Store(err) => DropError::Store(err.to_string()),
        _ => DropError::Locked,
    })?;
    if !transcript_saved(target.body(), transcript_file) {
        return Err(DropError::NoTranscript);
    }
    let body = without_audio(target.body(), audio);
    target
        .rewrite(store, &body, audio, origin)
        .map_err(|err| DropError::Store(err.to_string()))?;
    let mut dropped = Dropped::default();
    for id in audio {
        // Another live note may link the same file (a copy, a duplicate, a
        // merged note, a canvas card): then it stays.
        match store.delete_file_if_unowned(id, origin) {
            Ok(FileDrop::Deleted) => dropped.files += 1,
            Ok(FileDrop::Owned) => dropped.kept += 1,
            Ok(FileDrop::Gone) => {}
            Err(err) => return Err(DropError::Store(err.to_string())),
        }
    }
    Ok(dropped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recording_flow::{Destination, Recorded, save_recording};
    use crate::recording_note::{Segment, render_transcript};
    use kanso_core::DocType;
    use kanso_store::Put;
    use proptest::prelude::*;
    use serde_json::json;

    fn store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        (dir, store)
    }

    fn saved(
        dir: &std::path::Path,
        store: &Store,
        session: &str,
    ) -> crate::recording_flow::SavedRecording {
        let part = dir.join(format!("{session}.opus"));
        std::fs::write(&part, b"OggS fake audio").unwrap();
        let recorded = Recorded {
            session_id: session.to_owned(),
            parts: vec![part],
            duration_secs: 90,
            mic_only: true,
            when: "23 Oct 2026, 14:05".into(),
        };
        let dest = Destination::NewNote { book_id: None };
        save_recording(store, &recorded, &dest, Origin::Ui).unwrap()
    }

    fn transcript(file: &str) -> String {
        render_transcript(
            file,
            &[
                Segment {
                    start_secs: 0,
                    text: "We started with the roadmap.".into(),
                    speaker: None,
                },
                Segment {
                    start_secs: 22,
                    text: "The beta is on track.".into(),
                    speaker: None,
                },
            ],
        )
        .unwrap()
    }

    fn body_of(store: &Store, id: &str) -> String {
        store.note(id).unwrap().unwrap().body
    }

    #[test]
    fn the_audio_goes_after_the_transcript_and_the_words_stay() {
        let (dir, store) = store();
        let saved = saved(dir.path(), &store, "a-1");
        let file = saved.first_file().to_owned();
        // No transcript yet: nothing is deleted.
        let err = delete_audio_after_transcript(
            &store,
            &saved.note_id,
            &file,
            &saved.file_ids,
            None,
            Origin::Ui,
        );
        assert!(matches!(err, Err(DropError::NoTranscript)));
        assert!(store.get(&file).unwrap().is_some());
        // The transcript is saved in the note.
        let note = store.note(&saved.note_id).unwrap().unwrap();
        let body = format!("{}\n\n{}\n", note.body.trim_end(), transcript(&file));
        store
            .put(
                DocType::Note,
                json!({"_id": note.id, "_rev": note.rev, "body": body}),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap();
        let done = delete_audio_after_transcript(
            &store,
            &saved.note_id,
            &file,
            &saved.file_ids,
            None,
            Origin::Ui,
        )
        .unwrap();
        assert_eq!(done.files, 1);
        assert!(store.get(&file).unwrap().is_none());
        let after = body_of(&store, &saved.note_id);
        assert!(after.contains(DELETED_LINE));
        assert!(after.contains("We started with the roadmap."));
        assert!(after.contains("00:22 The beta is on track."));
        assert!(!after.contains(&file));
        // A second call has nothing left to delete and changes nothing.
        let again = delete_audio_after_transcript(
            &store,
            &saved.note_id,
            &file,
            &saved.file_ids,
            None,
            Origin::Ui,
        );
        assert!(matches!(again, Err(DropError::NoTranscript)));
    }

    #[test]
    fn a_locked_note_needs_its_key_before_its_audio_goes() {
        let (dir, store) = store();
        let saved = saved(dir.path(), &store, "b-2");
        let file = saved.first_file().to_owned();
        let mut note = store.note(&saved.note_id).unwrap().unwrap();
        note.body = format!("{}\n\n{}\n", note.body.trim_end(), transcript(&file));
        let key = LockKey::create(
            &kanso_sync::lock::LockPassword::new("correct horse").unwrap(),
            kanso_sync::KdfParams::TEST,
        )
        .unwrap();
        let body = std::mem::take(&mut note.body);
        kanso_sync::lock::Envelope::store(Some(&key.seal(&note.id, &body)), &mut note.extra);
        kanso_core::model::store_note_files(Some(&[]), &mut note.extra);
        // The audio the note links is still plain here; this case is the
        // key check, so the file stays plain and unowned.
        store
            .lock_note(serde_json::to_value(&note).unwrap())
            .unwrap();
        let err = delete_audio_after_transcript(
            &store,
            &saved.note_id,
            &file,
            &saved.file_ids,
            None,
            Origin::Ui,
        );
        assert!(matches!(err, Err(DropError::Locked)));
        assert!(store.get(&file).unwrap().is_some());
        let done = delete_audio_after_transcript(
            &store,
            &saved.note_id,
            &file,
            &saved.file_ids,
            Some(&key),
            Origin::Ui,
        )
        .unwrap();
        assert_eq!(done.files, 1);
        let reread = store.note(&saved.note_id).unwrap().unwrap();
        let envelope = kanso_sync::lock::Envelope::from_extra(&reread.extra)
            .unwrap()
            .unwrap();
        let opened = key.open(&reread.id, &envelope).unwrap();
        assert!(opened.contains(DELETED_LINE) && opened.contains("The beta is on track."));
        assert!(reread.body.is_empty());
    }

    /// The note holding `saved`'s recording, with its transcript saved.
    fn with_transcript(store: &Store, saved: &crate::recording_flow::SavedRecording) -> String {
        let file = saved.first_file().to_owned();
        let note = store.note(&saved.note_id).unwrap().unwrap();
        let body = format!("{}\n\n{}\n", note.body.trim_end(), transcript(&file));
        store
            .put(
                DocType::Note,
                json!({"_id": note.id, "_rev": note.rev, "body": body}),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap();
        body
    }

    /// A note linking the same audio as `body` does, in the trash or in
    /// the Inbox.
    fn copy_of(store: &Store, id: &str, body: &str, trashed: bool) {
        let book = if trashed {
            "trash".to_owned()
        } else {
            store
                .role_book(kanso_core::roles::Role::Inbox, Origin::Ui)
                .unwrap()
        };
        store
            .put(
                DocType::Note,
                json!({"_id": id, "title": "Copy", "body": body, "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
    }

    #[test]
    fn a_copy_of_the_note_keeps_the_audio_it_links() {
        let (dir, store) = store();
        let saved = saved(dir.path(), &store, "e-5");
        let file = saved.first_file().to_owned();
        let body = with_transcript(&store, &saved);
        // A copy of the note (Duplicate, a merge, a share copy) links the
        // same file.
        copy_of(&store, "note:copy", &body, false);
        let done = delete_audio_after_transcript(
            &store,
            &saved.note_id,
            &file,
            &saved.file_ids,
            None,
            Origin::Ui,
        )
        .unwrap();
        assert_eq!(done, Dropped { files: 0, kept: 1 });
        assert!(store.get(&file).unwrap().is_some());
        // This note lets go of it; the copy still plays it.
        assert!(body_of(&store, &saved.note_id).contains(DELETED_LINE));
        assert_eq!(body_of(&store, "note:copy"), body);
    }

    /// A single published note (and a web page export) inlines images only,
    /// so its recording's audio never goes with it.
    #[test]
    fn a_published_note_page_never_carries_its_audio() {
        use base64::Engine as _;
        let (dir, store) = store();
        let saved = saved(dir.path(), &store, "g-7");
        let body = with_transcript(&store, &saved);
        let page = crate::export::page_without_queries(
            &store,
            "Standup",
            &body,
            &kanso_render::Palette::LIGHT,
            &crate::export::PageStyle::default(),
        );
        let audio = base64::engine::general_purpose::STANDARD.encode(b"OggS fake audio");
        assert!(page.contains("The beta is on track."));
        assert!(!page.contains(&audio) && !page.contains("data:audio"));
    }

    #[test]
    fn audio_is_left_out_unless_the_owner_opts_in() {
        assert!(is_audio_type("audio/ogg") && is_audio_type("Audio/MPEG"));
        assert!(!is_audio_type("image/png") && !is_audio_type("audio") && !is_audio_type(""));
        assert!(!may_publish("audio/ogg", false));
        assert!(may_publish("audio/ogg", true));
        assert!(may_publish("image/png", false));
    }

    #[test]
    fn two_audio_lines_become_one_and_spacing_holds() {
        let ids = vec!["file:a".to_owned(), "file:b".to_owned()];
        let body = "Intro\n\n[Audio · 1 min](inkdrop://file:a)\n\n[System audio · 1 min](inkdrop://file:b)\n\nOutro\n";
        assert_eq!(
            without_audio(body, &ids),
            format!("Intro\n\n{DELETED_LINE}\n\nOutro\n")
        );
    }

    fn words() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9 .,!?']{0,40}"
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 48, rng_seed: proptest::test_runner::RngSeed::Fixed(20261017), ..ProptestConfig::default() })]

        /// Deleting never happens before the transcript is in the stored
        /// note: whatever else the note holds (another recording's
        /// transcript, markers with no paragraphs, prose), the audio and
        /// the note are untouched.
        #[test]
        fn prop_audio_is_never_deleted_before_its_transcript_is_saved(
            prose in words(),
            decoy in prop_oneof![
                Just(String::new()),
                Just("<!-- kanso:transcript -->\n<!-- /kanso:transcript -->".to_owned()),
                Just("<!-- kanso:transcript -->\nplain words\n<!-- /kanso:transcript -->".to_owned()),
                Just(transcript("file:somebody-else")),
            ],
        ) {
            let (dir, store) = store();
            let saved = saved(dir.path(), &store, "c-3");
            let file = saved.first_file().to_owned();
            let note = store.note(&saved.note_id).unwrap().unwrap();
            let body = format!("{}\n\n{prose}\n\n{decoy}\n", note.body.trim_end());
            store
                .put(DocType::Note, json!({"_id": note.id, "_rev": note.rev, "body": body}), Put::Merge, Origin::Ui)
                .unwrap();
            let before = body_of(&store, &saved.note_id);
            let result = delete_audio_after_transcript(
                &store, &saved.note_id, &file, &saved.file_ids, None, Origin::Ui,
            );
            prop_assert!(matches!(result, Err(DropError::NoTranscript)));
            prop_assert!(store.get(&file).unwrap().is_some());
            prop_assert_eq!(body_of(&store, &saved.note_id), before);
        }

        /// With the transcript saved, every word of it survives, and the
        /// prose around it, whatever it says.
        #[test]
        fn prop_the_transcript_and_prose_survive_the_delete(
            prose in "[a-zA-Z0-9 .,]{1,30}",
            spoken in "[a-z]{1,10}( [a-z]{1,10}){0,5}",
        ) {
            let (dir, store) = store();
            let saved = saved(dir.path(), &store, "d-4");
            let file = saved.first_file().to_owned();
            let block = render_transcript(
                &file,
                &[Segment { start_secs: 5, text: spoken.clone(), speaker: None }],
            ).unwrap();
            let note = store.note(&saved.note_id).unwrap().unwrap();
            let body = format!("{}\n\n{prose}\n\n{block}\n", note.body.trim_end());
            store
                .put(DocType::Note, json!({"_id": note.id, "_rev": note.rev, "body": body}), Put::Merge, Origin::Ui)
                .unwrap();
            delete_audio_after_transcript(
                &store, &saved.note_id, &file, &saved.file_ids, None, Origin::Ui,
            ).unwrap();
            let after = body_of(&store, &saved.note_id);
            prop_assert!(after.contains(&spoken));
            prop_assert!(after.contains(prose.trim()));
            prop_assert!(!after.contains(&file));
            prop_assert!(store.get(&file).unwrap().is_none());
        }

        /// The audio is deleted exactly when no other live note links it,
        /// however many copies there are and wherever they sit (a notebook
        /// or the trash).
        #[test]
        fn prop_audio_another_note_links_is_never_deleted(
            copies in proptest::collection::vec(proptest::bool::ANY, 0..3),
        ) {
            let (dir, store) = store();
            let saved = saved(dir.path(), &store, "f-6");
            let file = saved.first_file().to_owned();
            let body = with_transcript(&store, &saved);
            for (ix, trashed) in copies.iter().enumerate() {
                copy_of(&store, &format!("note:copy-{ix}"), &body, *trashed);
            }
            let done = delete_audio_after_transcript(
                &store, &saved.note_id, &file, &saved.file_ids, None, Origin::Ui,
            ).unwrap();
            prop_assert_eq!(store.get(&file).unwrap().is_some(), !copies.is_empty());
            prop_assert_eq!(done.files, usize::from(copies.is_empty()));
            prop_assert!(body_of(&store, &saved.note_id).contains("The beta is on track."));
        }

        /// Audio never goes into a published notebook unless the owner
        /// opted in, for any type spelling.
        #[test]
        fn prop_publish_never_includes_audio_without_opt_in(
            subtype in "[a-z0-9.+-]{0,12}",
            upper in proptest::bool::ANY,
            other in "(image|text|application|video)/[a-z]{1,8}",
        ) {
            let audio = if upper { format!("AUDIO/{subtype}") } else { format!("audio/{subtype}") };
            prop_assert!(!may_publish(&audio, false));
            prop_assert!(may_publish(&audio, true));
            prop_assert!(may_publish(&other, false));
        }
    }
}
