//! Seal Attachments of Locked Notes: the prompt at the first launch after
//! the update, and the menu item that does it later (#44, step 7).
//!
//! Notes locked before attachments were sealed still show their pictures and
//! files as plain files on disk. Nothing can seal those without the lock
//! password, so Kanso asks once: "Seal now" asks for the password and seals
//! every locked note's attachments in one pass, in the background with a
//! progress sheet; "Later" leaves it, with a quiet reminder at each launch
//! and the same action in the Note menu and the command palette. What does
//! the work, and its crash safety, is `kanso_engine::lock::files::migrate`.

use std::sync::Arc;

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::Button;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;
use kanso_engine::lock::files::migrate::{self, Outcome, Pending, Scan};
use kanso_sync::lock::LockKey;

use crate::dialogs::{self, Described, Field, FormSpec};
use crate::lock_flow::{self, size_words};
use crate::locked::{self, LockKeys};
use crate::zoom::px;

actions!(kanso, [SealLockedAttachments]);

/// The library's note of an answer to the prompt: `later`.
const ANSWER_META: &str = "locks.files_migration";

/// Whether this launch has asked or reminded already.
static OFFERED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The dialogs' title.
pub const TITLE: &str = "Seal attachments of locked notes";

fn notes(n: usize) -> String {
    format!("{n} locked note{}", if n == 1 { "" } else { "s" })
}

fn attachments(n: usize) -> String {
    format!("{n} attachment{}", if n == 1 { "" } else { "s" })
}

/// "1 attachment is" or "2 attachments are", to start a sentence about them.
fn attachments_are(n: usize) -> String {
    format!("{} {}", attachments(n), if n == 1 { "is" } else { "are" })
}

/// What the prompt says. With every waiting note open (the keys held fit
/// them all) it also counts their attachments and size, as the phone does;
/// else it says when that is known, rather than count part of them.
pub fn prompt_words(pending: &Pending, scan: Option<&Scan>) -> String {
    let mut words = format!(
        "Attachments of your locked notes aren't sealed yet. Enter your lock password to seal them now. {} still have pictures and files stored in plain form.",
        notes(pending.notes()),
    );
    match scan.filter(|scan| scan.notes > 0 && scan.unopened == 0) {
        Some(scan) => {
            words.push_str(&format!(
                " {} ({}) will be sealed.",
                attachments(scan.summary.files),
                size_words(scan.summary.bytes),
            ));
            if scan.summary.shared > 0 {
                words.push_str(&format!(
                    " {} also used by other notes: the locked note gets its own sealed copy, and the other notes keep theirs.",
                    attachments_are(scan.summary.shared),
                ));
            }
        }
        None => {
            words.push_str(" Kanso counts them, and their size, once the password opens the notes.")
        }
    }
    words
}

/// What the dialog says when an earlier pass stopped part way.
pub fn stopped_words(stopped: usize) -> String {
    format!(
        "Sealing the attachments of {} didn't finish. Some are sealed, and none is lost. Finish sealing, or undo it to keep everything as it was.",
        notes(stopped),
    )
}

/// The quiet reminder shown at each launch until it is done.
pub fn reminder_words(pending: &Pending) -> String {
    format!(
        "Attachments of {} aren't sealed yet.",
        notes(pending.notes())
    )
}

/// What Kanso says when a pass is over.
pub fn finished_words(outcome: &Outcome) -> String {
    let mut words = if outcome.stopped {
        format!(
            "Sealing stopped after {}. Run Seal Attachments of Locked Notes again to finish.",
            notes(outcome.notes),
        )
    } else if outcome.files() == 0 && outcome.shared == 0 {
        format!(
            "Checked {}: no attachments needed sealing.",
            notes(outcome.notes)
        )
    } else {
        format!(
            "Sealed {} in {}.",
            attachments(outcome.files() + outcome.shared),
            notes(outcome.notes),
        )
    };
    if outcome.copied > 0 {
        words.push_str(&format!(
            " {} also used by other notes, so each locked note has its own sealed {}.",
            attachments_are(outcome.copied),
            if outcome.copied == 1 {
                "copy"
            } else {
                "copies"
            },
        ));
    }
    if outcome.unopened > 0 {
        words.push_str(&format!(
            " {} use another lock password and were left as they are.",
            notes(outcome.unopened),
        ));
    }
    if let Some(first) = outcome.failed.first() {
        let many = outcome.failed.len() > 1;
        words.push_str(&format!(
            " {} couldn't be sealed and {}: {}",
            notes(outcome.failed.len()),
            if many {
                "stay as they were"
            } else {
                "stays as it was"
            },
            first.words,
        ));
    }
    words
}

/// What Kanso says when opening a note an older Kanso locked sealed its
/// pictures and files: once for that note, which has its list after. Nothing
/// when it linked none.
pub(crate) fn opened_words(sealed: &locked::SealedOnOpen) -> Option<String> {
    match sealed {
        locked::SealedOnOpen::Sealed(counts) if counts.sealed + counts.copied == 0 => None,
        locked::SealedOnOpen::Sealed(counts) => {
            let mut words = format!("Sealed {}.", attachments(counts.sealed + counts.copied));
            if counts.copied > 0 {
                words.push_str(&format!(
                    " {} also used by other notes, so this note has its own sealed {}.",
                    attachments_are(counts.copied),
                    if counts.copied == 1 { "copy" } else { "copies" },
                ));
            }
            Some(words)
        }
        locked::SealedOnOpen::Failed(words) => Some(format!(
            "This note's attachments couldn't all be sealed. {words} What wasn't sealed stays as it was; the note opens as usual."
        )),
    }
}

/// What Kanso says after an undo.
pub fn undone_words(files: usize) -> String {
    format!(
        "Undone. {} {} readable again, as before.",
        attachments(files),
        if files == 1 { "is" } else { "are" },
    )
}

// MARK: Starting

/// At launch: asks once, or reminds quietly after "Later". Nothing happens
/// when every locked note's attachments are sealed.
pub fn offer_at_launch(window: &mut Window, cx: &mut App) {
    // Once per launch, whatever windows open.
    if OFFERED.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    let store = locked::store(cx);
    let pending = migrate::pending(&store);
    if pending.is_done() {
        return;
    }
    if store.meta(ANSWER_META).ok().flatten().as_deref() == Some("later") {
        remind(&pending, window, cx);
    } else {
        ask(pending, true, window, cx);
    }
}

/// Seal Attachments of Locked Notes…, from the menu.
pub fn open(window: &mut Window, cx: &mut App) {
    let pending = migrate::pending(&locked::store(cx));
    if pending.is_done() {
        window.push_notification(
            Notification::info("The attachments of every locked note are sealed."),
            cx,
        );
        return;
    }
    ask(pending, false, window, cx);
}

fn remind(pending: &Pending, window: &mut Window, cx: &mut App) {
    let notification = Notification::info(reminder_words(pending)).action(|_, _, _| {
        Button::new("seal-attachments-now")
            .label("Seal now")
            .on_click(|_, window, cx| {
                window.dispatch_action(Box::new(SealLockedAttachments), cx);
            })
    });
    window.push_notification(notification, cx);
}

/// The prompt. `launch` is the first-launch one: "Later" is remembered, so
/// the next launches remind quietly instead of asking again.
fn ask(pending: Pending, launch: bool, window: &mut Window, cx: &mut App) {
    if pending.stopped > 0 {
        return ask_stopped(pending, window, cx);
    }
    let keys = LockKeys::all(cx);
    let scan = (!keys.is_empty()).then(|| migrate::scan(&locked::store(cx), &keys));
    open_prompt(prompt_words(&pending, scan.as_ref()), launch, window, cx);
}

/// The prompt's dialog: [`prompt_words`], "Seal now" and "Later".
pub(crate) fn open_prompt(body: String, launch: bool, window: &mut Window, cx: &mut App) {
    let body: SharedString = body.into();
    window.open_alert_dialog(cx, move |alert, _, _| {
        alert
            .title(TITLE)
            .width(px(dialogs::FORM_WIDTH))
            .description(body.clone())
            .show_cancel(true)
            .cancel_text("Later")
            .ok_text("Seal now")
            .on_ok(|_, window, cx| {
                // After this dialog closes: closing pops the top dialog,
                // which would otherwise be the password form opened here.
                window.defer(cx, proceed);
                true
            })
            .on_cancel(move |_, _, cx| {
                if launch {
                    locked::store(cx).set_meta(ANSWER_META, "later").ok();
                }
                true
            })
    });
}

/// A pass that stopped left its journal on the notes: finish it, or undo it.
fn ask_stopped(pending: Pending, window: &mut Window, cx: &mut App) {
    dialogs::choose(
        "Sealing didn't finish",
        stopped_words(pending.stopped),
        vec![
            Described {
                label: "Finish sealing".into(),
                description: "Seals the rest of the attachments. Nothing sealed is sealed again."
                    .into(),
            },
            Described {
                label: "Undo".into(),
                description: "Makes the attachments it sealed readable again, as they were.".into(),
            },
        ],
        window,
        cx,
        |choice, window, cx| match choice {
            Some(0) => proceed(window, cx),
            Some(_) => undo(window, cx),
            None => {}
        },
    );
}

/// "Seal now": with every note open already, straight to the work; else the
/// password first.
fn proceed(window: &mut Window, cx: &mut App) {
    let store = locked::store(cx);
    let keys = LockKeys::all(cx);
    let scan = migrate::scan(&store, &keys);
    if scan.notes > 0 && scan.unopened == 0 {
        start(window, cx);
    } else {
        password_form(window, cx);
    }
}

fn password_form(window: &mut Window, cx: &mut App) -> Entity<dialogs::FormView> {
    let spec = FormSpec::new(TITLE, "Seal")
        .description(
            "Type your lock password. Kanso seals the attachments of every locked note it opens.",
        )
        .field(Field::secret("Lock password"));
    dialogs::form(spec, window, cx, move |values, window, cx| {
        let password = match locked::password(&values[0]) {
            Ok(password) => password,
            Err(submit) => return submit,
        };
        let store = locked::store(cx);
        locked::run_in_background(
            window,
            cx,
            move || migrate::keys_for(&store, &password).map(secrets),
            move |keys, window, cx| {
                for secret in keys {
                    if let Ok(key) = LockKey::from_secret(&secret) {
                        LockKeys::add(key, cx);
                    }
                }
                start(window, cx);
                Ok(())
            },
        )
    })
}

/// The keys as their secrets, which cross to the main thread.
fn secrets(keys: Vec<LockKey>) -> Vec<String> {
    keys.iter().map(|key| key.to_secret().to_string()).collect()
}

/// Seals everything the keys held open, in the background with the sheet.
fn start(window: &mut Window, cx: &mut App) {
    let store = locked::store(cx);
    let keys: Vec<Arc<LockKey>> = LockKeys::all(cx);
    let scan = migrate::scan(&store, &keys);
    if scan.notes == 0 {
        window.push_notification(
            Notification::error("No locked note opened with that password."),
            cx,
        );
        return;
    }
    let book = LockKeys::file_keys(cx);
    lock_flow::run_with(
        true,
        "Sealing attachments",
        lock_flow::sheet_line(&scan.summary),
        scan.summary,
        window,
        cx,
        move |watch| Ok(migrate::migrate(&store, &book, &keys, watch)),
        finished,
    );
}

fn finished(result: Result<Outcome, String>, window: &mut Window, cx: &mut App) {
    match result {
        Ok(outcome) => {
            let words = finished_words(&outcome);
            let note = if outcome.failed.is_empty() {
                Notification::info(words)
            } else {
                Notification::error(words)
            };
            window.push_notification(note, cx);
        }
        Err(words) => {
            window.push_notification(Notification::error(format!("Couldn't seal: {words}")), cx);
        }
    }
    // Notes the pass rewrote are open in editors; Touch ID keeps the keys.
    locked::after_password(window, cx);
    crate::editor::peers::lock_keys_changed_everywhere(cx);
    cx.refresh_windows();
}

/// Undo: with the lock password, the attachments a stopped pass sealed are
/// readable files again.
fn undo(window: &mut Window, cx: &mut App) {
    let spec = FormSpec::new("Undo sealing", "Undo")
        .description("Type your lock password. The attachments the stopped pass sealed become readable files again.")
        .field(Field::secret("Lock password"));
    dialogs::form(spec, window, cx, move |values, window, cx| {
        let password = match locked::password(&values[0]) {
            Ok(password) => password,
            Err(submit) => return submit,
        };
        let store = locked::store(cx);
        locked::run_in_background(
            window,
            cx,
            move || {
                let keys: Vec<Arc<LockKey>> = migrate::keys_for(&store, &password)?
                    .into_iter()
                    .map(Arc::new)
                    .collect();
                let (undone, _) =
                    migrate::undo_all(&store, &keys).map_err(|err| err.to_string())?;
                Ok(undone)
            },
            move |undone, window, cx| {
                window.push_notification(Notification::info(undone_words(undone)), cx);
                crate::editor::peers::lock_keys_changed_everywhere(cx);
                cx.refresh_windows();
                Ok(())
            },
        )
    });
}

#[cfg(test)]
mod tests {
    use kanso_engine::lock::files::Summary;
    use kanso_engine::lock::files::migrate::Failure;
    use proptest::prelude::*;

    use super::*;

    fn pending(legacy: usize, unsealed: usize, stopped: usize) -> Pending {
        Pending {
            legacy,
            unsealed,
            stopped,
        }
    }

    #[::core::prelude::v1::test]
    fn the_prompt_follows_the_design_and_counts_when_the_notes_are_open() {
        let closed = prompt_words(&pending(3, 0, 0), None);
        assert!(closed.starts_with(
            "Attachments of your locked notes aren't sealed yet. Enter your lock password to seal them now."
        ));
        assert!(closed.contains("3 locked notes"));
        assert!(closed.contains("once the password opens the notes"));
        let open = prompt_words(
            &pending(1, 0, 0),
            Some(&Scan {
                notes: 1,
                unopened: 0,
                summary: Summary {
                    files: 12,
                    bytes: 84 * 1024 * 1024,
                    shared: 2,
                },
            }),
        );
        assert!(open.contains("1 locked note "));
        assert!(open.contains("12 attachments (84 MB) will be sealed."));
        assert!(open.contains("2 attachments are also used by other notes"));
        // Some notes open and some not: no count of part of them.
        let partly = prompt_words(
            &pending(2, 0, 0),
            Some(&Scan {
                notes: 1,
                unopened: 1,
                summary: Summary {
                    files: 5,
                    bytes: 1024,
                    shared: 0,
                },
            }),
        );
        assert!(!partly.contains("5 attachments"));
        assert!(partly.contains("once the password opens the notes"));
    }

    #[::core::prelude::v1::test]
    fn the_end_says_what_was_sealed_copied_and_left() {
        let outcome = Outcome {
            notes: 3,
            sealed: 4,
            copied: 2,
            unopened: 1,
            ..Outcome::default()
        };
        let words = finished_words(&outcome);
        assert!(words.starts_with("Sealed 6 attachments in 3 locked notes."));
        assert!(words.contains("2 attachments are also used by other notes"));
        assert!(words.contains("1 locked note use another lock password"));
        let none = finished_words(&Outcome {
            notes: 2,
            ..Outcome::default()
        });
        assert_eq!(
            none,
            "Checked 2 locked notes: no attachments needed sealing."
        );
        let failed = finished_words(&Outcome {
            notes: 1,
            failed: vec![Failure {
                title: "T".into(),
                words: "Nope.".into(),
            }],
            ..Outcome::default()
        });
        assert!(failed.contains("couldn't be sealed"));
        assert!(failed.ends_with("Nope."));
    }

    #[::core::prelude::v1::test]
    fn opening_a_note_locked_before_says_what_it_sealed_or_which_file_stayed() {
        use kanso_engine::lock::files::Counts;
        let sealed = |sealed, copied| {
            opened_words(&locked::SealedOnOpen::Sealed(Counts {
                sealed,
                copied,
                ..Counts::default()
            }))
        };
        assert_eq!(
            sealed(0, 0),
            None,
            "a note with no attachments says nothing"
        );
        assert_eq!(sealed(1, 0).as_deref(), Some("Sealed 1 attachment."));
        assert_eq!(
            sealed(2, 1).as_deref(),
            Some(
                "Sealed 3 attachments. 1 attachment is also used by other notes, so this note has its own sealed copy."
            )
        );
        let failed = opened_words(&locked::SealedOnOpen::Failed(
            "\u{201c}scan.pdf\u{201d}: The attachment couldn't be sealed safely.".into(),
        ))
        .unwrap();
        assert!(failed.contains("\u{201c}scan.pdf\u{201d}"), "{failed}");
        assert!(failed.contains("the note opens as usual"));
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            rng_seed: proptest::test_runner::RngSeed::Fixed(0x0044_0007),
            ..ProptestConfig::default()
        })]

        /// Whatever the counts, the words use the right number and plural.
        #[::core::prelude::v1::test]
        fn prop_the_words_count_what_they_say(
            (counts, files, bytes) in ((0usize..50, 0usize..50, 0usize..50), 0usize..500, 0u64..(1 << 40)),
        ) {
            let (legacy, unsealed, stopped) = counts;
            let pending = pending(legacy, unsealed, stopped);
            let total = pending.notes();
            let scan = Scan {
                notes: total,
                unopened: 0,
                summary: Summary { files, bytes, shared: 0 },
            };
            let prompt = prompt_words(&pending, Some(&scan));
            let notes_words = notes(total);
            prop_assert!(prompt.contains(&notes_words));
            if total > 0 {
                prop_assert!(prompt.contains(&attachments(files)));
                prop_assert!(prompt.contains(&size_words(bytes)));
            }
            prop_assert!(reminder_words(&pending).contains(&notes_words));
            let one = notes(1);
            prop_assert_eq!(one, "1 locked note");
        }

        /// What an open says counts every file it sealed or copied.
        #[::core::prelude::v1::test]
        fn prop_the_open_words_count_the_sealed_files(sealed in 0usize..200, copied in 0usize..200) {
            let counts = kanso_engine::lock::files::Counts { sealed, copied, ..Default::default() };
            let words = opened_words(&locked::SealedOnOpen::Sealed(counts));
            prop_assert_eq!(words.is_none(), sealed + copied == 0);
            if let Some(words) = words {
                let expected = format!("Sealed {}.", attachments(sealed + copied));
                prop_assert!(words.starts_with(&expected));
                prop_assert_eq!(words.contains("own sealed"), copied > 0);
            }
        }
    }
}
