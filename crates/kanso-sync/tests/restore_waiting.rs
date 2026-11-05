//! `kanso-sync` (this package's binary) leaves a staged restore alone: only
//! the app applies one (issue #243 step 11, PR #301). It says so and works
//! on the library as it is.

// The binary needs the desktop's secret stores; the phones' build leaves
// them, and the binary, out.
#![cfg(all(feature = "system-keychain", feature = "file-secrets"))]

use std::path::Path;
use std::process::Command;

use kanso_sync::{FileSecrets, SECRETS_FILE, SECRETS_KEY_FILE_ENV};

const WAITING: &str = "A restore is waiting; open Kanso to apply it.";

/// The library in `dir`, opened with the secrets file its config names.
fn open(dir: &Path) -> kanso_store::Store {
    let secrets = FileSecrets::open(dir.join(SECRETS_FILE), None).unwrap();
    kanso_sync::open_library(&dir.join("kanso.db"), &secrets)
        .unwrap()
        .store
}

#[test]
fn kanso_sync_leaves_a_staged_restore_alone() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("config.json"),
        r#"{"sync": {"secretStore": "file"}}"#,
    )
    .unwrap();
    let db = dir.path().join("kanso.db");
    {
        let store = open(dir.path());
        let backup = store
            .backup_now(&dir.path().join("backups"), 1_000)
            .unwrap();
        // Written after the backup: applying the restore would drop it.
        store.set_meta("later", "yes").unwrap();
        let key = store.library_key().unwrap().clone();
        drop(store);
        kanso_store::stage_restore(&db, &backup, Some(&key), &key).unwrap();
    }

    let output = Command::new(env!("CARGO_BIN_EXE_kanso-sync"))
        .args(["status", "--data-dir"])
        .arg(dir.path())
        .env_remove(SECRETS_KEY_FILE_ENV)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    assert!(stderr.contains(WAITING), "{stderr}");

    assert!(kanso_store::restore_staged(&db), "the restore still waits");
    assert_eq!(
        open(dir.path()).meta("later").unwrap().as_deref(),
        Some("yes"),
        "the library is the one it was"
    );
}
