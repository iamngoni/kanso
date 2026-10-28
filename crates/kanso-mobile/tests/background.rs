//! The sync with no app: signed out, signed in and pulling, out of time, and
//! the secret store refusing. The same numbers the C entry returns.

mod common;

use std::sync::Arc;
use std::time::Duration;

use common::*;
use kanso_mobile::background::{BackgroundOutcome, run};
use kanso_sync::MemoryServer;

fn backend(server: &Arc<MemoryServer>) -> Arc<dyn kanso_sync::Backend> {
    server.clone()
}

// no-prop-test: the contract's numbers, each reached once.
#[test]
fn a_background_sync_signed_out_pulling_out_of_time_and_without_secrets() {
    let server = Arc::new(MemoryServer::new());
    make_account(&server);

    // Signed out: nothing to do.
    let empty = lone_phone();
    let path = empty.path();
    let secrets = empty.secrets.clone();
    drop(empty.library);
    let outcome = run(&path, Duration::from_secs(5), backend(&server), secrets);
    assert_eq!(outcome, BackgroundOutcome::NotSignedIn);
    assert_eq!(outcome.code(), -1);

    // Another phone writes a note; this one pulls it with no app running.
    let writer = phone(&server);
    sign_in(&writer, "Writer");
    let id = note(&writer.library, "From elsewhere", "text");
    writer.library.sync().sync_now().unwrap();

    let reader = phone(&server);
    sign_in(&reader, "Reader");
    let (path, secrets) = (reader.path(), reader.secrets.clone());
    // The app is not running: its library is closed. (The folder stays.)
    let keep = reader.dir;
    drop(reader.library);
    let outcome = run(
        &path,
        Duration::from_secs(30),
        backend(&server),
        secrets.clone(),
    );
    assert_eq!(outcome, BackgroundOutcome::Done { changed: true });
    assert_eq!(outcome.code(), 1);
    // Nothing new the second time.
    let outcome = run(
        &path,
        Duration::from_secs(30),
        backend(&server),
        secrets.clone(),
    );
    assert_eq!(outcome.code(), 0);

    // No time at all.
    let outcome = run(&path, Duration::ZERO, backend(&server), secrets.clone());
    assert_eq!(outcome, BackgroundOutcome::OutOfTime);
    assert_eq!(outcome.code(), -4);

    // The secret store refuses.
    let refusing = Arc::new(kanso_sync::MemorySecrets::new());
    refusing.fail_reads(true);
    let outcome = run(&path, Duration::from_secs(5), backend(&server), refusing);
    // The library is not open, so it is opened with these secrets.
    assert_eq!(outcome.code(), -2);

    // The note arrived, and is there when the app opens the library again.
    let library = kanso_mobile::Library::open_for_test(&path, secrets, backend(&server)).unwrap();
    assert_eq!(library.note(&id).unwrap().title, "From elsewhere");
    drop(keep);
}

// no-prop-test: the app has the library open; the background reuses it.
#[test]
fn a_background_sync_reuses_the_library_the_app_has_open() {
    let server = Arc::new(MemoryServer::new());
    make_account(&server);
    let phone = phone(&server);
    sign_in(&phone, "Phone");
    note(&phone.library, "Local", "text");
    // The background run, from another thread of the same process, pushes the
    // app's note through the app's own library.
    let (path, secrets, other) = (phone.path(), phone.secrets.clone(), backend(&server));
    let outcome = std::thread::spawn(move || run(&path, Duration::from_secs(30), other, secrets))
        .join()
        .unwrap();
    assert_eq!(outcome, BackgroundOutcome::Done { changed: true });
    // It left the app's library with nothing pending.
    let status = phone.library.sync().status().unwrap();
    assert_eq!(status.phase, kanso_mobile::SyncPhase::Synced);
}

// no-prop-test: a library path that can't be opened.
#[test]
fn a_library_that_cannot_be_opened_is_internal() {
    let dir = tempfile::tempdir().unwrap();
    // A folder where the database file should be.
    let path = dir.path().join("library.db");
    std::fs::create_dir(&path).unwrap();
    let outcome = run(
        &path,
        Duration::from_secs(1),
        backend(&Arc::new(MemoryServer::new())),
        Arc::new(kanso_sync::MemorySecrets::new()),
    );
    assert_eq!(outcome, BackgroundOutcome::Internal);
    assert_eq!(outcome.code(), -5);
}

// no-prop-test: one setting, on and off (the schedule's own property covers
// the foreground).
#[test]
fn a_background_sync_with_sync_automatically_off_pulls_nothing() {
    let server = Arc::new(MemoryServer::new());
    make_account(&server);
    let writer = phone(&server);
    sign_in(&writer, "Writer");
    let id = note(&writer.library, "From elsewhere", "text");
    writer.library.sync().sync_now().unwrap();

    let reader = phone(&server);
    sign_in(&reader, "Reader");
    let off = reader.library.sync().settings().unwrap();
    reader
        .library
        .sync()
        .set_settings(false, off.interval_secs)
        .unwrap();
    let (path, secrets) = (reader.path(), reader.secrets.clone());
    let keep = reader.dir;
    drop(reader.library);

    // Nothing to do, and nothing pulled.
    let outcome = run(
        &path,
        Duration::from_secs(30),
        backend(&server),
        secrets.clone(),
    );
    assert_eq!(outcome, BackgroundOutcome::Done { changed: false });
    let library =
        kanso_mobile::Library::open_for_test(&path, secrets.clone(), backend(&server)).unwrap();
    assert!(library.note(&id).is_err());

    // Turned on again, the next run pulls the note.
    let on = library.sync().settings().unwrap();
    library.sync().set_settings(true, on.interval_secs).unwrap();
    drop(library);
    let outcome = run(&path, Duration::from_secs(30), backend(&server), secrets);
    assert_eq!(outcome, BackgroundOutcome::Done { changed: true });
    drop(keep);
}
