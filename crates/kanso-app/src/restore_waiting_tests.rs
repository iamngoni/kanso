//! A staged restore is applied by the app alone (issue #243 step 11 follow-up):
//! `kanso capture`, `serve`, `sticky`, `plugin`, the importers and the account
//! commands open the current library as it is and leave the restore for the
//! app, which can tell the person in a toast.

use std::path::Path;

use crate::library_open;
use crate::paths;

/// A library with the note "later" written after a backup of it, and a
/// restore of that backup staged: applying it would drop "later".
fn staged_library(dir: &Path) {
    let (store, _) =
        library_open::open_store_reporting(dir, library_open::test_secrets(), &|_| {}).unwrap();
    let backup = store.backup_now(&paths::backups_dir(dir), 1_000).unwrap();
    store.set_meta("later", "yes").unwrap();
    let key = store.library_key().unwrap().clone();
    drop(store);
    kanso_store::stage_restore(&paths::db_path(dir), &backup, Some(&key), &key).unwrap();
}

/// The staged restore still waits and the library is the current one.
fn assert_left_alone(dir: &Path) {
    assert!(
        kanso_store::restore_staged(&paths::db_path(dir)),
        "the restore is still waiting"
    );
    let store = library_open::open_store(dir).unwrap();
    assert_eq!(store.meta("later").unwrap().as_deref(), Some("yes"));
    assert!(
        kanso_store::restore_staged(&paths::db_path(dir)),
        "opening again changed nothing"
    );
}

#[test]
fn open_store_leaves_a_staged_restore_alone() {
    let dir = tempfile::tempdir().unwrap();
    staged_library(dir.path());
    drop(library_open::open_store(dir.path()).unwrap());
    assert_left_alone(dir.path());
}

#[test]
fn the_account_commands_leave_a_staged_restore_alone() {
    let dir = tempfile::tempdir().unwrap();
    staged_library(dir.path());
    drop(library_open::open_store_with(dir.path(), library_open::test_secrets()).unwrap());
    assert_left_alone(dir.path());
}

#[test]
fn capture_leaves_a_staged_restore_alone() {
    let dir = tempfile::tempdir().unwrap();
    staged_library(dir.path());
    crate::capture_cli::run_cli(dir.path(), &["a thought".into()]).unwrap();
    assert_left_alone(dir.path());
}

#[test]
fn sticky_leaves_a_staged_restore_alone() {
    let dir = tempfile::tempdir().unwrap();
    staged_library(dir.path());
    crate::sticky_cli::run_cli(dir.path(), &["a sticky".into()]).unwrap();
    assert_left_alone(dir.path());
}

#[test]
fn the_plugin_commands_leave_a_staged_restore_alone() {
    let dir = tempfile::tempdir().unwrap();
    staged_library(dir.path());
    crate::plugins::run_cli(dir.path(), &["list".into()]).unwrap();
    assert_left_alone(dir.path());
}

#[test]
fn the_importers_leave_a_staged_restore_alone() {
    let dir = tempfile::tempdir().unwrap();
    staged_library(dir.path());
    // An empty folder to read: the import finds nothing, but it opens the
    // library first, and that is the step under test.
    let source = tempfile::tempdir().unwrap();
    let args = [
        "apple-notes".to_owned(),
        source.path().display().to_string(),
    ];
    let _ = crate::library_import::run_cli(dir.path(), &args);
    assert_left_alone(dir.path());
}

/// `kanso serve`, and anything else that opens the library through the
/// app's own call, would apply a restore silently. Only `main.rs` may.
#[test]
fn only_the_app_opens_the_library_applying_a_restore() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut found = Vec::new();
    for entry in std::fs::read_dir(&src).unwrap().flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if matches!(
            name.as_str(),
            "library_open.rs" | "restore_waiting_tests.rs"
        ) {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        if text.contains("open_store_progress") || text.contains("open_store_applying_restore") {
            found.push(name);
        }
    }
    assert_eq!(found, ["main.rs"]);
}

/// The line the other programs print.
#[test]
fn the_waiting_line_says_to_open_kanso() {
    assert_eq!(
        library_open::RESTORE_WAITING,
        "A restore is waiting; open Kanso to apply it."
    );
}

/// The app's own open still applies it, and says so for its toast.
#[test]
fn the_app_still_applies_it() {
    let dir = tempfile::tempdir().unwrap();
    staged_library(dir.path());
    let (store, _) = library_open::open_store_progress(dir.path(), &|_| {}).unwrap();
    assert_eq!(store.meta("later").unwrap(), None);
    drop(store);
    assert!(!kanso_store::restore_staged(&paths::db_path(dir.path())));
    assert!(matches!(library_open::take_restore_outcome(), Some(Ok(()))));
}
