//! Rename Speakers (issue #38, step 13): the names before the paragraphs of
//! a recording's transcript ("Speaker 1", "You", "Meeting"), changed in one
//! sheet. Giving two the same name merges them.
//!
//! The names are text in the note, so this rewrites that text, inside the
//! transcript's markers only, as one step in the open editor: Undo brings
//! the old names back. The rules are `kanso_engine::recording_speakers`'s.

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;
use kanso_engine::recording_note::transcript_range;
use kanso_engine::recording_speakers::{self, SpeakerName, SpeakersError};

use crate::dialogs::{self, Field, FormSpec, Progress, Submit};
use crate::editor::NoteEditor;

/// What the sheet says above the fields.
pub const DESCRIPTION: &str = "Type a name for each voice. Two voices given the same name become one. The names are text in the note, so Undo brings the old ones back.";

/// The label of the field for `speaker`: the name, and how much it says.
/// Pure.
pub fn field_label(speaker: &SpeakerName) -> String {
    let n = speaker.paragraphs;
    format!(
        "{} · {n} paragraph{}",
        speaker.name,
        if n == 1 { "" } else { "s" }
    )
}

/// What to say when the note has nobody to rename. Pure.
pub fn nobody_words(has_transcript: bool) -> &'static str {
    if has_transcript {
        "This transcript doesn't say who spoke. Turn on Tell speakers apart in Settings › Editor and transcribe the recording again, or start a paragraph with a name and a colon."
    } else {
        "Put a recording with a transcript in this note first: Record…, or Transcribe Recording on a recording you dropped in."
    }
}

/// Why the note `id` shown in the editor can't have its speakers renamed,
/// if it can't: it is in the trash or tagged `#ai-exclude`, or gone. A
/// locked note is the editor's to refuse: while its lock is open the words
/// are in the editor (never read as plaintext from disk), and while it is
/// closed the edit is refused there. Fails closed.
pub fn refused(store: &kanso_store::Store, id: &str) -> Option<String> {
    match recording_speakers::refusal(store, id) {
        None | Some(SpeakersError::Locked) => None,
        Some(why) => Some(why.to_string()),
    }
}

/// Opens the sheet for the note shown in `editor`, or says why it can't.
// no-prop-test: glue over `recording_speakers`, which has the properties.
pub fn open(editor: Entity<NoteEditor>, window: &mut Window, cx: &mut App) {
    let Some(note_id) = editor.read(cx).note_id().map(str::to_owned) else {
        return;
    };
    let store = cx.global::<crate::workspace::AppState>().store.clone();
    if let Some(why) = refused(&store, &note_id) {
        window.push_notification(Notification::info(why), cx);
        return;
    }
    let body = editor.read(cx).body_text(cx).to_string();
    let found = recording_speakers::first_recording(&body);
    let names = found
        .as_deref()
        .map(|id| recording_speakers::speakers(&body, id))
        .unwrap_or_default();
    let (Some(file_id), false) = (found, names.is_empty()) else {
        let words = nobody_words(body.contains(kanso_engine::recording_note::TRANSCRIPT_OPEN));
        window.push_notification(Notification::info(words), cx);
        return;
    };
    show(editor, note_id, file_id, names, window, cx);
}

fn show(
    editor: Entity<NoteEditor>,
    note_id: String,
    file_id: String,
    names: Vec<SpeakerName>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<dialogs::FormView> {
    let mut spec = FormSpec::new("Rename Speakers", "Rename").description(DESCRIPTION);
    for speaker in &names {
        spec = spec.field(Field::text(field_label(speaker)).initial(speaker.name.clone()));
    }
    dialogs::form(spec, window, cx, move |typed, window, cx| {
        let asked = recording_speakers::changes(&names, &typed);
        if asked.is_empty() {
            return finished(Ok(()));
        }
        finished(apply(&editor, &note_id, &file_id, &asked, window, cx))
    })
}

/// A submit that is already over.
fn finished(result: Result<(), String>) -> Submit {
    let (tx, rx) = async_channel::bounded(1);
    tx.try_send(Progress::Done(result)).ok();
    Submit::Running(rx)
}

/// Writes the new names into the open note as one step.
fn apply(
    editor: &Entity<NoteEditor>,
    note_id: &str,
    file_id: &str,
    changes: &[(String, String)],
    window: &mut Window,
    cx: &mut App,
) -> Result<(), String> {
    // The sheet's note, still: not another one opened meanwhile, and not
    // one trashed or tagged #ai-exclude since.
    if editor.read(cx).note_id() != Some(note_id) {
        return Err("Another note is open now. Open Rename Speakers again on this one.".into());
    }
    let store = cx.global::<crate::workspace::AppState>().store.clone();
    if let Some(why) = refused(&store, note_id) {
        return Err(why);
    }
    let body = editor.read(cx).body_text(cx).to_string();
    let renamed = recording_speakers::rename_many(&body, file_id, changes)
        .ok_or("The names in the note changed since this sheet opened. Open it again.")?;
    let range = transcript_range(&renamed, file_id)
        .ok_or("The recording's transcript is no longer in the note.")?;
    let block = renamed[range].to_owned();
    let put = editor.update(cx, |editor, cx| {
        editor.insert_transcript_below(Some(file_id), &block, window, cx)
    });
    if put {
        Ok(())
    } else {
        Err("The note is locked. Unlock it to rename the speakers.".to_owned())
    }
}

/// Opens the sheet over fixed names, for the snapshots.
// no-prop-test: drawing and glue.
#[cfg(test)]
pub fn open_for_snapshot(
    editor: Entity<NoteEditor>,
    names: Vec<SpeakerName>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<dialogs::FormView> {
    let note_id = editor.read(cx).note_id().unwrap_or_default().to_owned();
    show(editor, note_id, "file:snapshot".into(), names, window, cx)
}

#[cfg(test)]
mod tests {
    use super::{field_label, nobody_words, refused};
    use kanso_engine::recording_speakers::SpeakerName;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 48,
            rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f),
            failure_persistence: Some(Box::new(proptest::test_runner::FileFailurePersistence::Off)),
            ..ProptestConfig::default()
        })]

        /// A field's label always names the speaker and counts paragraphs.
        #[test]
        fn prop_labels_name_the_speaker_and_count(name in "[A-Za-z ]{1,20}", n in 1u32..500) {
            let label = field_label(&SpeakerName { name: name.clone(), paragraphs: n });
            prop_assert!(label.starts_with(&name));
            prop_assert!(label.contains(&n.to_string()));
            prop_assert_eq!(label.ends_with("paragraph"), n == 1);
        }
    }

    #[test]
    fn trashed_and_ai_exclude_notes_are_refused_and_a_locked_one_is_the_editors() {
        use kanso_core::DocType;
        use kanso_store::{Origin, Put, Store};
        use serde_json::json;
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        let put = |fields: serde_json::Value| {
            store
                .put(DocType::Note, fields, Put::Upsert, Origin::Ui)
                .unwrap()
                .id
        };
        let book = store
            .put(
                DocType::Book,
                json!({"name": "Work"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let plain = put(json!({"title": "A", "body": "x", "bookId": book}));
        let trashed =
            put(json!({"title": "B", "body": "x", "bookId": kanso_core::model::TRASH_BOOK_ID}));
        let tag = store
            .put(
                DocType::Tag,
                json!({"name": "ai-exclude"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let excluded = put(json!({"title": "C", "body": "x", "bookId": book, "tags": [tag]}));
        let locked = put(json!({"title": "D", "body": "x", "bookId": book}));
        store
            .put(
                DocType::Note,
                json!({"_id": locked, "kansoLock": {"salt": "abc"}}),
                Put::Merge,
                Origin::Sync,
            )
            .unwrap();
        assert_eq!(refused(&store, &plain), None);
        assert_eq!(refused(&store, &locked), None);
        assert!(refused(&store, &trashed).is_some_and(|w| w.contains("trash")));
        assert!(refused(&store, &excluded).is_some_and(|w| w.contains("#ai-exclude")));
        assert!(refused(&store, "note:gone").is_some());
    }

    #[test]
    fn nobody_to_rename_says_what_to_do() {
        assert!(nobody_words(true).contains("Tell speakers apart"));
        assert!(nobody_words(false).contains("Record"));
    }
}
