//! The words and rules of the library key's screens (issue #243 step 11): the
//! save sheet, Show key…, and Restore from backup…. What each says, when the
//! save sheet comes back, and how a backup that didn't open is explained.
//! The sheets are in `library_key_sheet` and `restore_sheet`.

use kanso_store::{LibraryFile, StoreError, WordsError};

/// The save sheet's title, right after encryption and at each launch until
/// the key is saved.
pub const SAVE_TITLE: &str = "Save your library key";

/// The title when the person asked to see the key.
pub const SHOW_TITLE: &str = "Your library key";

pub const SAVE_BODY: &str = "Your library and its backups are now encrypted. On another computer, a backup opens only with this key. Write these 24 words down and keep them somewhere safe.";

pub const SHOW_BODY: &str = "Your library and its backups are encrypted with this key. On another computer, a backup opens only with these 24 words. Write them down and keep them somewhere safe, away from this computer.";

/// What the toast says after Copy.
pub const COPIED: &str = "Library key copied. It leaves the clipboard in a minute.";

/// What Show key… says when it can't confirm the person.
pub const NOT_CONFIRMED: &str = "Couldn't confirm it's you, so the key stays hidden.";

/// What the ask for confirmation says when there is no Touch ID or Windows
/// Hello to use.
pub const CONFIRM_BODY: &str = "The key opens every backup of your library. Make sure no one can see your screen, then show it.";

/// Whether the save sheet comes up at this launch: the library is
/// encrypted, the person hasn't said they saved the key, and the library
/// isn't signed in to sync (signing in on a new computer brings the notes
/// back, so it needs no key).
pub fn offer_save_sheet(encrypted: bool, saved: bool, signed_in: bool) -> bool {
    encrypted && !saved && !signed_in
}

/// What the Library key row says under its name.
pub fn key_row_detail(saved: bool) -> &'static str {
    if saved {
        "Your library and backups are encrypted. The key opens them on another computer; Show key\u{2026} writes it as 24 words."
    } else {
        "Library key not saved. Show it and write down the 24 words, or a backup can't be opened on another computer."
    }
}

/// The words of a key as a numbered list, `(1, "word")`.
pub fn numbered(words: &str) -> Vec<(usize, &str)> {
    words
        .split(' ')
        .filter(|word| !word.is_empty())
        .enumerate()
        .map(|(ix, word)| (ix + 1, word))
        .collect()
}

/// Whether a backup needs the 24 words before it can be restored: it is
/// encrypted and this computer's key doesn't open it.
pub fn needs_words(file: LibraryFile, opens_with_this_key: bool) -> bool {
    file == LibraryFile::Encrypted && !opens_with_this_key
}

/// Why a backup couldn't be restored, in words a person can act on.
/// `typed_words` is whether the 24 words were used (so a wrong key means
/// they belong to another library, not that none were given).
pub fn restore_problem(err: &StoreError, typed_words: bool) -> String {
    match err {
        StoreError::WrongKey if typed_words => {
            "These words are a real library key, but they don't open this backup. They may belong to another library.".to_owned()
        }
        StoreError::WrongKey => {
            "This backup is encrypted with another key. Enter the 24 words of the library it came from.".to_owned()
        }
        StoreError::Invalid(why) => why.clone(),
        other => format!("Couldn't read that backup: {other}"),
    }
}

/// What typing the 24 words got wrong, for the line under the field.
pub fn words_problem(err: WordsError) -> String {
    err.to_string()
}

/// What a finished restore says, once it is staged.
pub const STAGED: &str = "The backup is ready. Quit Kanso and open it again to finish. The library you have now is kept as a backup.";

/// What the next launch says when it applied the restore.
pub const APPLIED: &str =
    "Restored from your backup. The library you had before is kept in your backups.";

/// What the next launch says when it couldn't.
pub fn not_applied(why: &str) -> String {
    format!("Couldn't restore the backup: {why}. Your library is as it was.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Invariants: the save sheet is for an encrypted library, not yet
        /// saved, not signed in, and for nothing else.
        #[test]
        fn prop_the_save_sheet_comes_back_until_saved(
            encrypted in any::<bool>(),
            saved in any::<bool>(),
            signed_in in any::<bool>(),
        ) {
            prop_assert_eq!(
                offer_save_sheet(encrypted, saved, signed_in),
                encrypted && !saved && !signed_in
            );
        }

        /// Invariants: a list of words is numbered from 1 in order, and
        /// nothing is lost or invented.
        #[test]
        fn prop_numbering_keeps_every_word_in_order(
            words in proptest::collection::vec("[a-z]{3,8}", 0..30),
        ) {
            let text = words.join(" ");
            let listed = numbered(&text);
            prop_assert_eq!(listed.len(), words.len());
            for (ix, (n, word)) in listed.iter().enumerate() {
                prop_assert_eq!(*n, ix + 1);
                prop_assert_eq!(*word, words[ix].as_str());
            }
        }

        /// Invariants: only an encrypted backup that this key doesn't open
        /// asks for words.
        #[test]
        fn prop_only_a_locked_backup_asks_for_words(
            kind in prop_oneof![
                Just(LibraryFile::New),
                Just(LibraryFile::Plain),
                Just(LibraryFile::Encrypted)
            ],
            opens in any::<bool>(),
        ) {
            prop_assert_eq!(
                needs_words(kind, opens),
                kind == LibraryFile::Encrypted && !opens
            );
        }
    }

    #[test]
    fn a_wrong_key_says_whether_words_were_tried() {
        let asked = restore_problem(&StoreError::WrongKey, false);
        let tried = restore_problem(&StoreError::WrongKey, true);
        assert!(asked.contains("24 words"));
        assert!(tried.contains("another library"));
        assert_ne!(asked, tried);
        assert_eq!(
            restore_problem(&StoreError::Invalid("That file is empty.".into()), false),
            "That file is empty."
        );
    }

    #[test]
    fn the_unsaved_row_names_the_state_the_settings_page_shows() {
        assert!(key_row_detail(false).starts_with("Library key not saved"));
        assert!(!key_row_detail(true).contains("not saved"));
    }
}
