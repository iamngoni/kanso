//! A restored backup meets the account again (issue #243 step 11, PR #301):
//! a library that had synced is signed out, a backup is restored into it,
//! and it signs in and syncs. Nothing on the server or another Mac may be
//! deleted or overwritten without a copy, and a merge that can bring back
//! notes deleted elsewhere only happens after the question that says so
//! (docs/ACCOUNTS.md › Library binding, plan review 10).

mod common;

use std::path::{Path, PathBuf};

use common::*;
use kanso_store::{
    KEY_BYTES, LibraryKey, Origin, Store, apply_staged_restore, restore_staged, stage_restore,
};
use kanso_sync::{
    Choice, Flow, LocalLibrary, MemoryServer, Plan, Session, SyncOptions, local_library, plan,
};

fn key_from(seed: u8) -> LibraryKey {
    let mut bytes = [seed; KEY_BYTES];
    bytes[0] = seed.wrapping_mul(13).wrapping_add(3);
    LibraryKey::from_bytes(bytes)
}

/// An encrypted library, as a desktop holds one.
struct Library {
    dir: tempfile::TempDir,
    key: LibraryKey,
}

impl Library {
    fn new(seed: u8) -> Library {
        let library = Library {
            dir: tempfile::tempdir().unwrap(),
            key: key_from(seed),
        };
        drop(library.open());
        library
    }

    fn db(&self) -> PathBuf {
        self.dir.path().join("kanso.db")
    }

    fn backups(&self) -> PathBuf {
        self.dir.path().join("backups")
    }

    fn open(&self) -> Store {
        Store::open_with_key(self.db(), Some(&self.key)).unwrap()
    }

    fn sync(&self, server: &MemoryServer, session: &Session) -> kanso_sync::SyncReport {
        kanso_sync::sync(&self.open(), server, session, &SyncOptions::default()).expect("sync")
    }

    fn backup(&self, at: i64) -> PathBuf {
        self.open().backup_now(&self.backups(), at).unwrap()
    }

    /// Restore from backup…, then the next launch applies it.
    fn restore(&self, backup: &Path, backup_key: &LibraryKey) {
        stage_restore(&self.db(), backup, Some(backup_key), &self.key).unwrap();
        assert!(apply_staged_restore(&self.db(), &self.key, &self.backups(), 9_000).unwrap());
        assert!(!restore_staged(&self.db()));
    }
}

/// What a sign-in does with the library: the plan the app follows.
fn sign_in_plan(store: &Store, session: &Session) -> Plan {
    plan(
        Flow::SignIn,
        &local_library(store, &session.origin, &session.account_id).unwrap(),
    )
}

/// Live documents on a Mac: id -> body (notes) or rev.
fn live(store: &Store) -> std::collections::BTreeMap<String, String> {
    live_docs(store)
        .into_iter()
        .map(|(id, (rev, fields))| {
            let body = fields
                .get("body")
                .and_then(|body| body.as_str())
                .map_or(rev, str::to_owned);
            (id, body)
        })
        .collect()
}

/// Live documents of the account on the server: id -> body (notes) or rev.
fn live_on_server(
    server: &MemoryServer,
    session: &Session,
) -> std::collections::BTreeMap<String, String> {
    server_live(server, session)
        .into_iter()
        .map(|(id, (rev, fields))| {
            let body = fields
                .get("body")
                .and_then(|body| body.as_str())
                .map_or(rev, str::to_owned);
            (id, body)
        })
        .collect()
}

/// Two Macs on one account, a backup of the first, then the second deletes
/// one note, edits another and writes a third. Mac 1 syncs all that, signs
/// out (the binding stays, docs/ACCOUNTS.md › Signing out), and restores the
/// backup. Gives back the server, Mac 1, and Mac 2 with its session.
fn restored_after_changes_elsewhere() -> (MemoryServer, Library, Library, Session) {
    let server = MemoryServer::new();
    let one = sign_up(&server, "ngoni", 1);
    let mac1 = Library::new(1);
    kanso_sync::bind(&mac1.open(), &one, Choice::Add, &mac1.backups(), 0).unwrap();
    {
        let store = mac1.open();
        book(&store, "book:b1", "Inbox");
        note(&store, "note:gone", "book:b1", "deleted later on Mac 2", 100);
        note(&store, "note:edited", "book:b1", "before", 100);
        note(&store, "note:kept", "book:b1", "untouched", 100);
    }
    mac1.sync(&server, &one);

    let two = sign_in(&server, "ngoni", 2);
    let mac2 = Library::new(2);
    kanso_sync::bind(&mac2.open(), &two, Choice::Add, &mac2.backups(), 0).unwrap();
    mac2.sync(&server, &two);

    let backup = mac1.backup(1_000);

    {
        let store = mac2.open();
        store.delete("note:gone", Origin::Api).unwrap();
        edit_note(&store, "note:edited", "after, on Mac 2", Some(500));
        note(&store, "note:new", "book:b1", "written on Mac 2", 500);
    }
    mac2.sync(&server, &two);
    mac1.sync(&server, &one);
    assert!(!live(&mac1.open()).contains_key("note:gone"));

    // Sign out: the session goes, the library's binding stays.
    drop(one);
    let key = mac1.key.clone();
    mac1.restore(&backup, &key);
    (server, mac1, mac2, two)
}

/// Regression (PR #301 review): the restore kept the library's own account
/// binding, so the next sign-in was `Same` and resumed with no question. The
/// first sync then merged silently: the note Mac 2 deleted after the backup
/// came back on the server and on Mac 2. The binding goes with the restore,
/// so signing in asks first, as for any library with notes.
#[test]
fn signing_in_after_a_restore_asks_before_merging() {
    let (server, mac1, _mac2, _two) = restored_after_changes_elsewhere();
    let one = sign_in(&server, "ngoni", 1);
    let asked = sign_in_plan(&mac1.open(), &one);
    assert!(
        matches!(asked, Plan::Ask { from: None, .. }),
        "a restored library is asked about, not resumed: {asked:?}"
    );
}

/// Add them: the merge the question describes. Nothing the account holds is
/// deleted or overwritten: Mac 2's edit stays the account's version, its new
/// note stays, and only the note deleted after the backup comes back, as the
/// Add choice says it may.
#[test]
fn adding_a_restored_library_loses_nothing_on_the_account() {
    let (server, mac1, mac2, two) = restored_after_changes_elsewhere();
    let before = live_on_server(&server, &two);
    let one = sign_in(&server, "ngoni", 1);
    assert!(matches!(
        sign_in_plan(&mac1.open(), &one),
        Plan::Ask { .. }
    ));
    kanso_sync::bind(&mac1.open(), &one, Choice::Add, &mac1.backups(), 9_500).unwrap();
    mac1.sync(&server, &one);
    mac2.sync(&server, &two);

    let after = live_on_server(&server, &two);
    for (id, body) in &before {
        assert_eq!(after.get(id), Some(body), "{id} kept on the server as it was");
    }
    assert_eq!(after.get("note:gone").map(String::as_str), Some("deleted later on Mac 2"));
    assert_eq!(live(&mac2.open()), after, "Mac 2 holds the account's notes");
    assert_eq!(live(&mac1.open()), after, "Mac 1 holds the account's notes");
    // The restored version of the edited note isn't lost either: it is in
    // the note's history on Mac 1.
    let store = mac1.open();
    let history = store.revisions("note:edited").unwrap();
    assert!(!history.is_empty(), "the restored version is kept as history");
}

/// Replace them: the account's notes, as they are, after a backup of the
/// restored library. The server and Mac 2 don't change at all.
#[test]
fn replacing_a_restored_library_changes_nothing_on_the_account() {
    let (server, mac1, mac2, two) = restored_after_changes_elsewhere();
    let server_before = server_docs(&server, &two);
    let mac2_before = all_docs(&mac2.open());
    let one = sign_in(&server, "ngoni", 1);
    let saved = kanso_sync::bind(&mac1.open(), &one, Choice::Replace, &mac1.backups(), 9_500)
        .unwrap()
        .expect("the restored library is backed up first");
    assert!(saved.exists());
    mac1.sync(&server, &one);
    mac2.sync(&server, &two);
    assert_eq!(server_docs(&server, &two), server_before);
    assert_eq!(all_docs(&mac2.open()), mac2_before);
    assert_eq!(live(&mac1.open()), live_on_server(&server, &two));
}

/// A backup from another account, restored into a library that synced with
/// this one: signing in asks (the notes aren't this account's), and adding
/// them leaves both accounts' own notes as they were.
#[test]
fn a_backup_from_another_account_is_asked_about_and_loses_nothing() {
    let server = MemoryServer::new();
    // alex's library on another computer.
    let alex = sign_up(&server, "alex", 3);
    let theirs = Library::new(3);
    kanso_sync::bind(&theirs.open(), &alex, Choice::Add, &theirs.backups(), 0).unwrap();
    {
        let store = theirs.open();
        book(&store, "book:a1", "Alex's");
        note(&store, "note:alex", "book:a1", "alex's note", 100);
    }
    theirs.sync(&server, &alex);
    let backup = theirs.backup(1_000);
    let alex_before = server_docs(&server, &alex);

    // ngoni's library here, synced, then signed out.
    let one = sign_up(&server, "ngoni", 1);
    let mac1 = Library::new(1);
    kanso_sync::bind(&mac1.open(), &one, Choice::Add, &mac1.backups(), 0).unwrap();
    {
        let store = mac1.open();
        book(&store, "book:b1", "Inbox");
        note(&store, "note:mine", "book:b1", "ngoni's note", 100);
    }
    mac1.sync(&server, &one);
    let ngoni_before = live_on_server(&server, &one);
    drop(one);

    mac1.restore(&backup, &theirs.key);
    let one = sign_in(&server, "ngoni", 1);
    let asked = sign_in_plan(&mac1.open(), &one);
    assert!(
        matches!(asked, Plan::Ask { .. }),
        "alex's notes are asked about before they join ngoni's account: {asked:?}"
    );
    assert_ne!(
        local_library(&mac1.open(), &one.origin, &one.account_id).unwrap(),
        LocalLibrary::Same
    );
    kanso_sync::bind(&mac1.open(), &one, Choice::Add, &mac1.backups(), 9_500).unwrap();
    mac1.sync(&server, &one);

    let ngoni_after = live_on_server(&server, &one);
    for (id, body) in &ngoni_before {
        assert_eq!(ngoni_after.get(id), Some(body), "{id} kept in ngoni's account");
    }
    assert!(ngoni_after.contains_key("note:alex"), "added, as asked");
    assert_eq!(server_docs(&server, &alex), alex_before, "alex's account untouched");
    assert_eq!(live(&mac1.open()), ngoni_after);
}
