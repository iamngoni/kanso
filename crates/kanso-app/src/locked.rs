//! Locked notes in the app: the lock keys this session holds, what the
//! editor shows for a locked note, and the dialogs that lock, unlock and
//! re-key them (`kanso_sync::lock` has the cryptography).
//!
//! Nothing is unlocked when Kanso starts. Typing the lock password once
//! opens every note locked with it until Lock Notes Now, quitting, or
//! `AUTO_LOCK` (ten minutes, in `kanso_engine::lock`) after it was typed.
//! Opening locked notes doesn't keep them open longer; saving one does, so
//! they don't close while you write. The editor holds an open note's text
//! in memory as any other note's; the store only ever gets it sealed.

use std::sync::Arc;
use std::time::Instant;

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::Button;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;
use kanso_core::Note;
use kanso_core::model::note_files_from_extra;
use kanso_store::Store;
use kanso_sync::SecretStore;
use kanso_sync::keychain::{delete_lock_keys, load_lock_keys, save_lock_keys};
use kanso_sync::lock::{Envelope, LockError, LockKey, LockPassword, SealedFileMetadata};

use crate::dialogs::{self, Field, FormSpec, FormView, Progress, Submit};
use crate::editor::NoteEditor;
use crate::services::Services;
use crate::touch_id::{secret_store_name, unlock_name};
use crate::workspace::AppState;

actions!(
    kanso,
    [
        LockNote,
        UnlockNote,
        RemoveLock,
        LockNotesNow,
        ChangeLockPassword,
        TurnOnTouchId,
        TurnOffTouchId
    ]
);

pub use kanso_engine::lock::LockView;

/// The lock keys typed this session, which `kanso_engine::lock::Session`
/// holds; this is the window's global around it. The first key is the one
/// new locks use; the others open notes locked on another device.
#[derive(Default)]
pub struct LockKeys(kanso_engine::lock::Session);

impl Global for LockKeys {}

// no-prop-test: delegates to kanso_engine::lock::Session, tested there.
impl LockKeys {
    fn get(cx: &App) -> Option<&LockKeys> {
        cx.try_global::<LockKeys>()
    }

    fn get_mut(cx: &mut App) -> &mut LockKeys {
        cx.default_global::<LockKeys>()
    }

    /// The key new locks use, if a password was typed.
    pub fn current(cx: &App) -> Option<Arc<LockKey>> {
        Self::get(cx).and_then(|keys| keys.0.current())
    }

    /// Whether any locked notes are open now.
    pub fn is_unlocked(cx: &App) -> bool {
        Self::get(cx).is_some_and(|keys| keys.0.is_unlocked())
    }

    pub(crate) fn fitting(cx: &App, envelope: &Envelope) -> Option<Arc<LockKey>> {
        Self::get(cx)?.0.fitting(envelope)
    }

    /// Whether a key this session holds opens `target`.
    pub(crate) fn opens(target: &Target, cx: &App) -> bool {
        Self::get(cx).is_some_and(|keys| target.is_open(&keys.0))
    }

    /// The file keys this session holds, for readers that keep them. They
    /// empty when the lock keys do.
    pub(crate) fn file_keys(cx: &mut App) -> kanso_engine::lock::KeyBook {
        Self::get_mut(cx).0.file_keys()
    }

    /// [`LockKeys::file_keys`] for a reader that doesn't keep them: with no
    /// session yet, nothing is open.
    pub(crate) fn peek_file_keys(cx: &App) -> kanso_engine::lock::KeyBook {
        Self::get(cx)
            .map(|keys| keys.0.file_keys())
            .unwrap_or_default()
    }

    /// The key that seals `note`: the one that opens its lock when held,
    /// else the one new locks use.
    pub(crate) fn sealing_key(note: &Note, cx: &App) -> Option<Arc<LockKey>> {
        let fitting = match Envelope::from_extra(&note.extra) {
            Some(Ok(envelope)) => Self::fitting(cx, &envelope),
            _ => None,
        };
        fitting.or_else(|| Self::current(cx))
    }

    pub(crate) fn add(key: LockKey, cx: &mut App) {
        Self::get_mut(cx).0.add(key, Instant::now());
    }

    /// Takes the keys this session holds, with `key` alone in their place
    /// when given (a new password).
    pub(crate) fn replace(key: Option<LockKey>, cx: &mut App) {
        Self::get_mut(cx).0.replace(key, Instant::now());
    }

    /// Whether `AUTO_LOCK` has passed at `now` since the keys were added
    /// or a locked note was saved.
    pub fn idle(cx: &App, now: Instant) -> bool {
        Self::get(cx).is_some_and(|keys| keys.0.idle(now))
    }

    pub(crate) fn params(cx: &App) -> kanso_sync::KdfParams {
        Self::get(cx).map_or(kanso_sync::lock::LOCK_KDF, |keys| keys.0.params())
    }

    /// Cheap Argon2 for new keys, so tests derive in milliseconds.
    #[cfg(test)]
    pub fn use_test_params(cx: &mut App) {
        Self::get_mut(cx).0.set_params(kanso_sync::KdfParams::TEST);
    }
}

/// The mark lists show on a locked note, and its word: the lock, drawn open
/// with "Unlocked" while this session holds the keys, so it's plain that
/// the note opens without the password until Lock Notes Now.
pub fn note_mark(cx: &App) -> (gpui_kit::component::Icon, &'static str) {
    if LockKeys::is_unlocked(cx) {
        (
            gpui_kit::component::Icon::new(gpui_kit::assets::IconName::LockOpen),
            "Unlocked",
        )
    } else {
        (
            crate::icon_themes::library_icon(crate::icon_themes::Slot::NoteLocked),
            "Locked",
        )
    }
}

/// Puts a locked note's text in its `body` when a key this session holds
/// opens it, and says what the editor should show. A note already revealed
/// is revealed again to the same text.
// no-prop-test: delegates to kanso_engine::lock::Session::reveal, tested there.
pub fn reveal(note: &mut Note, cx: &mut App) -> LockView {
    let store = cx.try_global::<AppState>().map(|state| state.store.clone());
    match LockKeys::get(cx) {
        Some(keys) => keys.0.reveal(note, store.as_deref()),
        None => kanso_engine::lock::Session::default().reveal(note, store.as_deref()),
    }
}

/// Seals an open locked note's `body` into its `kansoLock` and empties the
/// body, as the store takes it. The note keeps its salt when this session
/// holds its key; otherwise it moves to the current one.
// no-prop-test: delegates to kanso_engine::lock::Session::seal, tested there.
pub fn seal(note: &mut Note, cx: &mut App) -> Result<(), String> {
    LockKeys::get_mut(cx).0.seal(note, Instant::now())
}

/// Opens the sealed metadata of a file owned by this open note. The note's
/// own envelope and ownership wrap must both match a key held by this app
/// session; the local API never calls this path.
pub(crate) fn opened_file(
    note: &Note,
    file_id: &str,
    cx: &App,
) -> Option<(SealedFileMetadata, Vec<u8>)> {
    use kanso_engine::lock::files::{Reading, read_file};
    let envelope = Envelope::from_extra(&note.extra)?.ok()?;
    let key = LockKeys::fitting(cx, &envelope)?;
    // Only a file this note's own list holds.
    note_files_from_extra(&note.extra)?
        .into_iter()
        .find(|file| file.file == file_id)?;
    let store = store(cx);
    if store.attachment_state(file_id).ok()? != kanso_store::AttachmentState::Sealed {
        return None;
    }
    // The session's file keys (#44 step 3), filled from the note's list
    // the first time one of its files is read.
    let keys = LockKeys::peek_file_keys(cx);
    let opened = match read_file(&store, file_id, &keys) {
        Reading::Opened(opened) => opened,
        Reading::Locked => {
            keys.remember_note(note, &key);
            match read_file(&store, file_id, &keys) {
                Reading::Opened(opened) => opened,
                Reading::Locked | Reading::Missing => return None,
            }
        }
        Reading::Missing => return None,
    };
    let public_in = store
        .file_meta(file_id)
        .ok()
        .flatten()
        .map(|meta| meta.public_in)
        .unwrap_or_default();
    Some((
        SealedFileMetadata {
            name: opened.name,
            content_type: opened.content_type,
            public_in,
        },
        opened.bytes,
    ))
}

/// The key this session holds for note `note_id`, when the note is locked and
/// a key opens it. A recording into the note seals its audio under it.
// no-prop-test: reads the store and the session's keys.
pub(crate) fn key_for_note(note_id: &str, cx: &App) -> Option<Arc<LockKey>> {
    let note = store(cx).note(note_id).ok()??;
    let envelope = Envelope::from_extra(&note.extra)?.ok()?;
    LockKeys::fitting(cx, &envelope)
}

/// The key this session holds for where a recording goes: a locked note
/// it opens, or the locked notebook a new note goes into (#44 step 3 seals
/// that note and its audio).
// no-prop-test: reads the store and the session's keys.
pub(crate) fn key_for_destination(
    destination: &kanso_engine::recording_flow::Destination,
    cx: &App,
) -> Option<Arc<LockKey>> {
    use kanso_engine::recording_flow::Destination;
    match destination {
        Destination::ThisNote { note_id } => key_for_note(note_id, cx),
        Destination::NewNote { book_id } => {
            let target = Target::of_book(book_id.as_deref()?, &store(cx))?;
            LockKeys::fitting(cx, target.envelope.as_ref().ok()?)
        }
    }
}

/// The bytes of the audio files `file_ids` of note `note_id` when the note is
/// locked (opened with the key this session holds), or `None` when it isn't
/// and the files are ordinary attachments. Fails in plain words when the
/// note is locked and no key opens it.
// no-prop-test: reads the store and the session's keys; the sealing has its
// tests in `kanso_engine::recording_seal`.
pub(crate) fn opened_parts(
    note_id: &str,
    file_ids: &[String],
    cx: &App,
) -> Result<Option<Vec<Vec<u8>>>, String> {
    let Some(note) = store(cx).note(note_id).ok().flatten() else {
        return Ok(None);
    };
    if !note.is_locked() {
        return Ok(None);
    }
    file_ids
        .iter()
        .map(|id| {
            opened_file(&note, id, cx)
                .map(|(_, bytes)| bytes)
                .ok_or_else(|| "Unlock the note to transcribe its recording.".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

// MARK: Dialogs

pub(crate) fn store(cx: &App) -> Arc<Store> {
    cx.global::<AppState>().store.clone()
}

/// Runs `work` on a thread of its own, then `done` with its result on the
/// main thread, in `window`, and tells the form how it went.
pub(crate) fn run_in_background<T: Send + 'static>(
    window: &Window,
    cx: &mut App,
    work: impl FnOnce() -> Result<T, String> + Send + 'static,
    done: impl FnOnce(T, &mut Window, &mut App) -> Result<(), String> + 'static,
) -> Submit {
    let handle = window.window_handle();
    let (progress, listening) = async_channel::bounded(1);
    let (tx, rx) = async_channel::bounded(1);
    let spawned = std::thread::Builder::new()
        .name("kanso-lock".into())
        .spawn(move || {
            tx.try_send(work()).ok();
        });
    if spawned.is_err() {
        return Submit::Invalid("Couldn't start. Try again.".into());
    }
    cx.spawn(async move |cx| {
        let result = match rx.recv().await {
            Ok(Ok(value)) => cx.update(|cx| {
                handle
                    .update(cx, |_, window, cx| done(value, window, cx))
                    .unwrap_or_else(|_| Err("The window closed.".into()))
            }),
            Ok(Err(words)) => Err(words),
            Err(_) => Err("Something went wrong. Try again.".into()),
        };
        progress.try_send(Progress::Done(result)).ok();
    })
    .detach();
    Submit::Running(listening)
}

// no-prop-test: delegates to kanso_engine::lock::password, tested there.
pub(crate) fn password(text: &str) -> Result<LockPassword, Submit> {
    kanso_engine::lock::password(text).map_err(Submit::Invalid)
}

pub(crate) use kanso_engine::lock::{Target, key_for, sentence};

/// Seals `note`, which isn't sealed, for the locked notebook it is in or
/// is going into, keeping its text in `body` for the editor; `stored` is
/// what the store then takes. Fails when no key is held.
// no-prop-test: delegates to kanso_engine::lock::Session::seal_for_notebook, tested there.
pub(crate) fn seal_for_notebook(note: &mut Note, cx: &App) -> Result<Note, String> {
    match LockKeys::get(cx) {
        Some(keys) => keys.0.seal_for_notebook(note, &store(cx)),
        None => kanso_engine::lock::Session::default().seal_for_notebook(note, &store(cx)),
    }
}

/// Seals the notes in locked notebooks that aren't sealed yet (made there
/// empty, or put there by another device or an older Kanso), now that a
/// key is held. One that changed meanwhile waits for the next time.
// no-prop-test: delegates to kanso_engine::lock::Session::seal_unsealed, tested there.
pub(crate) fn seal_unsealed(cx: &mut App) {
    let Some(state) = cx.try_global::<AppState>() else {
        return;
    };
    let store = state.store.clone();
    if let Some(keys) = LockKeys::get(cx) {
        keys.0.seal_unsealed(&store);
    }
}

/// Lock Note: seals the open note. The first time this session, Touch ID or
/// Windows Hello (when on) or the lock password opens the keys, or a
/// password is made when no note is locked yet. Returns the form when it
/// asks for a password.
pub fn lock_open_note(
    editor: Entity<NoteEditor>,
    window: &mut Window,
    cx: &mut App,
) -> Option<Entity<FormView>> {
    let note = editor.read(cx).current_note(cx)?;
    if note.is_locked() {
        return None;
    }
    let title = display_title(&note);
    let summary = kanso_engine::lock::files::summarize(&store(cx), std::slice::from_ref(&note));
    if let Some(key) = LockKeys::current(cx) {
        dialogs::confirm(
            "Lock this note?",
            lock_description(&title, &summary),
            "Lock",
            window,
            cx,
            move |window, cx| {
                editor.update(cx, |editor, cx| editor.lock_with(&key, window, cx));
            },
        );
        return None;
    }
    let Some(locked) = any_lock(&store(cx)) else {
        return Some(set_password_form(editor, title, summary, window, cx));
    };
    if touch_id_on(cx) {
        // The fingerprint is the go-ahead; without it, the password is.
        let fallback = (editor.clone(), locked.clone(), title.clone());
        with_touch_id(
            window,
            cx,
            move |window, cx| match LockKeys::current(cx) {
                Some(key) => editor.update(cx, |editor, cx| editor.lock_with(&key, window, cx)),
                None => {
                    lock_password_form(editor, locked, title, summary, window, cx);
                }
            },
            move |window, cx| {
                let (editor, locked, title) = fallback;
                lock_password_form(editor, locked, title, summary, window, cx);
            },
        );
        return None;
    }
    Some(lock_password_form(
        editor, locked, title, summary, window, cx,
    ))
}

/// What the lock dialog says: how the note opens, what is deleted and what
/// is hidden, and that its pictures and files are sealed too (and copied,
/// where another note shows them).
pub(crate) fn lock_description(
    title: &str,
    summary: &kanso_engine::lock::files::Summary,
) -> String {
    with_files(
        format!(
            "\u{201c}{title}\u{201d} will only open with your lock password. Its earlier versions are deleted, and it won't show in search, backlinks or tasks while locked."
        ),
        summary,
    )
}

/// `description`, with the sentence about the note's pictures and files.
pub(crate) fn with_files(
    description: String,
    summary: &kanso_engine::lock::files::Summary,
) -> String {
    let files = crate::lock_flow::dialog_sentence(summary);
    if files.is_empty() {
        description
    } else {
        format!("{description} {files}")
    }
}

pub(crate) use kanso_engine::lock::any_lock;

/// Locking a note with the lock password set before.
fn lock_password_form(
    editor: Entity<NoteEditor>,
    locked: Target,
    title: String,
    summary: kanso_engine::lock::files::Summary,
    window: &mut Window,
    cx: &mut App,
) -> Entity<FormView> {
    let spec = FormSpec::new("Lock this note", "Lock")
        .description(with_files(
            format!(
                "Type your lock password to lock \u{201c}{title}\u{201d}. Its earlier versions are deleted."
            ),
            &summary,
        ))
        .field(Field::secret("Lock password"));
    dialogs::form(spec, window, cx, move |values, window, cx| {
        let password = match password(&values[0]) {
            Ok(password) => password,
            Err(submit) => return submit,
        };
        let locked = locked.clone();
        let editor = editor.clone();
        run_in_background(
            window,
            cx,
            move || key_for(&locked, &password),
            move |key, window, cx| {
                LockKeys::add(key, cx);
                let key = LockKeys::current(cx).expect("a key was just added");
                editor.update(cx, |editor, cx| editor.lock_with(&key, window, cx));
                after_password(window, cx);
                Ok(())
            },
        )
    })
}

/// Locking the first note: a new lock password, typed twice.
fn set_password_form(
    editor: Entity<NoteEditor>,
    title: String,
    summary: kanso_engine::lock::files::Summary,
    window: &mut Window,
    cx: &mut App,
) -> Entity<FormView> {
    let spec = FormSpec::new("Set a lock password", "Lock")
        .description(with_files(
            format!(
                "Locked notes open with this password, on this device and any other that syncs with it. Kanso can't recover it: forget it, and the notes locked with it can't be read. \u{201c}{title}\u{201d}'s earlier versions are deleted."
            ),
            &summary,
        ))
        .field(Field::secret("Lock password").hint("At least 8 characters."))
        .field(Field::secret("Type it again"));
    dialogs::form(spec, window, cx, move |values, window, cx| {
        if values[0] != values[1] {
            return Submit::Invalid("The passwords don't match.".into());
        }
        let password = match password(&values[0]) {
            Ok(password) => password,
            Err(submit) => return submit,
        };
        let params = LockKeys::params(cx);
        let editor = editor.clone();
        run_in_background(
            window,
            cx,
            move || LockKey::create(&password, params).map_err(|err| sentence(&err)),
            move |key, window, cx| {
                LockKeys::replace(Some(key), cx);
                let key = LockKeys::current(cx).expect("a key was just set");
                editor.update(cx, |editor, cx| editor.lock_with(&key, window, cx));
                after_password(window, cx);
                Ok(())
            },
        )
    })
}

/// Unlock: Touch ID or Windows Hello when it's on, else (or when it fails) the lock password,
/// opens the open note and every other note locked with the same password.
/// Returns the form when it asks for the password.
pub fn unlock_open_note(
    editor: Entity<NoteEditor>,
    window: &mut Window,
    cx: &mut App,
) -> Option<Entity<FormView>> {
    let note = editor.read(cx).current_note(cx)?;
    if !editor.read(cx).lock_view().is_closed() {
        return None;
    }
    let target = Target::of_note(&note, &store(cx))?;
    if touch_id_on(cx) {
        let fallback = (editor.clone(), target.clone());
        with_touch_id(
            window,
            cx,
            move |window, cx| {
                editor.update(cx, |editor, cx| editor.lock_keys_changed(window, cx));
                crate::editor::peers::lock_keys_changed_everywhere(cx);
                cx.refresh_windows();
                // Keys kept before this note was locked elsewhere don't open it.
                if editor.read(cx).lock_view().is_closed() {
                    password_unlock(editor, target, window, cx);
                }
            },
            move |window, cx| {
                let (editor, target) = fallback;
                password_unlock(editor, target, window, cx);
            },
        );
        return None;
    }
    Some(password_unlock(editor, target, window, cx))
}

fn password_unlock(
    editor: Entity<NoteEditor>,
    target: Target,
    window: &mut Window,
    cx: &mut App,
) -> Entity<FormView> {
    let spec = FormSpec::new("Unlock note", "Unlock").field(Field::secret("Lock password"));
    dialogs::form(spec, window, cx, move |values, window, cx| {
        let password = match LockPassword::new(&values[0]) {
            Ok(password) => password,
            // A shorter text can't be the password.
            Err(_) => return Submit::Invalid(sentence(&LockError::WrongPassword)),
        };
        let target = target.clone();
        let editor = editor.clone();
        run_in_background(
            window,
            cx,
            move || key_for(&target, &password),
            move |key, window, cx| {
                LockKeys::add(key, cx);
                seal_unsealed(cx);
                editor.update(cx, |editor, cx| editor.lock_keys_changed(window, cx));
                crate::editor::peers::lock_keys_changed_everywhere(cx);
                cx.refresh_windows();
                after_password(window, cx);
                Ok(())
            },
        )
    })
}

// MARK: A lock that stopped

/// What to do with a lock that stopped, once its password is typed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resume {
    Finish,
    Undo,
}

/// Asks for the lock password of a lock that stopped between a note's files
/// (no key the session holds is the one it used), then finishes or undoes
/// it. The password derives the key from the lock's own header, so it works
/// even when that lock was the first one and left no other note to try.
///
/// The editor passes its note in: it asks from inside its own update, where
/// reading the editor entity would panic.
pub fn ask_interrupted(
    editor: Entity<NoteEditor>,
    note: Note,
    resume: Resume,
    window: &mut Window,
    cx: &mut App,
) -> Option<Entity<FormView>> {
    let header = kanso_engine::lock::files::interrupted_header(&note)?;
    let (title, button) = match resume {
        Resume::Finish => ("Finish locking", "Finish"),
        Resume::Undo => ("Undo locking", "Undo"),
    };
    let spec = FormSpec::new(title, button)
        .description(
            "Type the lock password this note was being locked with. Its pictures and files were sealed with it.",
        )
        .field(Field::secret("Lock password"));
    Some(dialogs::form(
        spec,
        window,
        cx,
        move |values, window, cx| {
            let password = match LockPassword::new(&values[0]) {
                Ok(password) => password,
                Err(_) => return Submit::Invalid(sentence(&LockError::WrongPassword)),
            };
            let (derive, note, kept, editor) =
                (header.clone(), note.clone(), header.clone(), editor.clone());
            run_in_background(
                window,
                cx,
                move || {
                    let key =
                        LockKey::for_envelope(&password, &derive).map_err(|err| sentence(&err))?;
                    if kanso_engine::lock::files::opens_interrupted(&note, &key) {
                        Ok(key)
                    } else {
                        Err(sentence(&LockError::WrongPassword))
                    }
                },
                move |key, window, cx| {
                    LockKeys::add(key, cx);
                    // The key the session holds for the lock that stopped.
                    let key = LockKeys::fitting(cx, &kept)
                        .or_else(|| LockKeys::current(cx))
                        .ok_or_else(|| "The notes were locked again.".to_owned())?;
                    editor.update(cx, |editor, cx| match resume {
                        Resume::Finish => editor.lock_with(&key, window, cx),
                        Resume::Undo => editor.undo_with(&key, window, cx),
                    });
                    after_password(window, cx);
                    Ok(())
                },
            )
        },
    ))
}

// MARK: Touch ID and Windows Hello

/// Whether Touch ID (or Windows Hello) unlocks locked notes on this device
/// now.
pub fn touch_id_on(cx: &App) -> bool {
    cx.try_global::<AppState>()
        .is_some_and(|state| state.config.locked.touch_id == Some(true))
        && crate::touch_id::available(cx)
}

fn secrets(cx: &App) -> Option<Arc<dyn SecretStore>> {
    cx.try_global::<Services>()?.secrets.clone()
}

/// The library's ID, which names its Keychain (or Credential Manager) items.
// no-prop-test: delegates to kanso_engine::lock::library_id, tested there.
fn library_id(store: &Store) -> Result<String, String> {
    kanso_engine::lock::library_id(store, &kanso_sync::Machine::this_mac())
}

/// Asks for a fingerprint (or Windows Hello), takes the lock keys kept for it
/// into this session and runs `then`; runs `otherwise` when it is off, is
/// cancelled or fails, or no keys are kept.
pub(crate) fn with_touch_id(
    window: &mut Window,
    cx: &mut App,
    then: impl FnOnce(&mut Window, &mut App) + 'static,
    otherwise: impl FnOnce(&mut Window, &mut App) + 'static,
) {
    let answer = if touch_id_on(cx) {
        crate::touch_id::authenticate("unlock your locked notes", cx)
    } else {
        None
    };
    let (Some(answer), Some(secrets)) = (answer, secrets(cx)) else {
        otherwise(window, cx);
        return;
    };
    let store = store(cx);
    let handle = window.window_handle();
    cx.spawn(async move |cx| {
        let keys = if answer.recv().await.unwrap_or(false) {
            cx.background_spawn(async move {
                let library_id = library_id(&store).ok()?;
                load_lock_keys(secrets.as_ref(), &library_id).ok().flatten()
            })
            .await
        } else {
            None
        };
        cx.update(|cx| {
            handle
                .update(cx, |_, window, cx| match keys {
                    Some(keys) => {
                        for key in keys {
                            LockKeys::add(key, cx);
                        }
                        seal_unsealed(cx);
                        then(window, cx);
                    }
                    None => otherwise(window, cx),
                })
                .ok();
        });
    })
    .detach();
}

/// Keeps the lock keys this session holds for Touch ID, in the background,
/// when Touch ID is on: after a password opened another one, or a new
/// password replaced them.
fn remember_keys(cx: &mut App) {
    if !touch_id_on(cx) {
        return;
    }
    let Some(secrets) = secrets(cx) else {
        return;
    };
    let keys = LockKeys::get(cx)
        .map(|keys| keys.0.keys().to_vec())
        .unwrap_or_default();
    let store = store(cx);
    cx.background_spawn(async move {
        let saved = library_id(&store).and_then(|library_id| {
            let keys: Vec<&LockKey> = keys.iter().map(|key| key.as_ref()).collect();
            save_lock_keys(secrets.as_ref(), &library_id, &keys).map_err(|err| err.to_string())
        });
        if let Err(err) = saved {
            eprintln!(
                "kanso: locked notes: couldn't keep the keys for {}: {err}",
                crate::touch_id::NAME
            );
        }
    })
    .detach();
}

/// After the lock password worked: keeps the keys for Touch ID when it's on,
/// or offers it once when it could be.
pub(crate) fn after_password(window: &mut Window, cx: &mut App) {
    remember_keys(cx);
    seal_unsealed(cx);
    let asked = cx.global::<AppState>().config.locked.touch_id.is_some();
    if asked || !crate::touch_id::available(cx) || secrets(cx).is_none() {
        return;
    }
    AppState::update_config(cx, |config| config.locked.touch_id = Some(false));
    let notification = Notification::info(
        concat!(
            "Use ",
            unlock_name!(),
            " to unlock your locked notes on this device? The password still works, and other devices need it."
        ),
    )
    .action(|_, _, _| {
        Button::new("turn-on-touch-id")
            .label(concat!("Use ", unlock_name!()))
            .on_click(|_, window, cx| {
                window.dispatch_action(Box::new(TurnOnTouchId), cx);
            })
    });
    window.push_notification(notification, cx);
}

/// Turn On Touch ID (or Windows Hello): after a fingerprint, keeps the lock
/// keys this session holds in the Keychain (or the Credential Manager), so
/// it opens locked notes on this device.
pub fn turn_on_touch_id(window: &mut Window, cx: &mut App) {
    let say = |words: &str, window: &mut Window, cx: &mut App| {
        window.push_notification(Notification::info(words.to_owned()), cx);
    };
    if !crate::touch_id::available(cx) {
        return say(
            concat!(unlock_name!(), " isn't set up on this device."),
            window,
            cx,
        );
    }
    let Some(secrets) = secrets(cx) else {
        return say(
            concat!(
                "This copy of Kanso can't use the ",
                secret_store_name!(),
                "."
            ),
            window,
            cx,
        );
    };
    if !LockKeys::is_unlocked(cx) {
        return say(
            concat!(
                "Unlock a locked note with your lock password first, then turn on ",
                unlock_name!(),
                "."
            ),
            window,
            cx,
        );
    }
    let Some(answer) = crate::touch_id::authenticate(
        concat!("turn on ", unlock_name!(), " for your locked notes"),
        cx,
    ) else {
        return;
    };
    let keys = LockKeys::get(cx)
        .map(|keys| keys.0.keys().to_vec())
        .unwrap_or_default();
    let store = store(cx);
    let handle = window.window_handle();
    cx.spawn(async move |cx| {
        if !answer.recv().await.unwrap_or(false) {
            return;
        }
        let saved = cx
            .background_spawn(async move {
                let library_id = library_id(&store)?;
                let keys: Vec<&LockKey> = keys.iter().map(|key| key.as_ref()).collect();
                save_lock_keys(secrets.as_ref(), &library_id, &keys).map_err(|err| err.to_string())
            })
            .await;
        cx.update(|cx| {
            handle
                .update(cx, |_, window, cx| match saved {
                    Ok(()) => {
                        AppState::update_config(cx, |config| config.locked.touch_id = Some(true));
                        window.push_notification(
                            Notification::success(concat!(
                                unlock_name!(),
                                " now unlocks your locked notes on this device."
                            )),
                            cx,
                        );
                    }
                    Err(err) => window.push_notification(
                        Notification::error(format!(
                            "Couldn't turn on {}: {err}",
                            crate::touch_id::NAME
                        )),
                        cx,
                    ),
                })
                .ok();
        });
    })
    .detach();
}

/// Turn Off Touch ID (or Windows Hello): forgets the keys kept for it; the password opens
/// locked notes again.
pub fn turn_off_touch_id(window: &mut Window, cx: &mut App) {
    AppState::update_config(cx, |config| config.locked.touch_id = Some(false));
    if let Some(secrets) = secrets(cx) {
        let store = store(cx);
        cx.background_spawn(async move {
            if let Ok(library_id) = library_id(&store) {
                delete_lock_keys(secrets.as_ref(), &library_id).ok();
            }
        })
        .detach();
    }
    window.push_notification(
        Notification::info("Locked notes open with the lock password again on this device."),
        cx,
    );
}

/// Remove Lock: keeps the open note's text as plain text again.
pub fn remove_lock(editor: Entity<NoteEditor>, window: &mut Window, cx: &mut App) {
    let view = editor.read(cx).lock_view();
    if view == LockView::Plain {
        return;
    }
    if view.is_closed() {
        unlock_open_note(editor, window, cx);
        return;
    }
    let in_locked_notebook = editor
        .read(cx)
        .note_id()
        .is_some_and(|id| store(cx).note_in_locked_book(id).unwrap_or(false));
    if in_locked_notebook {
        window.push_notification(
            Notification::info(
                "The note is in a locked notebook, so it stays locked. Move it out first.",
            ),
            cx,
        );
        return;
    }
    // The text as typed is what stays.
    editor.update(cx, |editor, cx| editor.flush(cx));
    dialogs::confirm(
        "Remove the lock?",
        "The note's text is kept as plain text again: search, backlinks, tasks and the local API can read it. Its pictures and files are decrypted again too; one that another locked note also shows is copied, so that note keeps its sealed copy.",
        "Remove Lock",
        window,
        cx,
        move |window, cx| editor.update(cx, |editor, cx| editor.remove_lock(window, cx)),
    );
}

/// Lock Notes Now: saves the open note and forgets the keys, so every
/// locked note needs the password again.
pub fn lock_now(editor: &Entity<NoteEditor>, window: &mut Window, cx: &mut App) {
    if !LockKeys::is_unlocked(cx) {
        return;
    }
    editor.update(cx, |editor, cx| editor.flush(cx));
    // Editors in other windows save too, while the keys can still seal.
    crate::workspace::for_each_editor(cx, |editor, cx| editor.flush(cx));
    LockKeys::replace(None, cx);
    // Nothing decoded from a sealed file outlives the keys.
    crate::attachments::forget_caches(cx);
    editor.update(cx, |editor, cx| editor.lock_keys_changed(window, cx));
    crate::editor::peers::lock_keys_changed_everywhere(cx);
    cx.refresh_windows();
}

/// Change Lock Password: seals every locked note again under a new password.
/// The old password must open them; a note it doesn't open (locked with
/// another one) is left as it was and counted.
pub fn change_password(
    editor: Entity<NoteEditor>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<FormView> {
    editor.update(cx, |editor, cx| editor.flush(cx));
    let spec = FormSpec::new("Change lock password", "Change Password")
        .description(
            "Every locked note is sealed again with the new password. Kanso can't recover it.",
        )
        .field(Field::secret("Current lock password"))
        .field(Field::secret("New lock password").hint("At least 8 characters."))
        .field(Field::secret("Type it again"));
    dialogs::form(spec, window, cx, move |values, window, cx| {
        if values[1] != values[2] {
            return Submit::Invalid("The new passwords don't match.".into());
        }
        let old = match LockPassword::new(&values[0]) {
            Ok(old) => old,
            Err(_) => return Submit::Invalid(sentence(&LockError::WrongPassword)),
        };
        let new = match password(&values[1]) {
            Ok(new) => new,
            Err(submit) => return submit,
        };
        let store = store(cx);
        let params = LockKeys::params(cx);
        let editor = editor.clone();
        run_in_background(
            window,
            cx,
            move || rekey(&store, &old, &new, params),
            move |(key, skipped), window, cx| {
                LockKeys::replace(Some(key), cx);
                // Touch ID (or Windows Hello) keeps the new key in place of
                // the old ones.
                remember_keys(cx);
                crate::editor::peers::lock_keys_changed_everywhere(cx);
                editor.update(cx, |editor, cx| {
                    editor.lock_keys_changed(window, cx);
                    if skipped > 0 {
                        editor.notice(
                            format!(
                                "{skipped} locked {} didn't open with the current password and kept it.",
                                if skipped == 1 {
                                    "note or notebook"
                                } else {
                                    "notes or notebooks"
                                }
                            ),
                            cx,
                        );
                    }
                });
                cx.refresh_windows();
                Ok(())
            },
        )
    })
}

use kanso_engine::lock::display_title;
pub use kanso_engine::lock::rekey;

#[cfg(test)]
mod tests {
    // The rekey tests are `kanso_engine::lock`'s.

    mod window {
        use std::sync::Arc;

        use gpui_kit::component::WindowExt as _;
        use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};
        use kanso_core::DocType;
        use kanso_store::{Origin, Put, Store};
        use kanso_sync::KdfParams;
        use kanso_sync::lock::{Envelope, LockKey, LockPassword};
        use serde_json::json;

        use std::time::Duration;

        use kanso_engine::lock::AUTO_LOCK;

        use super::super::{
            LockKeys, LockView, lock_now, lock_open_note, note_mark, remove_lock, reveal, seal,
            unlock_open_note,
        };
        use crate::config::AppConfig;
        use crate::dialogs::FormView;
        use crate::editor::NoteEditor;
        use crate::services::Services;
        use crate::workspace::{AppState, Workspace};

        struct Fixture {
            _dir: tempfile::TempDir,
            store: Arc<Store>,
            window: AnyWindowHandle,
            workspace: Entity<Workspace>,
            editor: Entity<NoteEditor>,
            note: String,
        }

        fn setup(cx: &mut TestAppContext) -> Fixture {
            setup_with(cx, None)
        }

        /// The fixture, with a Touch ID that answers as `touch_id` is told and
        /// a Keychain in memory.
        fn setup_with(
            cx: &mut TestAppContext,
            touch_id: Option<Arc<crate::touch_id::FakeTouchId>>,
        ) -> Fixture {
            cx.executor().allow_parking();
            let dir = tempfile::tempdir().unwrap();
            let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
            let book = store
                .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
                .unwrap()
                .id;
            let note = store
                .put(
                    DocType::Note,
                    json!({"title": "Plan", "body": "The secret plan.", "bookId": book}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap()
                .id;
            let (data_dir, app_store) = (dir.path().to_path_buf(), store.clone());
            let (window, workspace) = cx.update(|cx| {
                gpui_kit::init(cx);
                cx.set_global(AppState {
                    store: app_store,
                    data_dir,
                    config: AppConfig::default(),
                });
                let mut services = Services::default();
                if let Some(touch_id) = touch_id {
                    services.secrets = Some(Arc::new(kanso_sync::MemorySecrets::new()));
                    cx.set_global(crate::touch_id::TouchId(touch_id));
                }
                cx.set_global(services);
                LockKeys::use_test_params(cx);
                gpui_kit::open_window(Default::default(), cx, |window, cx| {
                    cx.new(|cx| Workspace::new(window, cx))
                })
                .unwrap()
            });
            cx.run_until_parked();
            let library = cx.read(|cx| workspace.read(cx).library.clone());
            let id = note.clone();
            library.update(cx, |library, cx| library.select(Some(id), cx));
            cx.run_until_parked();
            let editor = cx.read(|cx| workspace.read(cx).editor.clone());
            Fixture {
                _dir: dir,
                store,
                window,
                workspace,
                editor,
                note,
            }
        }

        impl Fixture {
            fn dialog_open(&self, cx: &mut TestAppContext) -> bool {
                self.window
                    .update(cx, |_, window, cx| window.has_active_dialog(cx))
                    .unwrap()
            }

            fn kept_keys(&self, cx: &mut TestAppContext) -> bool {
                let secrets = cx.read(|cx| cx.global::<Services>().secrets.clone().unwrap());
                let library_id = super::super::library_id(&self.store).unwrap();
                kanso_sync::keychain::load_lock_keys(secrets.as_ref(), &library_id)
                    .unwrap()
                    .is_some()
            }

            fn submit(&self, cx: &mut TestAppContext, form: &Entity<FormView>, values: &[&str]) {
                self.window
                    .update(cx, |_, window, cx| {
                        form.update(cx, |form, cx| {
                            form.fill(values, window, cx);
                            form.submit(window, cx);
                        })
                    })
                    .unwrap();
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
                loop {
                    cx.run_until_parked();
                    if !cx.read(|cx| form.read(cx).is_running()) {
                        break;
                    }
                    assert!(std::time::Instant::now() < deadline, "the task finished");
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
            }

            fn view(&self, cx: &mut TestAppContext) -> LockView {
                cx.read(|cx| self.editor.read(cx).lock_view())
            }

            fn text(&self, cx: &mut TestAppContext) -> String {
                cx.read(|cx| self.editor.read(cx).body_text(cx).to_string())
            }

            fn stored_body(&self, password: &str) -> Option<String> {
                let note = self.store.note(&self.note).unwrap().unwrap();
                assert_eq!(note.body, "", "the store never has the text");
                let envelope = Envelope::from_extra(&note.extra)?.unwrap();
                let key = LockKey::for_envelope(&LockPassword::new(password).unwrap(), &envelope)
                    .unwrap();
                key.open(&self.note, &envelope)
                    .ok()
                    .map(|body| body.to_string())
            }

            fn type_at_end(&self, cx: &mut TestAppContext, text: &str) {
                let body = cx.read(|cx| self.editor.read(cx).body_state());
                let text = text.to_owned();
                self.window
                    .update(cx, |_, window, cx| {
                        body.update(cx, |state, cx| {
                            let end = state.text().len();
                            state.set_selected_range(end..end, cx);
                            state.replace(text, window, cx);
                        })
                    })
                    .unwrap();
                self.editor.update(cx, |editor, cx| editor.flush(cx));
                cx.run_until_parked();
            }
        }

        #[gpui_kit::test]
        fn a_note_locks_saves_sealed_closes_and_opens_with_the_password(cx: &mut TestAppContext) {
            let f = setup(cx);
            let form = f
                .window
                .update(cx, |_, window, cx| {
                    lock_open_note(f.editor.clone(), window, cx)
                })
                .unwrap()
                .expect("the first lock sets a password");
            f.submit(cx, &form, &["correct horse", "correct horse"]);
            assert_eq!(f.view(cx), LockView::Open);
            assert_eq!(f.text(cx), "The secret plan.");
            assert_eq!(
                f.stored_body("correct horse").as_deref(),
                Some("The secret plan.")
            );
            assert!(f.store.revisions(&f.note).unwrap().is_empty());

            // Typing in it saves it sealed again.
            f.type_at_end(cx, " More.");
            assert_eq!(
                f.stored_body("correct horse").as_deref(),
                Some("The secret plan. More.")
            );

            // Lock Notes Now closes it: its text leaves the editor.
            f.window
                .update(cx, |_, window, cx| lock_now(&f.editor, window, cx))
                .unwrap();
            cx.run_until_parked();
            assert_eq!(f.view(cx), LockView::Closed(None));
            assert_eq!(f.text(cx), "");

            // A wrong password says so and leaves it closed; the right one opens it.
            let form = f
                .window
                .update(cx, |_, window, cx| {
                    unlock_open_note(f.editor.clone(), window, cx)
                })
                .unwrap()
                .expect("a closed note asks");
            f.submit(cx, &form, &["battery staple"]);
            assert_eq!(f.view(cx), LockView::Closed(None));
            f.submit(cx, &form, &["correct horse"]);
            assert_eq!(f.view(cx), LockView::Open);
            assert_eq!(f.text(cx), "The secret plan. More.");

            // Removing the lock keeps the text as plain text.
            f.window
                .update(cx, |_, window, cx| {
                    remove_lock(f.editor.clone(), window, cx);
                    window.close_dialog(cx);
                })
                .unwrap();
            f.window
                .update(cx, |_, window, cx| {
                    f.editor
                        .update(cx, |editor, cx| editor.remove_lock(window, cx))
                })
                .unwrap();
            cx.run_until_parked();
            let note = f.store.note(&f.note).unwrap().unwrap();
            assert!(!note.is_locked());
            assert_eq!(note.body, "The secret plan. More.");
            assert_eq!(f.view(cx), LockView::Plain);
        }

        const PIXEL: &[u8] = &[
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
            0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x44, 0x41, 0x54, 0x78,
            0x9c, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00,
            0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
        ];

        /// A picture in a note being locked is sealed with it, the open note
        /// draws it from memory, and Lock Notes Now leaves the closed stub and
        /// nothing decoded anywhere: not in the preview, not in the file keys.
        /// Finish locking with no key held asks for the password. The bar's
        /// button calls it from inside the editor's own update: reading the
        /// editor entity there panicked.
        #[gpui_kit::test]
        fn finish_locking_with_no_key_asks_for_the_password(cx: &mut TestAppContext) {
            use kanso_engine::lock::files::{self, Job, KeyBook, Phase, Progress};
            let f = setup(cx);
            let one = kanso_engine::attach::attach_image(&f.store, PIXEL, "png", None).unwrap();
            let two =
                kanso_engine::attach::attach_image(&f.store, &[PIXEL, b"2"].concat(), "png", None)
                    .unwrap();
            f.type_at_end(
                cx,
                &format!("\n\n![](inkdrop://{one})\n![](inkdrop://{two})"),
            );
            // A lock that stopped after its first file, the keys gone since.
            let key = LockKey::create(
                &LockPassword::new("correct horse").unwrap(),
                KdfParams::TEST,
            )
            .unwrap();
            let mut note = f.store.note(&f.note).unwrap().unwrap();
            let locking = std::collections::HashSet::from([note.id.clone()]);
            let mut copies = std::collections::HashMap::new();
            let mut watch = |progress: &Progress| {
                if progress.phase == Phase::Sealing && progress.done == 1 {
                    Err("stopped".to_owned())
                } else {
                    Ok(())
                }
            };
            let keys = KeyBook::default();
            let mut job = Job::new(&f.store, &keys, &locking, &mut copies, &mut watch);
            let text = note.body.clone();
            files::seal_note_files(&mut job, &mut note, &text, &key).unwrap_err();
            let library = cx.read(|cx| f.workspace.read(cx).library.clone());
            library.update(cx, |library, cx| library.select(None, cx));
            cx.run_until_parked();
            let id = f.note.clone();
            library.update(cx, |library, cx| library.select(Some(id), cx));
            cx.run_until_parked();

            f.window
                .update(cx, |_, window, cx| {
                    f.editor
                        .update(cx, |editor, cx| editor.finish_locking(window, cx))
                })
                .unwrap();
            cx.run_until_parked();
            assert!(f.dialog_open(cx), "the lock password is asked for");
        }

        #[gpui_kit::test]
        fn lock_notes_now_leaves_the_stub_and_no_decoded_image(cx: &mut TestAppContext) {
            let f = setup(cx);
            let image = kanso_engine::attach::attach_image(&f.store, PIXEL, "png", None).unwrap();
            f.type_at_end(cx, &format!("\n\n![](inkdrop://{image})"));
            let form = f
                .window
                .update(cx, |_, window, cx| {
                    lock_open_note(f.editor.clone(), window, cx)
                })
                .unwrap()
                .expect("the first lock sets a password");
            f.submit(cx, &form, &["correct horse", "correct horse"]);
            assert_eq!(f.view(cx), LockView::Open);
            // The picture is sealed in the database, and listed on the note.
            assert_eq!(
                f.store.attachment_state(&image).unwrap(),
                kanso_store::AttachmentState::Sealed
            );
            assert_ne!(
                f.store.stored_attachment(&image).unwrap().unwrap().data,
                PIXEL
            );
            assert!(
                f.store
                    .note(&f.note)
                    .unwrap()
                    .unwrap()
                    .extra
                    .contains_key("kansoFiles")
            );

            // The open note draws it from memory.
            let preview = cx.read(|cx| f.editor.read(cx).preview_for_test());
            assert!(
                cx.update(|cx| preview.update(cx, |preview, _| preview.resolve_for_test(&image)))
            );
            assert!(cx.read(|cx| preview.read(cx).decoded_for_test()) > 0);
            assert!(!cx.read(|cx| crate::attachments::FileKeys::peek(cx).is_empty()));

            f.window
                .update(cx, |_, window, cx| lock_now(&f.editor, window, cx))
                .unwrap();
            cx.run_until_parked();
            assert_eq!(f.view(cx), LockView::Closed(None));
            assert_eq!(f.text(cx), "");
            assert_eq!(cx.read(|cx| preview.read(cx).decoded_for_test()), 0);
            assert!(cx.read(|cx| crate::attachments::FileKeys::peek(cx).is_empty()));
            // And nothing opens the file now.
            let reading = cx.read(|cx| crate::attachments::open(&f.store, &image, cx));
            assert_eq!(reading, crate::attachments::Opened::Locked);
        }

        #[gpui_kit::test]
        fn a_second_note_locks_with_the_password_already_set(cx: &mut TestAppContext) {
            let f = setup(cx);
            // A note locked earlier, on another device, sets the password to type.
            let elsewhere = LockKey::create(
                &LockPassword::new("correct horse").unwrap(),
                KdfParams::TEST,
            )
            .unwrap();
            let book = f.store.books().unwrap()[0].id.clone();
            let other = f
                .store
                .put(
                    DocType::Note,
                    json!({"title": "Old", "body": "", "bookId": book}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap()
                .id;
            let mut note = f.store.note(&other).unwrap().unwrap();
            Envelope::store(Some(&elsewhere.seal(&other, "older")), &mut note.extra);
            f.store
                .lock_note(serde_json::to_value(&note).unwrap())
                .unwrap();

            let form = f
                .window
                .update(cx, |_, window, cx| {
                    lock_open_note(f.editor.clone(), window, cx)
                })
                .unwrap()
                .expect("asks for the password");
            f.submit(cx, &form, &["not it at all"]);
            assert_eq!(f.view(cx), LockView::Plain);
            f.submit(cx, &form, &["correct horse"]);
            assert_eq!(f.view(cx), LockView::Open);
            assert_eq!(
                f.stored_body("correct horse").as_deref(),
                Some("The secret plan.")
            );
        }

        #[gpui_kit::test]
        fn a_locked_notes_copy_is_locked_for_its_own_id(cx: &mut TestAppContext) {
            let f = setup(cx);
            let form = f
                .window
                .update(cx, |_, window, cx| {
                    lock_open_note(f.editor.clone(), window, cx)
                })
                .unwrap()
                .unwrap();
            f.submit(cx, &form, &["correct horse", "correct horse"]);
            let list = cx.read(|cx| f.workspace.read(cx).note_list.clone());
            let id = f.note.clone();
            list.update(cx, |_, cx| {
                cx.emit(crate::note_list::NoteListEvent::Command(
                    id,
                    crate::note_menu::NoteCommand::Duplicate,
                ))
            });
            cx.run_until_parked();
            let copy = f
                .store
                .locked_notes()
                .unwrap()
                .into_iter()
                .find(|note| note.id != f.note)
                .expect("the copy is locked");
            assert_eq!(copy.body, "");
            let envelope = Envelope::from_extra(&copy.extra).unwrap().unwrap();
            let key =
                LockKey::for_envelope(&LockPassword::new("correct horse").unwrap(), &envelope)
                    .unwrap();
            assert_eq!(
                key.open(&copy.id, &envelope).unwrap().as_str(),
                "The secret plan."
            );
        }

        #[gpui_kit::test]
        fn opening_doesnt_keep_the_notes_unlocked_but_saving_does(cx: &mut TestAppContext) {
            let f = setup(cx);
            let unlocked_at = |cx: &mut TestAppContext| {
                cx.read(|cx| cx.global::<LockKeys>().0.last_used())
                    .expect("unlocked")
            };
            let mark = |cx: &mut TestAppContext| cx.read(|cx| note_mark(cx).1);
            assert_eq!(mark(cx), "Locked");
            let form = f
                .window
                .update(cx, |_, window, cx| {
                    lock_open_note(f.editor.clone(), window, cx)
                })
                .unwrap()
                .unwrap();
            f.submit(cx, &form, &["correct horse", "correct horse"]);
            assert_eq!(mark(cx), "Unlocked");
            let at = unlocked_at(cx);
            std::thread::sleep(Duration::from_millis(2));

            // Opening the locked note again leaves the clock alone.
            let mut note = f.store.note(&f.note).unwrap().unwrap();
            assert_eq!(cx.update(|cx| reveal(&mut note, cx)), LockView::Open);
            assert_eq!(unlocked_at(cx), at);
            cx.read(|cx| {
                assert!(LockKeys::idle(cx, at + AUTO_LOCK));
                assert!(!LockKeys::idle(cx, at + AUTO_LOCK / 2));
            });

            // Saving it does, so the notes don't close while you write.
            cx.update(|cx| seal(&mut note, cx)).unwrap();
            assert!(unlocked_at(cx) > at);

            f.window
                .update(cx, |_, window, cx| lock_now(&f.editor, window, cx))
                .unwrap();
            assert_eq!(mark(cx), "Locked");
        }

        #[gpui_kit::test]
        fn the_note_menu_locks_and_removes_the_lock(cx: &mut TestAppContext) {
            let f = setup(cx);
            let list = cx.read(|cx| f.workspace.read(cx).note_list.clone());
            let menu = |command, cx: &mut TestAppContext| {
                let id = f.note.clone();
                list.update(cx, |_, cx| {
                    cx.emit(crate::note_list::NoteListEvent::Command(id, command))
                });
                cx.run_until_parked();
            };

            // Nothing is locked yet, so Lock Note asks for a new password.
            menu(crate::note_menu::NoteCommand::Lock, cx);
            assert!(f.dialog_open(cx), "Lock Note asks for a password");
            f.window
                .update(cx, |_, window, cx| window.close_dialog(cx))
                .unwrap();
            assert_eq!(f.view(cx), LockView::Plain);

            let form = f
                .window
                .update(cx, |_, window, cx| {
                    lock_open_note(f.editor.clone(), window, cx)
                })
                .unwrap()
                .unwrap();
            f.submit(cx, &form, &["correct horse", "correct horse"]);
            assert_eq!(f.view(cx), LockView::Open);

            // Remove Lock asks before keeping the text as plain text.
            menu(crate::note_menu::NoteCommand::RemoveLock, cx);
            assert!(f.dialog_open(cx), "Remove Lock asks first");
            assert!(f.store.note(&f.note).unwrap().unwrap().is_locked());
        }

        fn touch_id_setting(cx: &mut TestAppContext) -> Option<bool> {
            cx.read(|cx| cx.global::<AppState>().config.locked.touch_id)
        }

        #[gpui_kit::test]
        fn touch_id_opens_locked_notes_once_turned_on_and_the_password_stays(
            cx: &mut TestAppContext,
        ) {
            let touch = crate::touch_id::FakeTouchId::new(true);
            let f = setup_with(cx, Some(touch.clone()));
            let form = f
                .window
                .update(cx, |_, window, cx| {
                    lock_open_note(f.editor.clone(), window, cx)
                })
                .unwrap()
                .unwrap();
            f.submit(cx, &form, &["correct horse", "correct horse"]);
            // Offered once, and nothing kept until it is turned on.
            assert_eq!(touch_id_setting(cx), Some(false));
            assert!(!f.kept_keys(cx));

            f.window
                .update(cx, |_, window, cx| {
                    super::super::turn_on_touch_id(window, cx)
                })
                .unwrap();
            cx.run_until_parked();
            assert_eq!(touch_id_setting(cx), Some(true));
            assert!(f.kept_keys(cx));
            assert_eq!(touch.asked(), 1);

            // Closed again, a fingerprint opens it without the password.
            f.window
                .update(cx, |_, window, cx| lock_now(&f.editor, window, cx))
                .unwrap();
            cx.run_until_parked();
            assert_eq!(f.view(cx), LockView::Closed(None));
            let form = f
                .window
                .update(cx, |_, window, cx| {
                    unlock_open_note(f.editor.clone(), window, cx)
                })
                .unwrap();
            assert!(form.is_none(), "Touch ID asks, not a form");
            cx.run_until_parked();
            assert_eq!(f.view(cx), LockView::Open);
            assert_eq!(f.text(cx), "The secret plan.");
            assert!(!f.dialog_open(cx));

            // A cancelled fingerprint falls back to the password.
            f.window
                .update(cx, |_, window, cx| lock_now(&f.editor, window, cx))
                .unwrap();
            touch.answer(false);
            f.window
                .update(cx, |_, window, cx| {
                    unlock_open_note(f.editor.clone(), window, cx)
                })
                .unwrap();
            cx.run_until_parked();
            assert_eq!(f.view(cx), LockView::Closed(None));
            assert!(f.dialog_open(cx), "the password form shows");
            f.window
                .update(cx, |_, window, cx| window.close_dialog(cx))
                .unwrap();

            // Turned off, the keys are forgotten and the password is asked.
            f.window
                .update(cx, |_, window, cx| {
                    super::super::turn_off_touch_id(window, cx)
                })
                .unwrap();
            cx.run_until_parked();
            assert_eq!(touch_id_setting(cx), Some(false));
            assert!(!f.kept_keys(cx));
            let form = f
                .window
                .update(cx, |_, window, cx| {
                    unlock_open_note(f.editor.clone(), window, cx)
                })
                .unwrap();
            assert!(form.is_some());
        }
    }
}
