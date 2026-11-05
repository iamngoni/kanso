//! The words and rules of the library key's screens (issue #243 step 11): the
//! save sheet, Show key…, and Restore from backup…. What each says, when the
//! save sheet comes back, and how a backup that didn't open is explained.
//! The sheets are in `library_key_sheet` and `restore_sheet`.

use kanso_store::{LibraryFile, LibraryKey, StoreError, WordsError};

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
/// encrypted, the person hasn't said they saved the key, and sync doesn't
/// hold the notes (signing in on a new computer brings them back, so they
/// need no key). `synced` is [`sync_holds_notes`].
pub fn offer_save_sheet(encrypted: bool, saved: bool, synced: bool) -> bool {
    encrypted && !saved && !synced
}

/// Whether sync can bring the notes back on another computer without the
/// key: the library is signed in, and every note has reached the server at
/// least once. The key itself never syncs; a library signed in whose notes
/// haven't gone up yet (`never_sent` of them) still needs the words.
pub fn sync_holds_notes(signed_in: bool, never_sent: usize) -> bool {
    signed_in && never_sent == 0
}

/// Whether the key the person said they saved (`saved`, its fingerprint)
/// is the library's key now. A library with no key has nothing to save.
pub fn key_saved(saved: Option<&str>, key: Option<&LibraryKey>) -> bool {
    match (saved, key) {
        (Some(saved), Some(key)) => saved == key.fingerprint(),
        _ => false,
    }
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
        // The checksum lets about 1 in 256 typos through, so this may be one.
        StoreError::WrongKey if typed_words => {
            "These words don't open this backup. Check each word and their order; if they're all right, the backup was made with another library's key.".to_owned()
        }
        StoreError::WrongKey => {
            "This backup is encrypted with another key. Enter the 24 words of the library it came from.".to_owned()
        }
        StoreError::Invalid(why) => why.clone(),
        StoreError::DiskFull { needed_mb } => format!(
            "There isn't enough free space to restore this backup. Kanso needs about {needed_mb} MB free on the disk that holds your library."
        ),
        other => format!("Couldn't read that backup: {other}"),
    }
}

/// What typing the 24 words got wrong, for the line under the field.
pub fn words_problem(err: WordsError) -> String {
    err.to_string()
}

/// Why a restore isn't staged while the library is signed in to sync (issue
/// #243): sync would bring back the notes the account holds over the ones
/// restored, partly undoing it.
pub const RESTORE_SIGNED_IN: &str = "This library is signed in to sync, so it can't be restored now. Sign out first (Settings › Account), then restore the backup, then sign in again. Signing in again merges the restored library with your account: a note deleted in the backup but still in your account comes back, and a note that differs keeps both versions.";

/// Whether a restore must wait for the person to sign out: the library is
/// signed in to sync, or a sign-in is waiting on a password or key
/// (`locked`).
pub fn restore_refusal(signed_in: bool, locked: bool) -> Option<&'static str> {
    (signed_in || locked).then_some(RESTORE_SIGNED_IN)
}

/// What a finished restore says, once it is staged.
pub const STAGED: &str = "The backup is ready. Quit Kanso and open it again to finish. The library you have now is kept as a backup.";

/// What the next launch says when it applied the restore.
pub const APPLIED: &str =
    "Restored from your backup. The library you had before is kept in your backups.";

/// What the next launch says when it couldn't. `again`: the restore still
/// waits, and the next launch tries it again.
pub fn not_applied(why: &str, again: bool) -> String {
    if again {
        format!(
            "Couldn't restore the backup yet: {why}. Your library is as it was; Kanso tries again the next time it opens."
        )
    } else {
        format!("Couldn't restore the backup: {why}. Your library is as it was.")
    }
}

/// What the Restore from backup row says while a restore waits.
pub const WAITING: &str = "A backup replaces this library the next time Kanso opens. The library you have now is kept as a backup.";

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

    proptest! {
        #![proptest_config(ProptestConfig {
            rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f),
            failure_persistence: None,
            ..ProptestConfig::default()
        })]

        /// Invariants: a key counts as saved only when the mark is that
        /// key's; another key's mark, no mark, or no key is not saved.
        #[test]
        fn prop_saved_means_this_key_was_saved(
            ours in any::<[u8; kanso_store::KEY_BYTES]>(),
            other in any::<[u8; kanso_store::KEY_BYTES]>(),
        ) {
            let ours = LibraryKey::from_bytes(ours);
            let other = LibraryKey::from_bytes(other);
            prop_assert!(key_saved(Some(&ours.fingerprint()), Some(&ours)));
            prop_assert_eq!(
                key_saved(Some(&other.fingerprint()), Some(&ours)),
                other == ours
            );
            prop_assert!(!key_saved(None, Some(&ours)));
            prop_assert!(!key_saved(Some(&ours.fingerprint()), None));
        }

        /// Invariants: a restore is refused exactly when signed in or locked.
        #[test]
        fn prop_restore_refused_iff_signed_in_or_locked(signed_in in any::<bool>(), locked in any::<bool>()) {
            prop_assert_eq!(
                restore_refusal(signed_in, locked).is_some(),
                signed_in || locked
            );
        }

        /// Invariants: sync holds the notes only when signed in with
        /// nothing that never reached the server.
        #[test]
        fn prop_sync_holds_the_notes_once_they_went_up(
            signed_in in any::<bool>(),
            never_sent in 0usize..5,
        ) {
            prop_assert_eq!(
                sync_holds_notes(signed_in, never_sent),
                signed_in && never_sent == 0
            );
        }
    }

    /// Regression: "I've saved it" was a plain yes in config.json, so a
    /// library made again with a new key (the old one deleted, the settings
    /// kept) never asked for the new key's words.
    #[test]
    fn a_new_key_is_not_the_saved_one() {
        let old = LibraryKey::from_bytes([1; kanso_store::KEY_BYTES]);
        let new = LibraryKey::from_bytes([2; kanso_store::KEY_BYTES]);
        assert!(key_saved(Some(&old.fingerprint()), Some(&old)));
        assert!(!key_saved(Some(&old.fingerprint()), Some(&new)));
        assert!(offer_save_sheet(
            true,
            key_saved(Some(&old.fingerprint()), Some(&new)),
            false
        ));
    }

    /// Regression: signed in to sync used to skip the sheet even while
    /// notes had never gone up, so neither sync nor the key could bring
    /// them back.
    #[test]
    fn signed_in_with_notes_not_sent_still_asks() {
        assert!(offer_save_sheet(true, false, sync_holds_notes(true, 3)));
        assert!(!offer_save_sheet(true, false, sync_holds_notes(true, 0)));
    }

    #[test]
    fn a_full_disk_says_restore_not_encrypt() {
        let why = restore_problem(&StoreError::DiskFull { needed_mb: 12 }, false);
        assert!(why.contains("restore") && why.contains("12 MB"), "{why}");
        assert!(not_applied("x", true).contains("tries again"));
        assert!(!not_applied("x", false).contains("tries again"));
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
