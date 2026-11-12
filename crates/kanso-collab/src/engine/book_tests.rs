//! Shared notebooks against the in-memory server: sharing an existing
//! notebook (batches, resume, locked notes), joining, pulling and pushing,
//! the chain a device trusts, and old epochs read through history links.
//! Property tests run on the seed `PROPTEST_RNG_SEED` pins.

use std::collections::BTreeMap;

use kanso_core::id::DocType;
use kanso_store::{Origin, Put, Store};
use proptest::prelude::*;
use serde_json::json;

use super::tests::{Device, device};
use crate::api::{AccountWrap, CollabApi, CollabError, NewBookList};
use crate::book::{history_link, wrap};
use crate::book_members::sign;
use crate::book_members_model::{BookListError, BookMemberList};
use crate::book_model::BookKey;
use crate::codec::b64;
use crate::memory::MemoryCollab;
use crate::model::{Member, Role};

/// A notebook holding `notes` (title, text); its id and the notes' ids.
fn notebook(store: &Store, name: &str, notes: &[(String, String)]) -> (String, Vec<String>) {
    let book = store
        .put(
            DocType::Book,
            json!({"name": name}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let ids = notes
        .iter()
        .enumerate()
        .map(|(i, (title, body))| {
            store
                .put(
                    DocType::Note,
                    json!({"_id": format!("note:n{i}"), "bookId": book, "title": title, "body": body}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap()
                .id
        })
        .collect();
    (book, ids)
}

/// Title to text of the notes in a notebook of a library.
fn texts(device: &Device, book: &str) -> BTreeMap<String, String> {
    device
        .store
        .notes_in_books(&[book.to_owned()])
        .unwrap()
        .into_iter()
        .map(|n| (n.title, n.body))
        .collect()
}

fn note_of(device: &Device, book: &str, title: &str) -> String {
    device
        .store
        .notes_in_books(&[book.to_owned()])
        .unwrap()
        .into_iter()
        .find(|n| n.title == title)
        .unwrap_or_else(|| panic!("no note {title}"))
        .id
}

fn write(device: &Device, id: &str, body: &str) {
    device
        .store
        .put(
            DocType::Note,
            json!({"_id": id, "body": body}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
}

fn body(device: &Device, id: &str) -> String {
    device.store.note(id).unwrap().unwrap().body
}

fn invite(others: &[(&Device, Role)]) -> Vec<(String, Role)> {
    others
        .iter()
        .map(|(d, role)| (d.engine.caller().username.clone(), *role))
        .collect()
}

/// A shares `book` with the others, who each join; the share's id.
fn shared(owner: &Device, others: &[(&Device, Role)], book: &str) -> String {
    for (other, _) in others {
        owner
            .engine
            .person(&other.engine.caller().username)
            .unwrap();
    }
    let done = owner
        .engine
        .share_book(book, &invite(others), false, &mut |_| {})
        .unwrap();
    for (other, _) in others {
        other.engine.join_book(&done.share_id).unwrap();
    }
    done.share_id
}

fn sync(devices: &[&Device], share: &str) {
    for _ in 0..3 {
        for d in devices {
            d.engine.sync_book(share).unwrap();
        }
    }
}

fn member_of(d: &Device, role: Role) -> Member {
    let identity = d.engine.identity().unwrap();
    Member {
        account: d.engine.caller().account_id.clone(),
        username: d.engine.caller().username.clone(),
        role,
        sign_pub: b64(&identity.sign_pub()),
        box_pub: b64(&identity.box_pub()),
    }
}

/// The owner posts the next list, with `people` besides them: in the same
/// epoch (the added get a wrap), or in a new one (a fresh key, a wrap for
/// everyone and a history link). The key of the epoch the list is in.
fn post_list(
    server: &MemoryCollab,
    owner: &Device,
    share: &str,
    people: &[(&Device, Role)],
    new_epoch: bool,
) -> BookKey {
    let mut members = vec![member_of(owner, Role::Owner)];
    members.extend(people.iter().map(|(d, role)| member_of(d, *role)));
    post_members(server, owner, share, members, new_epoch)
}

/// `post_list` with the entries given (the owner's included).
fn post_members(
    server: &MemoryCollab,
    owner: &Device,
    share: &str,
    members: Vec<Member>,
    new_epoch: bool,
) -> BookKey {
    let ctx = owner.engine.book_ctx(share).unwrap();
    let old = ctx.keys[&ctx.row.epoch].clone();
    let head = ctx.chain.head.clone();
    let (epoch, key, from_seq) = if new_epoch {
        let epoch = head.list.epoch + 1;
        (
            epoch,
            BookKey::generate(),
            Some(server.book_room(share).unwrap().0),
        )
    } else {
        (head.list.epoch, old.clone(), head.list.from_seq)
    };
    let mut list = BookMemberList {
        v: 1,
        share: share.to_owned(),
        rev: head.list.rev + 1,
        epoch,
        owner: head.list.owner.clone(),
        signer: owner.engine.caller().account_id.clone(),
        prev: Some(head.hash.clone()),
        key_commit: key.commit(share, epoch),
        at: kanso_core::now_ms(),
        from_seq,
        members,
    };
    list.sort();
    let signed = sign(&owner.engine.identity().unwrap(), &list);
    let before: Vec<String> = head
        .list
        .members
        .iter()
        .map(|m| m.account.clone())
        .collect();
    let wraps = list
        .members
        .iter()
        .filter(|m| new_epoch || !before.contains(&m.account))
        .map(|m| AccountWrap {
            account: m.account.clone(),
            wrapped: wrap(
                &key,
                share,
                epoch,
                &m.account,
                m.role,
                &crate::codec::unb64_array::<32>(&m.box_pub).unwrap(),
            )
            .unwrap(),
            epoch: None,
        })
        .collect();
    owner
        .engine
        .api
        .new_book_list(
            owner.engine.caller(),
            share,
            &NewBookList {
                rev: list.rev,
                list: signed.list,
                list_sig: signed.sig,
                wraps,
                history_link: new_epoch.then(|| b64(&history_link(share, epoch, &key, &old))),
            },
        )
        .unwrap();
    key
}

#[test]
fn an_owner_shares_300_notes_and_a_member_joins_and_reads_all() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let notes: Vec<(String, String)> = (0..300)
        .map(|i| (format!("Note {i}"), format!("Text of note {i}\n")))
        .collect();
    let (book, _) = notebook(&a.store, "Studio", &notes);
    a.engine.person("user2").unwrap();
    let mut progress = Vec::new();
    let done = a
        .engine
        .share_book(&book, &[("user2".into(), Role::Edit)], false, &mut |p| {
            progress.push((p.done, p.total))
        })
        .unwrap();
    assert_eq!(done.shared, 300);
    // "Sharing Studio… 120 of 340": batches of 50.
    assert_eq!(
        progress,
        vec![
            (0, 300),
            (50, 300),
            (100, 300),
            (150, 300),
            (200, 300),
            (250, 300),
            (300, 300)
        ]
    );
    let joined = b.engine.join_book(&done.share_id).unwrap();
    assert_eq!(joined.notes, 300);
    let want: BTreeMap<String, String> = notes.into_iter().collect();
    assert_eq!(texts(&b, &joined.book_id), want);
    let name = b
        .store
        .books()
        .unwrap()
        .into_iter()
        .find(|k| k.id == joined.book_id)
        .unwrap()
        .name;
    assert_eq!(name, "Studio");
    // The server holds the manifest and 300 notes, sealed.
    assert_eq!(server.book_room(&done.share_id).map(|r| r.1), Some(301));
    for bytes in server.book_bytes() {
        assert!(
            !bytes.windows(12).any(|w| w == b"Text of note"),
            "plaintext reached the server"
        );
    }
    // Nothing is left to do.
    let sync = b.engine.sync_book(&done.share_id).unwrap();
    assert_eq!((sync.pushed, sync.notes_written), (0, 0));
}

#[test]
fn sharing_stopped_part_way_goes_on_without_sending_notes_twice() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let notes: Vec<(String, String)> = (0..120)
        .map(|i| (format!("Note {i}"), format!("Text {i}\n")))
        .collect();
    let (book, _) = notebook(&a.store, "Studio", &notes);
    a.engine.person("user2").unwrap();
    // The connection fails once, some way into the notes.
    server.go_offline_after(80, 1);
    let err = a
        .engine
        .share_book(&book, &[("user2".into(), Role::Edit)], false, &mut |_| {})
        .unwrap_err();
    assert!(err.is_offline(), "{err:?}");
    let row = a.store.shared_book_of(&book).unwrap().unwrap();
    assert!(row.uploading);
    let up = a
        .store
        .book_streams(&row.share_id)
        .unwrap()
        .iter()
        .filter(|s| s.on_server)
        .count();
    assert!(up > 0 && up < 120, "{up}");
    let mut progress = Vec::new();
    a.engine
        .resume_book_share(&row.share_id, &mut |p| progress.push(p.done))
        .unwrap();
    // It went on from the notes that were up.
    assert!(progress[0] > 0, "{progress:?}");
    assert_eq!(progress.last(), Some(&120));
    assert!(
        !a.store
            .shared_book(&row.share_id)
            .unwrap()
            .unwrap()
            .uploading
    );
    assert_eq!(server.book_room(&row.share_id).map(|r| r.1), Some(121));
    let joined = b.engine.join_book(&row.share_id).unwrap();
    assert_eq!(joined.notes, 120);
    assert_eq!(texts(&b, &joined.book_id).len(), 120);
}

#[test]
fn locked_notes_are_refused_or_moved_to_a_private_notebook() {
    let server = MemoryCollab::new();
    let a = device(&server, 1, 1);
    let (book, _) = notebook(
        &a.store,
        "Studio",
        &[
            ("One".to_owned(), "1".to_owned()),
            ("Two".to_owned(), "2".to_owned()),
        ],
    );
    for id in ["note:l1", "note:l2"] {
        a.store
            .lock_note(json!({
                "_id": id,
                "bookId": book,
                "title": "",
                "body": "",
                "kansoLock": {"v": 1, "salt": "AAAA", "nonce": "AAAA", "sealed": "AAAA"}
            }))
            .unwrap();
    }
    let check = a.engine.check_book_shareable(&book).unwrap();
    assert_eq!((check.eligible.len(), check.locked.len()), (2, 2));
    let err = a
        .engine
        .share_book(&book, &[], false, &mut |_| {})
        .unwrap_err();
    assert!(matches!(err, CollabError::NotShareable(_)), "{err:?}");
    assert!(a.store.shared_books().unwrap().is_empty());
    let done = a.engine.share_book(&book, &[], true, &mut |_| {}).unwrap();
    assert_eq!((done.shared, done.moved_locked), (2, 2));
    let private = a
        .store
        .books()
        .unwrap()
        .into_iter()
        .find(|k| k.name == "Studio (locked)")
        .unwrap();
    for id in ["note:l1", "note:l2"] {
        assert_eq!(a.store.note(id).unwrap().unwrap().book_id, private.id);
    }
    // A shared note can't be locked afterwards.
    let id = a
        .store
        .notes_in_books(std::slice::from_ref(&book))
        .unwrap()
        .remove(0)
        .id;
    assert!(
        a.store
            .lock_note(json!({
                "_id": id,
                "kansoLock": {"v": 1, "salt": "AAAA", "nonce": "AAAA", "sealed": "AAAA"}
            }))
            .is_err()
    );
    // Sharing it again is refused.
    assert!(a.engine.check_book_shareable(&book).is_err());
}

#[test]
fn two_people_edit_text_and_properties_and_both_changes_land() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let brief = "---\nstage: Idea\ndue: 2026-01-01\n---\nBrief\n";
    let (book, _) = notebook(
        &a.store,
        "Studio",
        &[("Video".to_owned(), brief.to_owned())],
    );
    let share = shared(&a, &[(&b, Role::Edit)], &book);
    let b_book = b.store.shared_book(&share).unwrap().unwrap().book_id;
    let a_id = note_of(&a, &book, "Video");
    let b_id = note_of(&b, &b_book, "Video");
    assert_eq!(body(&b, &b_id), brief);
    // Offline, A moves the card and B sets a date and adds a line.
    write(
        &a,
        &a_id,
        "---\nstage: Shooting\ndue: 2026-01-01\n---\nBrief\n",
    );
    write(
        &b,
        &b_id,
        "---\nstage: Idea\ndue: 2026-02-02\n---\nBrief\nShot list\n",
    );
    sync(&[&a, &b], &share);
    let want = "---\nstage: Shooting\ndue: 2026-02-02\n---\nBrief\nShot list\n";
    assert_eq!(body(&a, &a_id), want);
    assert_eq!(body(&b, &b_id), want);
    // The properties are entries of the note's document, not text.
    let loaded =
        crate::doc::CollabDoc::load(&a.store.book_stream_of_note(&a_id).unwrap().unwrap().state)
            .unwrap();
    assert_eq!(loaded.props().len(), 2);
    assert_eq!(loaded.text(crate::doc::Field::Body), "Brief\nShot list\n");
    // Both moved the same card: one stage, whole.
    write(&a, &a_id, &want.replace("Shooting", "Editing"));
    write(&b, &b_id, &want.replace("Shooting", "Review"));
    sync(&[&a, &b], &share);
    let (ta, tb) = (body(&a, &a_id), body(&b, &b_id));
    assert_eq!(ta, tb);
    assert!(
        ta.contains("stage: Editing\n") || ta.contains("stage: Review\n"),
        "{ta:?}"
    );
}

#[test]
fn notes_made_in_a_shared_notebook_reach_everyone_and_a_deleted_note_comes_back_with_an_edit() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let (book, _) = notebook(
        &a.store,
        "Studio",
        &[("First".to_owned(), "one".to_owned())],
    );
    let share = shared(&a, &[(&b, Role::Edit)], &book);
    let b_book = b.store.shared_book(&share).unwrap().unwrap().book_id;
    // B makes a note in the notebook.
    b.store
        .put(
            DocType::Note,
            json!({"_id": "note:fresh", "bookId": b_book, "title": "Fresh", "body": "new"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    sync(&[&b, &a], &share);
    assert_eq!(
        texts(&a, &book).get("Fresh").map(String::as_str),
        Some("new")
    );
    // A deletes it; B, who didn't, keeps it; B's edit brings it back to A.
    let a_fresh = note_of(&a, &book, "Fresh");
    a.store.delete(&a_fresh, Origin::Ui).unwrap();
    sync(&[&a, &b], &share);
    assert!(!texts(&a, &book).contains_key("Fresh"));
    assert!(body(&b, "note:fresh") == "new");
    write(&b, "note:fresh", "new and edited");
    sync(&[&b, &a], &share);
    sync(&[&a, &b], &share);
    assert_eq!(
        texts(&a, &book).get("Fresh").map(String::as_str),
        Some("new and edited")
    );
}

#[test]
fn the_owners_other_device_joins_and_a_late_device_catches_up() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let (book, _) = notebook(
        &a.store,
        "Studio",
        &[
            ("One".to_owned(), "1".to_owned()),
            ("Two".to_owned(), "2".to_owned()),
        ],
    );
    let share = shared(&a, &[(&b, Role::Edit)], &book);
    let a_id = note_of(&a, &book, "One");
    write(&a, &a_id, "1 edited");
    sync(&[&a], &share);
    let a2 = device(&server, 1, 2);
    let joined = a2.engine.join_book(&share).unwrap();
    assert_eq!(joined.notes, 2);
    assert_eq!(
        texts(&a2, &joined.book_id).get("One").map(String::as_str),
        Some("1 edited")
    );
    assert_eq!(a2.store.shared_book(&share).unwrap().unwrap().role, "owner");
    // Joining again just syncs.
    assert_eq!(a2.engine.join_book(&share).unwrap().book_id, joined.book_id);
}

#[test]
fn old_epochs_stay_readable_and_a_writer_an_epoch_behind_catches_up() {
    let server = MemoryCollab::new();
    let (a, b, c) = (
        device(&server, 1, 1),
        device(&server, 2, 1),
        device(&server, 3, 1),
    );
    let notes: Vec<(String, String)> = (0..4)
        .map(|i| (format!("Note {i}"), format!("Epoch one text {i}\n")))
        .collect();
    let (book, _) = notebook(&a.store, "Studio", &notes);
    let share = shared(&a, &[(&b, Role::Edit)], &book);
    // The owner starts epoch 2 with C in it (C has only that epoch's wrap).
    a.engine.person("user3").unwrap();
    post_list(
        &server,
        &a,
        &share,
        &[(&b, Role::Edit), (&c, Role::Edit)],
        true,
    );
    a.engine.sync_book(&share).unwrap();
    assert_eq!(a.store.shared_book(&share).unwrap().unwrap().epoch, 2);
    // C reads the notes written in epoch 1 through the history link.
    let joined = c.engine.join_book(&share).unwrap();
    assert_eq!(joined.notes, 4);
    let want: BTreeMap<String, String> = notes.iter().cloned().collect();
    assert_eq!(texts(&c, &joined.book_id), want);
    let keys: Vec<u32> = c
        .store
        .book_keys(&share)
        .unwrap()
        .iter()
        .map(|k| k.epoch)
        .collect();
    assert_eq!(keys, vec![1, 2]);
    // B hasn't synced since: its edit is sealed under epoch 1, the server
    // says the epoch changed, and the edit lands after B catches up.
    let b_book = b.store.shared_book(&share).unwrap().unwrap().book_id;
    let b_id = note_of(&b, &b_book, "Note 0");
    write(&b, &b_id, "Epoch one text 0\nfrom B\n");
    b.engine.sync_book(&share).unwrap();
    assert_eq!(b.store.shared_book(&share).unwrap().unwrap().epoch, 2);
    sync(&[&a, &c, &b], &share);
    let a_id = note_of(&a, &book, "Note 0");
    assert_eq!(body(&a, &a_id), "Epoch one text 0\nfrom B\n");
    let c_id = note_of(&c, &joined.book_id, "Note 0");
    assert_eq!(body(&c, &c_id), "Epoch one text 0\nfrom B\n");
}

#[test]
fn a_forked_chain_is_refused_and_an_older_chain_keeps_the_current_head() {
    let server = MemoryCollab::new();
    let (a, b, c) = (
        device(&server, 1, 1),
        device(&server, 2, 1),
        device(&server, 3, 1),
    );
    let (book, _) = notebook(&a.store, "Studio", &[("One".to_owned(), "1".to_owned())]);
    let share = shared(&a, &[(&b, Role::Edit)], &book);
    // Rev 2 adds C; B trusts it.
    a.engine.person("user3").unwrap();
    post_list(
        &server,
        &a,
        &share,
        &[(&b, Role::Edit), (&c, Role::View)],
        false,
    );
    b.engine.sync_book(&share).unwrap();
    let trusted = b.engine.book_ctx(&share).unwrap().chain.head;
    assert_eq!(trusted.list.rev, 2);
    // A server that rolls back to rev 1 isn't believed: B keeps rev 2 and
    // goes on.
    server.truncate_book_chain(&share, 1);
    let sync = b.engine.sync_book(&share).unwrap();
    assert!(!sync.removed);
    assert_eq!(b.engine.book_ctx(&share).unwrap().chain.head, trusted);
    // A different rev 2, one the owner's key signed after the same rev 1
    // (a fork), is a hard error that stays.
    let mut rival = trusted.list.clone();
    rival.at += 1;
    rival.members.retain(|m| m.role != Role::View);
    let signed = sign(&a.engine.identity().unwrap(), &rival);
    let hash = crate::book_members::link_hash(&crate::codec::unb64(&signed.list).unwrap());
    assert_ne!(hash, trusted.hash);
    server
        .new_book_list(
            a.engine.caller(),
            &share,
            &NewBookList {
                rev: 2,
                list: signed.list,
                list_sig: signed.sig,
                wraps: Vec::new(),
                history_link: None,
            },
        )
        .unwrap();
    for _ in 0..2 {
        let err = b.engine.sync_book(&share).unwrap_err();
        assert!(
            matches!(err, CollabError::BookList(BookListError::Forked)),
            "{err:?}"
        );
    }
    // The head it trusts is untouched.
    assert_eq!(b.engine.book_ctx(&share).unwrap().chain.head, trusted);
}

#[test]
fn a_list_the_owner_never_signed_is_refused_at_join() {
    let server = MemoryCollab::new();
    let (a, b, c) = (
        device(&server, 1, 1),
        device(&server, 2, 1),
        device(&server, 3, 1),
    );
    let (book, _) = notebook(&a.store, "Studio", &[("One".to_owned(), "1".to_owned())]);
    let share = shared(&a, &[(&b, Role::Edit)], &book);
    // The server swaps the genesis for one a stranger signed.
    let mut forged = a.engine.book_ctx(&share).unwrap().chain.head.list;
    forged.at += 5;
    let signed = sign(&c.engine.identity().unwrap(), &forged);
    server.replace_book_list(&share, 1, &signed.list, &signed.sig);
    let d = device(&server, 2, 2);
    let err = d.engine.join_book(&share).unwrap_err();
    assert!(
        matches!(err, CollabError::BookList(BookListError::BadSignature)),
        "{err:?}"
    );
    assert!(d.store.shared_book(&share).unwrap().is_none());
}

#[test]
fn a_member_whose_keys_arent_the_ones_their_account_published_stops_the_sync() {
    let server = MemoryCollab::new();
    let (a, b, c) = (
        device(&server, 1, 1),
        device(&server, 2, 1),
        device(&server, 3, 1),
    );
    let (book, _) = notebook(&a.store, "Studio", &[("One".to_owned(), "1".to_owned())]);
    let share = shared(&a, &[(&b, Role::Edit)], &book);
    // The owner's signed list names C with keys that aren't the ones C's
    // account published.
    // (A server that doesn't check, or is the one doing it.)
    server.skip_book_member_checks();
    let mut stranger = member_of(&c, Role::Edit);
    stranger.box_pub = b64(&a.engine.identity().unwrap().box_pub());
    post_members(
        &server,
        &a,
        &share,
        vec![
            member_of(&a, Role::Owner),
            member_of(&b, Role::Edit),
            stranger,
        ],
        false,
    );
    let err = b.engine.sync_book(&share).unwrap_err();
    assert!(matches!(err, CollabError::Crypto(_)), "{err:?}");
    assert_eq!(b.engine.book_ctx(&share).unwrap().chain.head.list.rev, 1);
    // A new device of B's refuses it at join as well.
    let d = device(&server, 2, 2);
    let err = d.engine.join_book(&share).unwrap_err();
    assert!(matches!(err, CollabError::Crypto(_)), "{err:?}");
    assert!(d.store.shared_book(&share).unwrap().is_none());
}

#[test]
fn entries_of_an_epoch_whose_key_is_missing_are_not_skipped_for_good() {
    let server = MemoryCollab::new();
    let (a, b, c) = (
        device(&server, 1, 1),
        device(&server, 2, 1),
        device(&server, 3, 1),
    );
    let notes: Vec<(String, String)> = (0..4)
        .map(|i| (format!("Note {i}"), format!("Epoch one text {i}\n")))
        .collect();
    let (book, _) = notebook(&a.store, "Studio", &notes);
    let share = shared(&a, &[(&b, Role::Edit)], &book);
    a.engine.person("user3").unwrap();
    post_list(
        &server,
        &a,
        &share,
        &[(&b, Role::Edit), (&c, Role::Edit)],
        true,
    );
    a.engine.sync_book(&share).unwrap();
    // The server can't give C the link to epoch 1 for now: C can't open the
    // notes, and says so, rather than taking them for forged.
    let links = server.take_book_history(&share);
    let err = c.engine.join_book(&share).unwrap_err();
    assert_eq!(err.code(), Some("epoch_changed"), "{err:?}");
    let row = c.store.shared_book(&share).unwrap().unwrap();
    // Nothing was passed over: when the link is back, everything is read.
    server.put_book_history(&share, links);
    c.engine.sync_book(&share).unwrap();
    let want: BTreeMap<String, String> = notes.into_iter().collect();
    assert_eq!(texts(&c, &row.book_id), want);
}

#[test]
fn the_server_cant_name_another_owner_than_the_genesis_list() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let (book, _) = notebook(&a.store, "Studio", &[("One".to_owned(), "1".to_owned())]);
    let share = shared(&a, &[(&b, Role::Edit)], &book);
    server.set_book_owner(&share, &b.engine.caller().account_id);
    let d = device(&server, 2, 2);
    let err = d.engine.join_book(&share).unwrap_err();
    assert!(
        matches!(err, CollabError::BookList(BookListError::Malformed)),
        "{err:?}"
    );
    assert!(d.store.shared_book(&share).unwrap().is_none());
}

#[test]
fn joining_again_after_removal_holds_the_chain_to_the_old_head() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let (book, _) = notebook(&a.store, "Studio", &[("One".to_owned(), "1".to_owned())]);
    let share = shared(&a, &[(&b, Role::Edit)], &book);
    let mut row = b.store.shared_book(&share).unwrap().unwrap();
    row.status = kanso_store::SharedStatus::Removed;
    b.store.put_shared_book(&row).unwrap();
    // A history the owner signed that isn't the one B saw: a fork at the
    // head B trusted.
    let trusted: crate::book_model::StoredChain = serde_json::from_str(&row.chain).unwrap();
    let mut rival = trusted.head.list;
    rival.at += 7;
    let signed = sign(&a.engine.identity().unwrap(), &rival);
    server.replace_book_list(&share, 1, &signed.list, &signed.sig);
    let err = b.engine.join_book(&share).unwrap_err();
    assert!(matches!(err, CollabError::BookList(_)), "{err:?}");
}

#[test]
fn a_resumed_share_leaves_out_notes_moved_deleted_or_locked_since() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let notes: Vec<(String, String)> = (0..120)
        .map(|i| (format!("Note {i}"), format!("Text {i}\n")))
        .collect();
    let (book, _) = notebook(&a.store, "Studio", &notes);
    a.engine.person("user2").unwrap();
    server.go_offline_after(80, 1);
    let err = a
        .engine
        .share_book(&book, &[("user2".into(), Role::Edit)], false, &mut |_| {})
        .unwrap_err();
    assert!(err.is_offline(), "{err:?}");
    let row = a.store.shared_book_of(&book).unwrap().unwrap();
    let pending: Vec<String> = a
        .store
        .book_streams(&row.share_id)
        .unwrap()
        .into_iter()
        .filter(|s| !s.on_server)
        .filter_map(|s| s.note_id)
        .take(3)
        .collect();
    assert_eq!(pending.len(), 3);
    // Since the crash: one moved to another notebook, one deleted, one locked.
    let other = a
        .store
        .put(
            DocType::Book,
            json!({"name": "Elsewhere"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    a.store
        .put(
            DocType::Note,
            json!({"_id": pending[0], "bookId": other}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    a.store.delete(&pending[1], Origin::Ui).unwrap();
    let rev = a.store.note(&pending[2]).unwrap().unwrap().rev;
    a.store
        .lock_note(json!({
            "_id": pending[2],
            "_rev": rev,
            "bookId": book,
            "title": "",
            "body": "",
            "kansoLock": {"v": 1, "salt": "AAAA", "nonce": "AAAA", "sealed": "AAAA"}
        }))
        .unwrap();
    a.engine
        .resume_book_share(&row.share_id, &mut |_| {})
        .unwrap();
    // Manifest and 117 notes are on the server; the three never went up.
    assert_eq!(server.book_room(&row.share_id).map(|r| r.1), Some(118));
    let moved = a.store.note(&pending[0]).unwrap().unwrap();
    assert!(!moved.extra.contains_key(kanso_core::model::SHARE_FIELD));
    assert_eq!(moved.book_id, other);
    let joined = b.engine.join_book(&row.share_id).unwrap();
    assert_eq!(joined.notes, 117);
}

#[test]
fn the_head_a_device_trusts_only_grows_with_what_it_received() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let notes: Vec<(String, String)> = (0..3)
        .map(|i| (format!("Note {i}"), format!("Text {i}\n")))
        .collect();
    let (book, _) = notebook(&a.store, "Studio", &notes);
    let share = shared(&a, &[(&b, Role::Edit)], &book);
    // A server that claims a head far beyond its log.
    server.inflate_book_head(&share, 1_000_000);
    sync(&[&a, &b], &share);
    for d in [&a, &b] {
        let held = d.engine.book_ctx(&share).unwrap().row.head;
        let received = d
            .store
            .book_streams(&share)
            .unwrap()
            .iter()
            .map(|s| s.applied_seq)
            .max()
            .unwrap();
        assert!(held <= received, "{held} > {received}");
        assert!(held < 1_000, "{held}");
    }
}

#[test]
fn a_notebook_note_is_never_taken_for_a_single_note_share() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let notes = [("One".to_owned(), "---\nstage: x\n---\n1\n".to_owned())];
    let (book, _) = notebook(&a.store, "Studio", &notes);
    let share = shared(&a, &[(&b, Role::Edit)], &book);
    let b_book = b.store.shared_book(&share).unwrap().unwrap().book_id;
    for (d, book) in [(&a, &book), (&b, &b_book)] {
        let id = note_of(d, book, "One");
        let note = d.store.note(&id).unwrap().unwrap();
        let mark = &note.extra[kanso_core::model::SHARE_FIELD];
        // The mark says it's a notebook's, and what carries a single note's
        // share (the lookup by share, the share table) doesn't answer for it.
        assert_eq!(mark["book"], true);
        assert_eq!(d.store.note_with_share(&share).unwrap(), None);
        assert!(d.store.shared_note(&share).unwrap().is_none());
        // The single-note engine leaves it as it is.
        d.engine.refresh().unwrap();
        d.engine.sync_all().unwrap();
        let after = d.store.note(&id).unwrap().unwrap();
        assert_eq!(after.extra[kanso_core::model::SHARE_FIELD], *mark);
        assert_eq!(after.body, note.body);
    }
}

#[test]
fn half_typed_front_matter_and_a_key_edited_elsewhere_both_land() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let start = "---\nstage: draft\ntags: x\n---\nbody\n";
    let (book, _) = notebook(&a.store, "Studio", &[("One".to_owned(), start.to_owned())]);
    let share = shared(&a, &[(&b, Role::Edit)], &book);
    let b_book = b.store.shared_book(&share).unwrap().unwrap().book_id;
    let a_id = note_of(&a, &book, "One");
    let b_id = note_of(&b, &b_book, "One");
    // A changes a key; B, offline, is part way through typing a new one.
    write(&a, &a_id, "---\nstage: done\ntags: x\n---\nbody\n");
    let typed = "---\nstage: draft\nsta\ntags: x\n---\nbody 2\n";
    write(&b, &b_id, typed);
    sync(&[&a, &b], &share);
    // Held back: B's note keeps what was typed (the body edit with it),
    // nobody's properties are touched, and A has no second block.
    assert_eq!(body(&b, &b_id), typed);
    assert_eq!(body(&a, &a_id), "---\nstage: done\ntags: x\n---\nbody\n");
    // More typed meanwhile, then the property is finished: everything lands.
    write(
        &b,
        &b_id,
        "---\nstage: draft\nsta\ntags: x\n---\nbody 2 more\n",
    );
    sync(&[&b], &share);
    assert_eq!(
        body(&b, &b_id),
        "---\nstage: draft\nsta\ntags: x\n---\nbody 2 more\n"
    );
    write(
        &b,
        &b_id,
        "---\nstage: draft\nsta: x\ntags: x\n---\nbody 2 more\n",
    );
    sync(&[&a, &b], &share);
    let want = "---\nstage: done\nsta: x\ntags: x\n---\nbody 2 more\n";
    assert_eq!(body(&a, &a_id), want);
    assert_eq!(body(&b, &b_id), want);
}

#[test]
fn a_member_that_couldnt_be_pinned_is_tried_again_at_every_sync() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let (book, _) = notebook(&a.store, "Studio", &[("One".to_owned(), "1".to_owned())]);
    let share = shared(&a, &[(&b, Role::Edit)], &book);
    // The owner lists someone whose account the server doesn't have (yet):
    // the keys are the ones that account will publish.
    let nine = device(&server, 9, 1);
    let later = member_of(&nine, Role::Edit);
    let published = server.take_identity(&later.account).unwrap();
    server.skip_book_member_checks();
    post_members(
        &server,
        &a,
        &share,
        vec![
            member_of(&a, Role::Owner),
            member_of(&b, Role::Edit),
            later.clone(),
        ],
        false,
    );
    b.engine.sync_book(&share).unwrap();
    assert_eq!(
        b.engine.unpinned_members(&share),
        vec![later.account.clone()]
    );
    // Nothing changed in the chain: still tried, still left.
    b.engine.sync_book(&share).unwrap();
    assert_eq!(
        b.engine.unpinned_members(&share),
        vec![later.account.clone()]
    );
    // Their account appears: the next sync pins them, the chain unchanged.
    server.put_identity(&later.account, published);
    b.engine.sync_book(&share).unwrap();
    assert!(b.engine.unpinned_members(&share).is_empty());
    assert!(b.store.contact(&later.account).unwrap().is_some());
}

proptest! {
    #![proptest_config(crate::tests::config(8))]

    /// Two devices that share, join, edit offline and sync in any order end
    /// with the same notes and text, and nothing a person wrote is lost.
    #[test]
    fn prop_two_devices_converge(
        ops in prop::collection::vec((any::<bool>(), 0usize..3, "[a-z ]{1,6}", any::<bool>()), 1..8),
        order in prop::collection::vec(any::<bool>(), 1..6),
    ) {
        let server = MemoryCollab::new();
        let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
        let notes: Vec<(String, String)> = (0..3)
            .map(|i| (format!("N{i}"), format!("---\nstage: s{i}\n---\nbody {i}\n")))
            .collect();
        let (book, _) = notebook(&a.store, "Studio", &notes);
        let share = shared(&a, &[(&b, Role::Edit)], &book);
        let b_book = b.store.shared_book(&share).unwrap().unwrap().book_id;
        let mut wrote = Vec::new();
        for (by_a, which, word, property) in &ops {
            let (d, k) = if *by_a { (&a, &book) } else { (&b, &b_book) };
            let id = note_of(d, k, &format!("N{which}"));
            let now = body(d, &id);
            let next = if *property {
                now.replacen("stage: ", &format!("stage: {word}"), 1)
            } else {
                format!("{now}{word}\n")
            };
            write(d, &id, &next);
            if !*property { wrote.push((*which, format!("{word}\n"))); }
        }
        for first_a in &order {
            let pair = if *first_a { [&a, &b] } else { [&b, &a] };
            for d in pair { d.engine.sync_book(&share).unwrap(); }
        }
        sync(&[&a, &b], &share);
        let (ta, tb) = (texts(&a, &book), texts(&b, &b_book));
        prop_assert_eq!(&ta, &tb);
        for (which, line) in wrote {
            let key = format!("N{which}");
            prop_assert!(ta[&key].contains(&line));
        }
        // Every property is whole: one stage line each.
        for text in ta.values() {
            prop_assert_eq!(text.matches("stage: ").count(), 1);
        }
    }
}
