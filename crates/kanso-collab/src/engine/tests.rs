//! The engine against the in-memory server: accounts with devices, each
//! with its own library, sharing notes, editing them through the library and
//! through an open editor, and syncing in whatever order. Property tests run
//! on the seed `PROPTEST_RNG_SEED` pins.

use std::sync::Arc;
use std::time::{Duration, Instant};

use kanso_core::id::DocType;
use kanso_store::{Origin, Put, Store};
use proptest::prelude::*;
use serde_json::json;
use tempfile::TempDir;
use zeroize::Zeroizing;

use super::{Account, Engine, lock, rebase};
use crate::api::{Caller, CollabApi};
use crate::doc::{Field, apply_splices};
use crate::envelope;
use crate::memory::MemoryCollab;
use crate::model::{Kind, LiveNotice, Placement, Role};

const SERVER: &str = "https://sync.kanso.test";

struct Device {
    engine: Arc<Engine>,
    store: Arc<Store>,
    _dir: TempDir,
}

fn hex_id(prefix: &str, n: u8) -> String {
    format!("{prefix}_{}", format!("{n:02x}").repeat(16))
}

/// Device `device` of account `n` (username `user<n>`), with its own library.
fn device(server: &MemoryCollab, n: u8, device: u8) -> Device {
    let dir = TempDir::new().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let account = Account {
        caller: Caller {
            server: SERVER.into(),
            account_id: hex_id("a", n),
            username: format!("user{n}"),
            device_id: hex_id("d", n * 16 + device),
            authorization: Zeroizing::new("Bearer test".into()),
        },
        roots: kanso_sync::CollabRoots {
            identity_key: Zeroizing::new([n; 32]),
            vault_key: Zeroizing::new([n.wrapping_add(100); 32]),
        },
    };
    let engine = Arc::new(Engine::new(
        store.clone(),
        Arc::new(server.clone()),
        account,
    ));
    engine.identity().unwrap();
    Device {
        engine,
        store,
        _dir: dir,
    }
}

fn book(store: &Store) -> String {
    store
        .put(
            DocType::Book,
            json!({"name": "Notes"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

fn note(store: &Store, id: &str, title: &str, body: &str) -> String {
    let book = book(store);
    store
        .put(
            DocType::Note,
            json!({"_id": id, "bookId": book, "title": title, "body": body}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

/// Writes a note's body as the app's other writers do (Origin::Ui).
fn write_body(store: &Store, id: &str, body: &str) -> kanso_store::Result<()> {
    store
        .put(
            DocType::Note,
            json!({"_id": id, "body": body}),
            Put::Merge,
            Origin::Ui,
        )
        .map(|_| ())
}

fn body_of(device: &Device, share_id: &str) -> String {
    let note_id = device.store.shared_note(share_id).unwrap().unwrap().note_id;
    device.store.note(&note_id).unwrap().unwrap().body
}

/// A shares `body` with the others at `role`, and each accepts and syncs.
fn shared(owner: &Device, others: &[(&Device, Role)], body: &str) -> String {
    let id = note(&owner.store, "note:shared", "Plan", body);
    for (other, _) in others {
        owner
            .engine
            .person(&other.engine.caller().username)
            .unwrap();
    }
    let invite: Vec<(String, Role)> = others
        .iter()
        .map(|(d, role)| (d.engine.caller().username.clone(), *role))
        .collect();
    let share = owner.engine.share(&id, &invite).unwrap();
    for (other, _) in others {
        other.engine.refresh().unwrap();
        other.engine.accept(&share).unwrap();
    }
    share
}

fn sync(devices: &[&Device], share: &str) {
    for _ in 0..2 {
        for d in devices {
            d.engine.sync_note(share).unwrap();
        }
    }
}

#[test]
fn sharing_brings_the_note_to_its_members() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared(&a, &[(&b, Role::Edit)], "Hello");
    assert_eq!(body_of(&b, &share), "Hello");
    let b_note = b.store.shared_note(&share).unwrap().unwrap();
    assert_eq!(b_note.role, "edit");
    // The note says it is shared, in both libraries.
    for d in [&a, &b] {
        let id = d.store.shared_note(&share).unwrap().unwrap().note_id;
        let note = d.store.note(&id).unwrap().unwrap();
        assert!(note.extra.contains_key(kanso_core::model::SHARE_FIELD));
    }
    // Edits go both ways.
    write_body(&b.store, &b_note.note_id, "Hello, world").unwrap();
    sync(&[&b, &a], &share);
    assert_eq!(body_of(&a, &share), "Hello, world");
    // History says who changed it.
    let a_id = a.store.shared_note(&share).unwrap().unwrap().note_id;
    assert!(
        a.store
            .shared_authors(&a_id)
            .unwrap()
            .contains(&"user2".to_owned())
    );
}

#[test]
fn the_owners_other_device_and_a_late_device_catch_up() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared(&a, &[(&b, Role::Edit)], "One");
    let b2 = device(&server, 2, 2);
    b2.engine.refresh().unwrap();
    b2.engine.sync_note(&share).unwrap();
    assert_eq!(body_of(&b2, &share), "One");
}

#[test]
fn a_locked_note_and_a_note_in_a_locked_notebook_cant_be_shared() {
    let server = MemoryCollab::new();
    let a = device(&server, 1, 1);
    let book = book(&a.store);
    a.store
        .lock_note(json!({
            "_id": "note:locked",
            "bookId": book,
            "title": "",
            "body": "",
            "kansoLock": {"v": 1, "salt": "AAAA", "nonce": "AAAA", "sealed": "AAAA"}
        }))
        .unwrap();
    let err = a.engine.check_shareable("note:locked").unwrap_err();
    assert!(err.to_string().contains("Locked"), "{err}");
    let err = a.engine.share("note:locked", &[]).unwrap_err();
    assert!(matches!(err, crate::api::CollabError::NotShareable(_)));
    // Nor can a shared note be locked afterwards.
    let b = device(&server, 2, 1);
    let share = shared(&a, &[(&b, Role::Edit)], "x");
    let id = a.store.shared_note(&share).unwrap().unwrap().note_id;
    let locked = a.store.lock_note(json!({
        "_id": id,
        "kansoLock": {"v": 1, "salt": "AAAA", "nonce": "AAAA", "sealed": "AAAA"}
    }));
    assert!(locked.is_err());
}

#[test]
fn a_viewers_writes_are_refused_by_the_library_the_server_and_the_members() {
    let server = MemoryCollab::new();
    let (a, v) = (device(&server, 1, 1), device(&server, 3, 1));
    let share = shared(&a, &[(&v, Role::View)], "Read me");
    let v_note = v.store.shared_note(&share).unwrap().unwrap();
    // The library won't change a viewer's text.
    assert!(write_body(&v.store, &v_note.note_id, "Changed").is_err());
    // The server won't take a viewer's update.
    let forged = {
        let loaded = v.engine.loaded(&share).unwrap();
        let mut guard = lock(&loaded);
        let update = guard.doc.set_text(Field::Body, "Forged").unwrap();
        let (_, sealed) = v
            .engine
            .seal(&mut guard, Kind::Update, None, &update)
            .unwrap();
        sealed
    };
    let err = server
        .append(v.engine.caller(), &share, &forged)
        .unwrap_err();
    assert_eq!(err.code(), Some("forbidden_role"));
    // And a member that is handed it anyway (by a server that took it)
    // doesn't merge it.
    {
        let loaded = a.engine.loaded(&share).unwrap();
        let mut guard = lock(&loaded);
        let next = guard.note.applied_seq + 1;
        guard.inbox.insert(next, forged);
        let merge = a.engine.merge_delivered(&mut guard).unwrap();
        assert!(!merge.text);
        assert_eq!(guard.doc.text(Field::Body), "Read me");
    }
    // Commenting needs the comment role.
    assert!(v.engine.add_comment(&share, 0..4, "Nice").is_err());
}

#[test]
fn a_removed_member_cant_read_what_is_written_after() {
    let server = MemoryCollab::new();
    let (a, b, c) = (
        device(&server, 1, 1),
        device(&server, 2, 1),
        device(&server, 4, 1),
    );
    let share = shared(&a, &[(&b, Role::Edit), (&c, Role::Edit)], "Before");
    let old_keys: Vec<_> = {
        let loaded = b.engine.loaded(&share).unwrap();
        lock(&loaded).keys.values().cloned().collect()
    };
    a.engine
        .remove(&share, &b.engine.caller().account_id)
        .unwrap();
    let a_id = a.store.shared_note(&share).unwrap().unwrap().note_id;
    write_body(&a.store, &a_id, "After: the secret plan").unwrap();
    a.engine.sync_note(&share).unwrap();
    c.engine.refresh().unwrap();
    c.engine.sync_note(&share).unwrap();
    assert_eq!(body_of(&c, &share), "After: the secret plan");
    // B learns it is out, and keeps a private copy of what it had.
    let ended = b.engine.refresh().unwrap();
    assert_eq!(ended, vec![share.clone()]);
    assert!(b.engine.sync_note(&share).is_err() || b.store.shared_note(&share).unwrap().is_none());
    let b_id = b.store.note_with_share(&share).unwrap();
    assert_eq!(b_id, None, "B's copy is a private note now");
    let b_copy = b
        .store
        .note(&super::sharing::shared_note_id(&share))
        .unwrap()
        .unwrap();
    assert_eq!(b_copy.body, "Before");
    // No key B ever had opens anything written in the new epoch.
    let identity_a = a.engine.identity().unwrap();
    for stored in server.stored_bytes() {
        let Some((header, _)) = envelope::parse_header(&stored) else {
            continue;
        };
        if header.epoch < 2 {
            continue;
        }
        for key in &old_keys {
            assert!(envelope::open(&stored, &share, None, &identity_a.sign_pub(), key).is_err());
            assert!(envelope::open(&stored, &share, Some(0), &identity_a.sign_pub(), key).is_err());
        }
    }
    // And the server gives B nothing for the note.
    assert!(
        server
            .shares(b.engine.caller())
            .unwrap()
            .iter()
            .all(|s| s.share_id != share)
    );
}

#[test]
fn nothing_readable_reaches_the_server() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared(&a, &[(&b, Role::Comment)], "Zanzibar quartz");
    let a_id = a.store.shared_note(&share).unwrap().unwrap().note_id;
    write_body(&a.store, &a_id, "Zanzibar quartz, with walnut").unwrap();
    a.engine.sync_note(&share).unwrap();
    b.engine.sync_note(&share).unwrap();
    b.engine
        .add_comment(&share, 0..8, "Marmalade @user1")
        .unwrap();
    b.engine.sync_note(&share).unwrap();
    let plain = ["Zanzibar", "quartz", "walnut", "Marmalade", "Plan"];
    for stored in server.stored_bytes() {
        let text = String::from_utf8_lossy(&stored);
        for word in plain {
            assert!(!text.contains(word), "{word} reached the server");
        }
    }
}

#[test]
fn comments_mentions_and_orphans() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared(&a, &[(&b, Role::Comment)], "The quick brown fox");
    let id = b
        .engine
        .add_comment(&share, 4..9, "Faster? @user1")
        .unwrap();
    // Pending until the server takes it.
    assert!(b.engine.comments(&share).unwrap()[0].pending);
    b.engine.sync_note(&share).unwrap();
    assert!(!b.engine.comments(&share).unwrap()[0].pending);
    a.engine.sync_note(&share).unwrap();
    let comments = a.engine.comments(&share).unwrap();
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].comment.id, id);
    assert_eq!(comments[0].placement, Placement::Anchored(4..9));
    assert_eq!(
        comments[0].comment.mentions,
        vec![a.engine.caller().account_id.clone()]
    );
    // The mention reached A, sealed.
    let notices = a.engine.fetch_notices().unwrap();
    assert_eq!(notices.len(), 1);
    assert_eq!(notices[0].from_name, "user2");
    assert!(notices[0].excerpt.starts_with("Faster?"));
    // Text before the comment moves it; deleting its words orphans it.
    let a_id = a.store.shared_note(&share).unwrap().unwrap().note_id;
    write_body(&a.store, &a_id, "So, the quick brown fox").unwrap();
    a.engine.sync_note(&share).unwrap();
    assert_eq!(
        a.engine.comments(&share).unwrap()[0].placement,
        Placement::Anchored(8..13)
    );
    write_body(&a.store, &a_id, "So, the fox").unwrap();
    a.engine.sync_note(&share).unwrap();
    assert!(matches!(
        a.engine.comments(&share).unwrap()[0].placement,
        Placement::Orphaned(_)
    ));
    // Only its author edits it; anyone who may comment resolves it.
    a.engine.edit_comment(&share, &id, "Hijacked").unwrap();
    a.engine.resolve_comment(&share, &id, true).unwrap();
    sync(&[&a, &b], &share);
    let seen = b.engine.comments(&share).unwrap();
    assert_eq!(seen[0].comment.body, "Faster? @user1");
    assert!(seen[0].comment.resolved);
}

// no-prop-test: the last words of a note, one fixed case.
#[test]
fn a_comment_on_the_last_words_of_a_shared_note_stays_on_them() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared(&a, &[(&b, Role::Comment)], "The quick brown fox");
    b.engine.add_comment(&share, 16..19, "The end").unwrap();
    assert_eq!(
        b.engine.comments(&share).unwrap()[0].placement,
        Placement::Anchored(16..19)
    );
    b.engine.sync_note(&share).unwrap();
    a.engine.sync_note(&share).unwrap();
    assert_eq!(
        a.engine.comments(&share).unwrap()[0].placement,
        Placement::Anchored(16..19)
    );
    let a_id = a.store.shared_note(&share).unwrap().unwrap().note_id;
    write_body(&a.store, &a_id, "So, the quick brown fox").unwrap();
    a.engine.sync_note(&share).unwrap();
    assert_eq!(
        a.engine.comments(&share).unwrap()[0].placement,
        Placement::Anchored(20..23)
    );
}

// no-prop-test: the editor hands over the library text after a sync.
#[test]
fn the_editor_handing_over_the_librarys_text_adds_nothing_twice() {
    let server = MemoryCollab::new();
    let a = device(&server, 1, 1);
    let share = shared(&a, &[], "The quick brown fox");
    a.engine.add_comment(&share, 4..9, "On quick").unwrap();
    let a_id = a.store.shared_note(&share).unwrap().unwrap().note_id;
    write_body(&a.store, &a_id, "So, the quick brown fox").unwrap();
    a.engine.sync_note(&share).unwrap();
    let note = a.store.note(&a_id).unwrap().unwrap();
    let merge = a
        .engine
        .sync_editor(&share, &note.title, &note.body)
        .unwrap();
    assert_eq!(merge.body_text, "So, the quick brown fox");
    assert_eq!(
        a.engine.comments(&share).unwrap()[0].placement,
        Placement::Anchored(8..13)
    );
}

#[test]
fn many_updates_compact_and_a_new_device_starts_from_the_snapshot() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared(&a, &[(&b, Role::Edit)], "");
    let a_id = a.store.shared_note(&share).unwrap().unwrap().note_id;
    let mut body = String::new();
    for i in 0..(super::COMPACT_AFTER_UPDATES + 5) {
        body.push_str(&format!("{} ", i % 10));
        write_body(&a.store, &a_id, &body).unwrap();
        a.engine.sync_note(&share).unwrap();
    }
    assert!(server.snapshot_base(&share).unwrap() > 0);
    let (_, entries) = server.room(&share).unwrap();
    assert!(entries < 50, "{entries} entries left after compaction");
    let b2 = device(&server, 2, 2);
    b2.engine.refresh().unwrap();
    b2.engine.sync_note(&share).unwrap();
    assert_eq!(body_of(&b2, &share), body);
}

#[test]
fn an_invite_link_lets_someone_ask_and_the_owner_let_them_in() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let id = note(&a.store, "note:linked", "Plan", "Linked");
    let share = a.engine.share(&id, &[]).unwrap();
    a.engine.refresh().unwrap();
    let link = a.engine.create_link(&share, Role::Comment, 7).unwrap();
    assert!(link.starts_with(&format!("{SERVER}/j/")));
    assert!(b.engine.join("https://elsewhere.test/j/x#y").is_err());
    let joined = b.engine.join(&link).unwrap();
    assert_eq!(joined.share_id, share);
    a.engine.refresh().unwrap();
    let view = a.engine.share_view(&share).unwrap();
    assert_eq!(view.requests.len(), 1);
    assert_eq!(view.requests[0].role, Role::Comment);
    a.engine
        .approve(&share, &b.engine.caller().account_id)
        .unwrap();
    b.engine.refresh().unwrap();
    b.engine.sync_note(&share).unwrap();
    assert_eq!(body_of(&b, &share), "Linked");
    assert_eq!(b.engine.role(&share), Some(Role::Comment));
}

#[test]
fn stopping_leaves_everyone_a_private_copy() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared(&a, &[(&b, Role::Edit)], "Keep me");
    let b_id = b.store.shared_note(&share).unwrap().unwrap().note_id;
    a.engine.stop(&share).unwrap();
    b.engine.refresh().unwrap();
    let note = b.store.note(&b_id).unwrap().unwrap();
    assert_eq!(note.body, "Keep me");
    assert!(!note.extra.contains_key(kanso_core::model::SHARE_FIELD));
    // And it can be written again.
    write_body(&b.store, &b_id, "Mine now").unwrap();
}

#[test]
fn live_rooms_carry_edits_and_cursors() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared(&a, &[(&b, Role::Edit)], "Hello");
    let notices: Arc<std::sync::Mutex<Vec<LiveNotice>>> = Default::default();
    let seen = notices.clone();
    a.engine.attach(&share).unwrap();
    b.engine.attach(&share).unwrap();
    let _a_live = Engine::go_live(&a.engine, &share, Arc::new(|_| {}));
    let _b_live = Engine::go_live(
        &b.engine,
        &share,
        Arc::new(move |n| seen.lock().unwrap().push(n)),
    );
    // A types in its editor and puts its caret after the text.
    a.engine.sync_editor(&share, "Plan", "Hello there").unwrap();
    a.engine.set_caret(&share, 11, 11);
    // B's editor, with a word of its own typed meanwhile.
    let (mut title, mut body) = ("Plan".to_owned(), "Well, Hello".to_owned());
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let merge = b.engine.sync_editor(&share, &title, &body).unwrap();
        apply_splices(&mut title, &merge.title);
        apply_splices(&mut body, &merge.body);
        assert_eq!(body, merge.body_text);
        let cursors = b.engine.cursors(&share);
        if body == "Well, Hello there" && cursors.len() == 1 {
            assert_eq!(cursors[0].name, "user1");
            assert_eq!(cursors[0].head, body.len());
            break;
        }
        assert!(
            Instant::now() < deadline,
            "B's editor has {body:?}, cursors {cursors:?}"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    // A sees B's word too.
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let merge = a.engine.sync_editor(&share, "Plan", "Hello there").unwrap();
        if merge.body_text == "Well, Hello there" {
            break;
        }
        assert!(Instant::now() < deadline, "A has {:?}", merge.body_text);
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(
        notices
            .lock()
            .unwrap()
            .iter()
            .any(|n| matches!(n, LiveNotice::Connection { live: true, .. }))
    );
}

#[test]
fn someone_invited_later_reads_the_comments_written_before() {
    let server = MemoryCollab::new();
    let (a, b, c) = (
        device(&server, 1, 1),
        device(&server, 2, 1),
        device(&server, 4, 1),
    );
    let share = shared(&a, &[(&b, Role::Comment)], "Agenda");
    b.engine.add_comment(&share, 0..6, "Earlier").unwrap();
    b.engine.sync_note(&share).unwrap();
    a.engine.person("user4").unwrap();
    a.engine.invite(&share, "user4", Role::Comment).unwrap();
    c.engine.refresh().unwrap();
    assert_eq!(c.engine.invitations().len(), 1);
    c.engine.accept(&share).unwrap();
    assert_eq!(body_of(&c, &share), "Agenda");
    let comments = c.engine.comments(&share).unwrap();
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].comment.body, "Earlier");
    assert_eq!(comments[0].comment.author_name, "user2");
}

#[test]
fn a_writer_an_epoch_behind_catches_up_and_its_edit_lands() {
    let server = MemoryCollab::new();
    let (a, b, c) = (
        device(&server, 1, 1),
        device(&server, 2, 1),
        device(&server, 4, 1),
    );
    let share = shared(&a, &[(&b, Role::Edit), (&c, Role::Edit)], "Draft");
    a.engine
        .remove(&share, &c.engine.caller().account_id)
        .unwrap();
    // B hasn't heard of epoch 2 when it writes.
    let b_id = b.store.shared_note(&share).unwrap().unwrap().note_id;
    write_body(&b.store, &b_id, "Draft two").unwrap();
    b.engine.sync_note(&share).unwrap();
    a.engine.sync_note(&share).unwrap();
    assert_eq!(body_of(&a, &share), "Draft two");
    assert_eq!(b.store.shared_note(&share).unwrap().unwrap().epoch, 2);
}

#[test]
fn an_editor_made_a_viewer_drops_what_it_hadnt_sent() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared(&a, &[(&b, Role::Edit)], "Steady");
    let b_id = b.store.shared_note(&share).unwrap().unwrap().note_id;
    write_body(&b.store, &b_id, "Steady, unsent").unwrap();
    a.engine
        .set_role(&share, &b.engine.caller().account_id, Role::View)
        .unwrap();
    b.engine.sync_note(&share).unwrap();
    assert_eq!(b.engine.role(&share), Some(Role::View));
    assert_eq!(body_of(&b, &share), "Steady");
    assert!(b.store.outbox(&share).unwrap().is_empty());
    assert!(write_body(&b.store, &b_id, "Again").is_err());
}

// --- What the security review found, each kept from coming back.

/// Seals `plain` as `device`'s next envelope of `kind` in the note.
fn sealed(device: &Device, share: &str, kind: Kind, plain: &[u8]) -> Vec<u8> {
    let loaded = device.engine.loaded(share).unwrap();
    let mut guard = lock(&loaded);
    device.engine.seal(&mut guard, kind, None, plain).unwrap().1
}

#[test]
fn an_author_outside_the_epochs_list_is_never_merged() {
    let server = MemoryCollab::new();
    let (a, b, d) = (
        device(&server, 1, 1),
        device(&server, 2, 1),
        device(&server, 5, 1),
    );
    let share = shared(&a, &[(&b, Role::Edit)], "Ours");
    // D is known to A (pinned through a note of its own), but not in this one.
    let other = note(&d.store, "note:other", "D", "d");
    d.engine.person("user1").unwrap();
    let other_share = d
        .engine
        .share(&other, &[("user1".into(), Role::Edit)])
        .unwrap();
    a.engine.refresh().unwrap();
    a.engine.accept(&other_share).unwrap();
    // D signs an update for A's note with a key that opens there (B's,
    // as a server could hand out one of its own choosing).
    let forged = {
        let b_loaded = b.engine.loaded(&share).unwrap();
        let key = lock(&b_loaded).keys[&1].clone();
        let doc = crate::doc::CollabDoc::from_text("", "Ours");
        let update = doc.set_text(Field::Body, "Forged").unwrap();
        let header = crate::model::Header {
            kind: Kind::Update,
            epoch: 1,
            author_account: d.engine.caller().account_id.clone(),
            author_device: d.engine.caller().device_id.clone(),
            counter: 1,
        };
        envelope::seal(
            &d.engine.identity().unwrap(),
            &key,
            &share,
            &header,
            None,
            &update,
        )
    };
    let loaded = a.engine.loaded(&share).unwrap();
    let mut guard = lock(&loaded);
    let next = guard.note.applied_seq + 1;
    guard.inbox.insert(next, forged);
    let merge = a.engine.merge_delivered(&mut guard).unwrap();
    assert!(!merge.text);
    assert_eq!(guard.doc.text(Field::Body), "Ours");
}

#[test]
fn an_older_member_list_is_refused() {
    let server = MemoryCollab::new();
    let (a, b, c) = (
        device(&server, 1, 1),
        device(&server, 2, 1),
        device(&server, 4, 1),
    );
    let share = shared(&a, &[(&b, Role::Edit), (&c, Role::Edit)], "x");
    let old = server
        .shares(b.engine.caller())
        .unwrap()
        .into_iter()
        .find(|s| s.share_id == share)
        .unwrap();
    a.engine
        .remove(&share, &c.engine.caller().account_id)
        .unwrap();
    b.engine.refresh().unwrap();
    assert_eq!(b.store.shared_note(&share).unwrap().unwrap().epoch, 2);
    // A server replaying epoch 1's list (with C still in it) is refused...
    let err = b.engine.take_info(&old).unwrap_err();
    assert!(
        matches!(
            err,
            crate::api::CollabError::List(crate::members::ListError::Stale)
        ),
        "{err:?}"
    );
    // ...on another device of the account too, which knows the epoch from
    // the note's kansoShare.
    let b2 = device(&server, 2, 2);
    let b_id = b.store.shared_note(&share).unwrap().unwrap().note_id;
    let mark = b.store.note(&b_id).unwrap().unwrap().extra[kanso_core::model::SHARE_FIELD].clone();
    note(&b2.store, &b_id, "", "");
    b2.store
        .put(
            DocType::Note,
            json!({"_id": b_id, kanso_core::model::SHARE_FIELD: mark}),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
    let err = b2.engine.take_info(&old).unwrap_err();
    assert!(
        matches!(
            err,
            crate::api::CollabError::List(crate::members::ListError::Stale)
        ),
        "{err:?}"
    );
}

#[test]
fn coming_back_to_a_note_doesnt_duplicate_its_text() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared(&a, &[(&b, Role::Edit)], "Only once");
    a.engine
        .remove(&share, &b.engine.caller().account_id)
        .unwrap();
    b.engine.refresh().unwrap();
    let copy = super::sharing::shared_note_id(&share);
    write_body(&b.store, &copy, "Only once, and my own").unwrap();
    a.engine.invite(&share, "user2", Role::Edit).unwrap();
    b.engine.refresh().unwrap();
    b.engine.accept(&share).unwrap();
    sync(&[&a, &b], &share);
    assert_eq!(body_of(&a, &share), "Only once");
    assert_eq!(body_of(&b, &share), "Only once");
    // The private copy stays as it was, a note of its own.
    assert_eq!(
        b.store.note(&copy).unwrap().unwrap().body,
        "Only once, and my own"
    );
    assert_ne!(b.store.shared_note(&share).unwrap().unwrap().note_id, copy);
}

#[test]
fn an_update_that_doesnt_decode_is_skipped_and_the_log_goes_on() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared(&a, &[(&b, Role::Edit)], "Steady");
    let garbage = sealed(&b, &share, Kind::Update, b"not a yrs update");
    server.append(b.engine.caller(), &share, &garbage).unwrap();
    let b_id = b.store.shared_note(&share).unwrap().unwrap().note_id;
    write_body(&b.store, &b_id, "Steady on").unwrap();
    b.engine.sync_note(&share).unwrap();
    a.engine.sync_note(&share).unwrap();
    assert_eq!(body_of(&a, &share), "Steady on");
}

#[test]
fn a_list_cant_pin_someone_elses_account_under_other_keys() {
    let server = MemoryCollab::new();
    let (a, c, m) = (
        device(&server, 1, 1),
        device(&server, 4, 1),
        device(&server, 6, 1),
    );
    let identity_m = m.engine.identity().unwrap();
    let forged = crate::model::Member {
        account: c.engine.caller().account_id.clone(),
        username: "user4".into(),
        role: Role::Edit,
        sign_pub: crate::codec::b64(&identity_m.sign_pub()),
        box_pub: crate::codec::b64(&identity_m.box_pub()),
    };
    assert!(a.engine.pin_member(&forged).is_err());
    assert!(
        a.store
            .contact(&c.engine.caller().account_id)
            .unwrap()
            .is_none()
    );
}

#[test]
fn a_notice_without_its_signed_comment_is_dropped() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared(&a, &[(&b, Role::Comment)], "Quiet");
    let a_identity = a.engine.identity().unwrap();
    let notice = crate::model::Notice {
        comment: "no-such-comment".into(),
        title: "Urgent: reset your password".into(),
        excerpt: "click here".into(),
        at: 1,
    };
    let sealed = crate::keys::seal_notice(
        &a.engine.caller().account_id,
        &a_identity.box_pub(),
        &share,
        &b.engine.caller().account_id,
        &serde_json::to_vec(&notice).unwrap(),
    )
    .unwrap();
    server
        .notify(
            b.engine.caller(),
            &crate::api::NoticeIn {
                share_id: share.clone(),
                recipient: a.engine.caller().account_id.clone(),
                sealed,
            },
        )
        .unwrap();
    assert!(a.engine.fetch_notices().unwrap().is_empty());
    assert!(a.engine.notices().unwrap().is_empty());
}

#[test]
fn a_new_key_is_accepted_only_with_the_fingerprint_compared() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let shown = a.engine.offered_key("user2").unwrap();
    assert!(
        a.engine
            .accept_new_key("user2", "00000 00000 00000 00000 00000 00000")
            .is_err()
    );
    let person = a
        .engine
        .accept_new_key("user2", &shown.fingerprint)
        .unwrap();
    assert_eq!(person.fingerprint, b.engine.fingerprint().unwrap());
}

#[test]
fn a_long_paste_goes_in_parts_under_the_cap() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared(&a, &[(&b, Role::Edit)], "");
    let long: String = "a long paste, ".repeat(30_000);
    assert!(long.len() > envelope::MAX_UPDATE);
    let a_id = a.store.shared_note(&share).unwrap().unwrap().note_id;
    write_body(&a.store, &a_id, &long).unwrap();
    a.engine.sync_note(&share).unwrap();
    b.engine.sync_note(&share).unwrap();
    assert_eq!(body_of(&b, &share), long);
}

// --- What the second review found.

/// The members of the note's newest signed list, by username.
fn listed(device: &Device, share: &str) -> Vec<String> {
    let loaded = device.engine.loaded(share).unwrap();
    let guard = lock(&loaded);
    let (_, list) = guard.lists.iter().next_back().unwrap();
    list.members.iter().map(|m| m.username.clone()).collect()
}

#[test]
fn leaving_or_declining_takes_someone_out_of_the_next_list() {
    let server = MemoryCollab::new();
    let (a, b, c, e, f) = (
        device(&server, 1, 1),
        device(&server, 2, 1),
        device(&server, 4, 1),
        device(&server, 5, 1),
        device(&server, 6, 1),
    );
    let share = shared(&a, &[(&b, Role::Edit), (&c, Role::Edit)], "x");
    a.engine.person("user5").unwrap();
    a.engine.invite(&share, "user5", Role::Edit).unwrap();
    e.engine.refresh().unwrap();
    e.engine.decline(&share).unwrap();
    b.engine.leave(&share).unwrap();
    // The owner's next refresh starts an epoch without them.
    let before = a.store.shared_note(&share).unwrap().unwrap().epoch;
    a.engine.refresh().unwrap();
    assert!(a.store.shared_note(&share).unwrap().unwrap().epoch > before);
    let names = listed(&a, &share);
    assert!(!names.contains(&"user2".to_owned()), "{names:?}");
    assert!(!names.contains(&"user5".to_owned()), "{names:?}");
    // Inviting someone else later doesn't bring them back.
    a.engine.person("user6").unwrap();
    a.engine.invite(&share, "user6", Role::Edit).unwrap();
    let names = listed(&a, &share);
    assert!(names.contains(&"user6".to_owned()), "{names:?}");
    assert!(!names.contains(&"user2".to_owned()), "{names:?}");
    // And the note goes on for those still in it.
    f.engine.refresh().unwrap();
    f.engine.accept(&share).unwrap();
    let a_id = a.store.shared_note(&share).unwrap().unwrap().note_id;
    write_body(&a.store, &a_id, "Without them").unwrap();
    a.engine.sync_note(&share).unwrap();
    c.engine.refresh().unwrap();
    sync(&[&c, &f], &share);
    assert_eq!(body_of(&c, &share), "Without them");
    assert_eq!(body_of(&f, &share), "Without them");
}

#[test]
fn a_deleted_account_doesnt_stop_refresh_sync_or_other_invitations() {
    let server = MemoryCollab::new();
    let (a, b, c, d, e) = (
        device(&server, 1, 1),
        device(&server, 2, 1),
        device(&server, 4, 1),
        device(&server, 5, 1),
        device(&server, 6, 1),
    );
    let id = note(&a.store, "note:shared", "Plan", "Still here");
    a.engine.person("user2").unwrap();
    a.engine.person("user4").unwrap();
    let share = a
        .engine
        .share(
            &id,
            &[("user2".into(), Role::Edit), ("user4".into(), Role::Edit)],
        )
        .unwrap();
    // Two others invite B; one of them is deleted before B looks.
    for other in [&d, &e] {
        let id = note(&other.store, "note:theirs", "Theirs", "t");
        other.engine.person("user2").unwrap();
        other
            .engine
            .share(&id, &[("user2".into(), Role::Edit)])
            .unwrap();
    }
    server.delete_account(&c.engine.caller().account_id);
    server.delete_account(&d.engine.caller().account_id);
    b.engine.refresh().unwrap();
    // E's invitation shows, though D's can't be checked.
    let invited: Vec<String> = b
        .engine
        .invitations()
        .into_iter()
        .map(|i| i.owner.username)
        .collect();
    assert!(invited.contains(&"user6".to_owned()), "{invited:?}");
    assert!(!invited.contains(&"user5".to_owned()), "{invited:?}");
    b.engine.accept(&share).unwrap();
    assert_eq!(body_of(&b, &share), "Still here");
    // C, never seen by B, is shown unverified.
    let view = b.engine.share_view(&share).unwrap();
    let c_view = view
        .members
        .iter()
        .find(|m| m.person.account_id == c.engine.caller().account_id)
        .unwrap();
    assert!(c_view.unverified);
    // The owner's refresh and sync go on, and the next epoch leaves C out.
    a.engine.refresh().unwrap();
    a.engine
        .set_role(&share, &b.engine.caller().account_id, Role::Comment)
        .unwrap();
    assert!(!listed(&a, &share).contains(&"user4".to_owned()));
    write_body(&a.store, &id, "Still here, later").unwrap();
    a.engine.sync_note(&share).unwrap();
    b.engine.refresh().unwrap();
    b.engine.sync_note(&share).unwrap();
    assert_eq!(body_of(&b, &share), "Still here, later");
}

#[test]
fn a_title_longer_than_the_library_takes_is_cut_and_merging_goes_on() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared(&a, &[(&b, Role::Edit)], "Body");
    let long = "t".repeat(kanso_core::model::MAX_TITLE_LEN + 50);
    let update = {
        let loaded = a.engine.loaded(&share).unwrap();
        let guard = lock(&loaded);
        guard.doc.set_text(Field::Title, &long).unwrap()
    };
    let envelope = sealed(&a, &share, Kind::Update, &update);
    server.append(a.engine.caller(), &share, &envelope).unwrap();
    b.engine.sync_note(&share).unwrap();
    let b_id = b.store.shared_note(&share).unwrap().unwrap().note_id;
    let title = b.store.note(&b_id).unwrap().unwrap().title;
    assert_eq!(title.chars().count(), kanso_core::model::MAX_TITLE_LEN);
    assert!(b.engine.problems().iter().any(|p| p.share_id == share));
    // The cut isn't taken as a deletion, and later edits still land.
    a.engine.sync_note(&share).unwrap();
    let a_id = a.store.shared_note(&share).unwrap().unwrap().note_id;
    write_body(&a.store, &a_id, "Body, and more").unwrap();
    a.engine.sync_note(&share).unwrap();
    b.engine.sync_note(&share).unwrap();
    assert_eq!(body_of(&b, &share), "Body, and more");
    let loaded = b.engine.loaded(&share).unwrap();
    assert_eq!(lock(&loaded).doc.text(Field::Title), long);
}

#[test]
fn an_update_relayed_after_its_epoch_ended_is_dropped() {
    let server = MemoryCollab::new();
    let (a, b, c) = (
        device(&server, 1, 1),
        device(&server, 2, 1),
        device(&server, 4, 1),
    );
    let share = shared(&a, &[(&b, Role::Edit), (&c, Role::Edit)], "Ours");
    // B writes under epoch 1; the server holds it back.
    let late = {
        let loaded = b.engine.loaded(&share).unwrap();
        let update = lock(&loaded).doc.set_text(Field::Body, "Late").unwrap();
        drop(loaded);
        sealed(&b, &share, Kind::Update, &update)
    };
    a.engine
        .remove(&share, &b.engine.caller().account_id)
        .unwrap();
    c.engine.refresh().unwrap();
    c.engine.sync_note(&share).unwrap();
    // ...and relays it after epoch 2 began.
    let loaded = c.engine.loaded(&share).unwrap();
    let mut guard = lock(&loaded);
    assert!(guard.lists[&2].from_seq.is_some());
    let next = guard.note.applied_seq + 1;
    guard.inbox.insert(next, late);
    let merge = c.engine.merge_delivered(&mut guard).unwrap();
    assert!(!merge.text);
    assert_eq!(guard.doc.text(Field::Body), "Ours");
}

#[test]
fn edits_that_cant_be_sent_are_kept_and_the_person_told() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared(&a, &[(&b, Role::Edit)], "Steady");
    let b_id = b.store.shared_note(&share).unwrap().unwrap().note_id;
    write_body(&b.store, &b_id, "Steady, unsent").unwrap();
    a.engine
        .set_role(&share, &b.engine.caller().account_id, Role::View)
        .unwrap();
    b.engine.sync_note(&share).unwrap();
    assert_eq!(body_of(&b, &share), "Steady");
    let book = b.store.note(&b_id).unwrap().unwrap().book_id;
    let kept: Vec<_> = b
        .store
        .notes_in_books(&[book])
        .unwrap()
        .into_iter()
        .filter(|n| n.title.ends_with("(not shared)"))
        .collect();
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].body, "Steady, unsent");
    assert!(b.engine.problems().iter().any(|p| p.share_id == share));
}

#[test]
fn an_edit_past_the_size_limit_is_kept_as_a_private_note() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared(&a, &[(&b, Role::Edit)], "Small");
    let loaded = a.engine.loaded(&share).unwrap();
    let mut guard = lock(&loaded);
    // The CRDT is already at the limit (history and text the library cut).
    guard
        .doc
        .set_text(Field::Title, &"x".repeat(super::STATE_LIMIT))
        .unwrap();
    let local = (guard.base.0.clone(), format!("{} and more", guard.base.1));
    assert!(!a.engine.take_local(&mut guard, &local).unwrap());
    assert!(a.store.outbox(&share).unwrap().is_empty());
    drop(guard);
    assert!(a.engine.problems().iter().any(|p| p.share_id == share));
    let use_ = a.engine.size_use(&share).unwrap();
    assert!(use_.is_near());
    let a_id = a.store.shared_note(&share).unwrap().unwrap().note_id;
    let book = a.store.note(&a_id).unwrap().unwrap().book_id;
    assert!(
        a.store
            .notes_in_books(&[book])
            .unwrap()
            .iter()
            .any(|n| n.title.ends_with("(not shared)") && n.body == "Small and more")
    );
}

#[test]
fn coming_back_on_two_devices_makes_one_note() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared(&a, &[(&b, Role::Edit)], "Once");
    a.engine
        .remove(&share, &b.engine.caller().account_id)
        .unwrap();
    b.engine.refresh().unwrap();
    // B's second device has the private copy too (private sync brings it).
    let copy = super::sharing::shared_note_id(&share);
    let b2 = device(&server, 2, 2);
    note(&b2.store, &copy, "Plan", "Once");
    a.engine.invite(&share, "user2", Role::Edit).unwrap();
    b.engine.refresh().unwrap();
    b.engine.accept(&share).unwrap();
    b2.engine.refresh().unwrap();
    b2.engine.sync_note(&share).unwrap();
    let one = b.store.shared_note(&share).unwrap().unwrap().note_id;
    let two = b2.store.shared_note(&share).unwrap().unwrap().note_id;
    assert_eq!(one, two);
    assert_ne!(one, copy);
    assert_eq!(body_of(&b2, &share), "Once");
}

#[test]
fn a_shared_note_locked_elsewhere_gets_no_shared_text() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared(&a, &[(&b, Role::Edit)], "Open");
    let b_id = b.store.shared_note(&share).unwrap().unwrap().note_id;
    let mark = b.store.note(&b_id).unwrap().unwrap().extra[kanso_core::model::SHARE_FIELD].clone();
    // Another of B's devices locked it before it learnt the note is shared.
    b.store
        .put(
            DocType::Note,
            json!({
                "_id": b_id,
                "body": "",
                "kansoLock": {"v": 1, "salt": "AAAA", "nonce": "AAAA", "sealed": "AAAA"}
            }),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
    let written = b.store.write_shared_text(&kanso_store::SharedText {
        note_id: &b_id,
        title: "Plan",
        body: "Open, in the clear",
        mark: Some(mark),
        new_in: "",
        expected_rev: None,
        authors: &[],
    });
    assert!(
        matches!(&written, Err(kanso_store::StoreError::Invalid(m)) if m == kanso_store::LOCKED_SHARED),
        "{written:?}"
    );
    let a_id = a.store.shared_note(&share).unwrap().unwrap().note_id;
    write_body(&a.store, &a_id, "Open, in the clear").unwrap();
    a.engine.sync_note(&share).unwrap();
    b.engine.sync_note(&share).unwrap();
    assert_eq!(b.store.note(&b_id).unwrap().unwrap().body, "");
    assert!(
        b.engine
            .problems()
            .iter()
            .any(|p| p.share_id == share && p.message == kanso_store::LOCKED_SHARED)
    );
}

#[test]
fn rebase_keeps_a_local_edit_and_the_remote_one() {
    assert_eq!(rebase("abc", "abXc", "Zabc"), "ZabXc");
    assert_eq!(rebase("abc", "abc", "Zabc"), "Zabc");
    assert_eq!(rebase("abc", "ab", "abc"), "ab");
}

/// One step of the model.
#[derive(Clone, Debug)]
enum Step {
    /// Device `d` writes its library: inserts `text` at `at` (a fraction of
    /// the length) after deleting `delete` characters there.
    Write {
        d: usize,
        at: u8,
        delete: u8,
        text: String,
    },
    /// Device `d`'s open editor does the same, then syncs with the engine.
    Type {
        d: usize,
        at: u8,
        delete: u8,
        text: String,
    },
    /// Device `d` syncs over HTTP.
    Sync { d: usize },
    /// The server is unreachable for `n` requests.
    Offline { n: usize },
}

fn step(devices: usize) -> impl Strategy<Value = Step> {
    prop_oneof![
        3 => (0..devices, any::<u8>(), 0u8..4, "[a-zé ]{1,4}")
            .prop_map(|(d, at, delete, text)| Step::Write { d, at, delete, text }),
        3 => (0..devices, any::<u8>(), 0u8..4, "[a-zé ]{1,4}")
            .prop_map(|(d, at, delete, text)| Step::Type { d, at, delete, text }),
        3 => (0..devices).prop_map(|d| Step::Sync { d }),
        1 => (1usize..3).prop_map(|n| Step::Offline { n }),
    ]
}

/// Replaces `delete` characters at a fraction `at` of `text` with `insert`.
fn edit(text: &str, at: u8, delete: u8, insert: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let start = chars.len() * at as usize / 256;
    let end = (start + delete as usize).min(chars.len());
    let mut out: String = chars[..start].iter().collect();
    out.push_str(insert);
    out.extend(&chars[end..]);
    out
}

proptest! {
    #![proptest_config(crate::tests::config(24))]

    /// Whatever the devices write, in their libraries or open editors, and
    /// in whatever order they sync (with the server sometimes away), once
    /// everyone has synced every library holds the same text, and it is the
    /// CRDT's.
    #[test]
    fn every_library_converges(steps in proptest::collection::vec(step(3), 1..40)) {
        let server = MemoryCollab::new();
        let a = device(&server, 1, 1);
        let b = device(&server, 2, 1);
        let b2 = device(&server, 2, 2);
        let share = shared(&a, &[(&b, Role::Edit)], "start");
        b2.engine.refresh().unwrap();
        b2.engine.sync_note(&share).unwrap();
        let devices = [&a, &b, &b2];
        // Each device's editor: its text, while open.
        let mut editors: Vec<Option<String>> = vec![None; 3];
        for step in steps {
            match step {
                Step::Write { d, at, delete, text } => {
                    // A library write while the editor is open goes through
                    // the editor, as in the app.
                    if editors[d].is_some() {
                        continue;
                    }
                    let id = devices[d].store.shared_note(&share).unwrap().unwrap().note_id;
                    let now = devices[d].store.note(&id).unwrap().unwrap().body;
                    write_body(&devices[d].store, &id, &edit(&now, at, delete, &text)).unwrap();
                }
                Step::Type { d, at, delete, text } => {
                    let engine = &devices[d].engine;
                    let editor = editors[d].get_or_insert_with(|| {
                        engine.attach(&share).unwrap();
                        body_of(devices[d], &share)
                    });
                    *editor = edit(editor, at, delete, &text);
                    let merge = engine.sync_editor(&share, "Plan", editor).unwrap();
                    apply_splices(editor, &merge.body);
                    prop_assert_eq!(&*editor, &merge.body_text);
                }
                Step::Sync { d } => {
                    let _ = devices[d].engine.sync_note(&share);
                }
                Step::Offline { n } => server.go_offline(n),
            }
        }
        server.go_offline(0);
        // Everyone closes their editor (after a last merge) and syncs.
        for (d, editor) in editors.iter_mut().enumerate() {
            if let Some(text) = editor.take() {
                devices[d].engine.sync_editor(&share, "Plan", &text).unwrap();
                devices[d].engine.detach(&share);
            }
        }
        for _ in 0..3 {
            for d in devices {
                d.engine.sync_note(&share).unwrap();
            }
        }
        let texts: Vec<String> = devices.iter().map(|d| body_of(d, &share)).collect();
        prop_assert_eq!(&texts[0], &texts[1]);
        prop_assert_eq!(&texts[1], &texts[2]);
        for d in devices {
            let loaded = d.engine.loaded(&share).unwrap();
            let guard = lock(&loaded);
            prop_assert_eq!(guard.doc.text(Field::Body), texts[0].clone());
        }
    }

    /// A rebased edit lands where its text went: with nothing concurrent it
    /// is the edit, and with no edit it is the concurrent text.
    #[test]
    fn rebase_is_the_edit_or_the_concurrent_text(base in "[a-c]{0,12}", local in "[a-c]{0,12}", current in "[a-c]{0,12}") {
        prop_assert_eq!(rebase(&base, &local, &base), local.clone());
        prop_assert_eq!(rebase(&base, &base, &current), current.clone());
        let _ = rebase(&base, &local, &current);
    }

    /// What the library is given always fits it, and is the start of the
    /// shared text (whole characters), or all of it.
    #[test]
    fn clamped_text_fits_and_is_a_prefix(title in "[aé😀]{0,200}", extra in 0usize..4) {
        let body = "b".repeat(kanso_core::model::MAX_BODY_LEN - 2 + extra);
        let (t, b) = super::clamp_text(title.clone(), body.clone());
        prop_assert!(t.chars().count() <= kanso_core::model::MAX_TITLE_LEN);
        prop_assert!(b.chars().count() <= kanso_core::model::MAX_BODY_LEN);
        prop_assert!(title.starts_with(&t) && body.starts_with(&b));
        prop_assert_eq!(t == title, title.chars().count() <= kanso_core::model::MAX_TITLE_LEN);
    }

    /// An entry is dropped as late exactly when a later epoch's list began
    /// before it.
    #[test]
    fn late_means_after_a_later_epochs_start(froms in proptest::collection::vec(proptest::option::of(0u64..50), 1..5), epoch in 1u32..5, seq in 0u64..60) {
        let server = MemoryCollab::new();
        let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
        let share = shared(&a, &[(&b, Role::Edit)], "x");
        let loaded = a.engine.loaded(&share).unwrap();
        let mut guard = lock(&loaded);
        let template = guard.lists.values().next().unwrap().clone();
        guard.lists.clear();
        for (i, from) in froms.iter().enumerate() {
            let mut list = template.clone();
            list.epoch = i as u32 + 1;
            list.from_seq = *from;
            guard.lists.insert(list.epoch, list);
        }
        let expected = froms
            .iter()
            .enumerate()
            .skip(epoch as usize)
            .find_map(|(_, from)| *from)
            .is_some_and(|from| seq > from);
        prop_assert_eq!(super::log::past_its_epoch(&guard, epoch, seq), expected);
    }
}
