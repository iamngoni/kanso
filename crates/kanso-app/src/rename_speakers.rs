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
use kanso_engine::recording_speakers::{self, SpeakerName};

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

/// Opens the sheet for the note shown in `editor`, or says why it can't.
// no-prop-test: glue over `recording_speakers`, which has the properties.
pub fn open(editor: Entity<NoteEditor>, window: &mut Window, cx: &mut App) {
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
    show(editor, file_id, names, window, cx);
}

fn show(
    editor: Entity<NoteEditor>,
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
        finished(apply(&editor, &file_id, &asked, window, cx))
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
    file_id: &str,
    changes: &[(String, String)],
    window: &mut Window,
    cx: &mut App,
) -> Result<(), String> {
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
    show(editor, "file:snapshot".into(), names, window, cx)
}

#[cfg(test)]
mod tests {
    use super::{field_label, nobody_words};
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
    fn nobody_to_rename_says_what_to_do() {
        assert!(nobody_words(true).contains("Tell speakers apart"));
        assert!(nobody_words(false).contains("Record"));
    }
}
