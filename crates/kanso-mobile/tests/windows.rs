//! A note in two windows: each window is its own Flutter engine, and every
//! engine opens the library by its path. They must get the one library, so
//! that the lock session is shared and what is typed in one window shows in
//! the other, saved by the one typed in.

mod common;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use common::*;
use kanso_mobile::{
    EditorEvent, EditorSession, Library, LockEvent, MemorySecrets, Range16, SaveState, TextChange,
};
use kanso_sync::MemoryServer;

/// The library again, as a second engine opens it: the same path, spelled
/// another way, with a secret store and server of its own that are not used.
fn second_handle(phone: &Phone) -> Library {
    let again = phone.dir.path().join("./library.db");
    Library::open_for_test(
        &again,
        Arc::new(MemorySecrets::new()),
        Arc::new(MemoryServer::new()),
    )
    .unwrap()
}

fn listen(session: &EditorSession) -> Arc<Mutex<Vec<EditorEvent>>> {
    let heard = Arc::new(Mutex::new(Vec::new()));
    let log = heard.clone();
    session.events(move |event| {
        log.lock().unwrap().push(event);
        true
    });
    heard
}

fn insert(session: &EditorSession, at: u32, text: &str) {
    let version = session.snapshot().unwrap().version;
    let after = at + text.encode_utf16().count() as u32;
    session
        .apply(TextChange {
            base_version: version,
            start: at,
            end: at,
            text: text.to_owned(),
            selection: Range16 {
                start: after,
                end: after,
            },
            composing: None,
        })
        .unwrap();
}

// no-prop-test: the identity of the library and what hangs on it, in a fixed
// sequence of two handles.
#[test]
fn two_handles_on_one_path_share_the_lock_session_and_see_each_others_edits() {
    let phone = lone_phone();
    let first = &phone.library;
    let second = second_handle(&phone);
    let id = note(first, "Plan", "the plan");
    // The lock session is one: unlocking through one handle opens it for
    // the other, and the other's listener hears it.
    first.locks().lock_note(&id, LOCK_PASSWORD.into()).unwrap();
    first.locks().lock().unwrap();
    assert!(!second.locks().is_unlocked().unwrap());
    let (tx, rx) = std::sync::mpsc::channel();
    second.locks().lock_events(move |event| {
        tx.send(event).ok();
        true
    });
    assert!(first.locks().unlock(LOCK_PASSWORD.into()).unwrap());
    assert!(second.locks().is_unlocked().unwrap());
    assert_eq!(
        rx.recv_timeout(Duration::from_secs(5)).unwrap(),
        LockEvent::Unlocked
    );
    second.locks().lock().unwrap();
    assert!(!first.locks().is_unlocked().unwrap());
    // A write through one handle is read through the other.
    let plain = note(&second, "Plain", "first words");
    assert_eq!(
        first.note(&plain).unwrap().body.as_deref(),
        Some("first words")
    );
}

// no-prop-test: two sessions and the order of a few edits.
#[test]
fn typing_in_one_window_shows_in_the_other_at_once_and_only_one_saves() {
    let phone = lone_phone();
    let first = &phone.library;
    let second = second_handle(&phone);
    let id = note(first, "Plan", "hello world");
    let in_main = first.edit(&id).unwrap();
    let in_window = second.edit(&id).unwrap();
    let main_events = listen(&in_main);
    let window_events = listen(&in_window);

    // Typed in the note window, shown in the main window at once.
    in_main
        .select(Range16 { start: 11, end: 11 }, false)
        .unwrap();
    insert(&in_window, 0, "Big ");
    assert_eq!(in_main.snapshot().unwrap().text, "Big hello world");
    assert_eq!(in_window.snapshot().unwrap().text, "Big hello world");
    assert!(main_events.lock().unwrap().iter().any(|event| matches!(
        event,
        EditorEvent::Reloaded { snapshot } if snapshot.text == "Big hello world"
    )));
    // Typing back from the main window shows in the note window, too.
    insert(&in_main, 15, "!");
    assert_eq!(in_window.snapshot().unwrap().text, "Big hello world!");

    // Neither is saved yet. The one typed in last saves; the other takes the
    // save without asking which version to keep.
    assert_eq!(
        first.note(&id).unwrap().body.as_deref(),
        Some("hello world")
    );
    in_main.tick(Instant::now() + Duration::from_secs(2));
    assert_eq!(
        first.note(&id).unwrap().body.as_deref(),
        Some("Big hello world!")
    );
    let end = Instant::now() + Duration::from_secs(10);
    while !matches!(in_window.snapshot().unwrap().save, SaveState::Saved { .. }) {
        assert!(Instant::now() < end, "the note window never saw the save");
        std::thread::sleep(Duration::from_millis(5));
    }
    let heard = window_events.lock().unwrap();
    assert!(
        !heard
            .iter()
            .any(|event| matches!(event, EditorEvent::Incoming { .. })),
        "a peer's save is not a change made elsewhere: {heard:?}"
    );
    assert!(
        heard
            .iter()
            .any(|event| matches!(event, EditorEvent::Saved { .. }))
    );
    drop(heard);
    // The window that took the save has nothing left to write.
    in_window.tick(Instant::now() + Duration::from_secs(2));
    assert_eq!(
        first.note(&id).unwrap().body.as_deref(),
        Some("Big hello world!")
    );
}

// no-prop-test: the race it is about, in a fixed order.
#[test]
fn typing_that_goes_on_in_the_other_window_while_a_save_arrives_is_no_conflict() {
    let phone = lone_phone();
    let first = &phone.library;
    let second = second_handle(&phone);
    let id = note(first, "Plan", "abc");
    let in_main = first.edit(&id).unwrap();
    let in_window = second.edit(&id).unwrap();
    insert(&in_main, 3, "d");
    // The save of "abcd" is written; the other window has typed on, and
    // been heard by the first, before the save reaches it.
    in_main.flush().unwrap();
    insert(&in_window, 4, "e");
    assert_eq!(in_main.snapshot().unwrap().text, "abcde");
    in_window.flush().unwrap();
    in_main.flush().unwrap();
    assert_eq!(first.note(&id).unwrap().body.as_deref(), Some("abcde"));
    for session in [&in_main, &in_window] {
        assert!(matches!(
            session.snapshot().unwrap().save,
            SaveState::Saved { .. }
        ));
    }
}
