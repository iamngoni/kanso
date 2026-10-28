//! Voice input at the facade: what a transcript turns into at the caret, with
//! the caret in the unit Flutter counts in, and the setting.

mod common;

use common::*;
use kanso_mobile::{VoiceSettings, dictated};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    /// Whatever was heard and wherever the caret is, the caret is inside the
    /// text in UTF-16 units and on a character boundary (it never lands
    /// between the halves of a surrogate pair); with commands off the text is
    /// the trimmed transcript (after the space that joins it to what is there, and the case of its first letter, which follows the sentence).
    #[test]
    fn prop_the_caret_is_inside_the_text_on_a_character_boundary(
        before in "\\PC{0,40}",
        after in proptest::option::of("\\PC{0,10}"),
        transcript in "[a-zA-Z0-9 .,!?😀é日本]{0,60}|\\PC{0,60}",
        commands in any::<bool>(),
    ) {
        let said = dictated(before.clone(), after.clone(), transcript.clone(), commands);
        if !commands {
            let words = transcript.trim();
            let text = said.as_ref().map(|said| said.text.trim().to_lowercase());
            prop_assert_eq!(text, (!words.is_empty()).then(|| words.to_lowercase()));
        }
        if let Some(said) = said {
            let units: Vec<u16> = said.text.encode_utf16().collect();
            let caret = said.caret_utf16 as usize;
            prop_assert!(caret <= units.len());
            // Not between the halves of a surrogate pair.
            if caret > 0 && caret < units.len() {
                prop_assert!(!(0xD800..0xDC00).contains(&units[caret - 1]));
            }
        }
    }
}

// no-prop-test: the setting's default and its round trip.
#[test]
fn spoken_commands_are_on_until_turned_off_and_the_setting_is_kept() {
    let phone = lone_phone();
    assert_eq!(
        phone.library.voice_settings().unwrap(),
        VoiceSettings {
            spoken_commands: true
        }
    );
    phone
        .library
        .set_voice_settings(VoiceSettings {
            spoken_commands: false,
        })
        .unwrap();
    assert!(!phone.library.voice_settings().unwrap().spoken_commands);
    assert!(phone.dir.path().join("voice.json").exists());
}

// no-prop-test: fixed dictations, one with a character of two UTF-16 units.
#[test]
fn a_dictation_is_spoken_punctuation_and_a_utf16_caret() {
    let said = dictated(String::new(), None, "Buy milk. Comma. Eggs.".into(), true).unwrap();
    assert_eq!(said.text, "Buy milk, eggs.");
    assert_eq!(said.caret_utf16 as usize, said.text.encode_utf16().count());
    let said = dictated(String::new(), None, "  hello 😀  ".into(), false).unwrap();
    assert_eq!(said.text, "hello 😀");
    assert_eq!(said.caret_utf16, 8, "the emoji counts two units");
    assert_eq!(dictated(String::new(), None, "   ".into(), false), None);
}
