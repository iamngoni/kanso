//! Taking people out of a shared notebook, leaving it and stopping it,
//! against the in-memory server that follows the Worker: the key moves on,
//! the one who is cut can't open what comes after, the private copy and the
//! words that go with each, and two members ticking the same repeating
//! task offline. Property tests run on the seed `PROPTEST_RNG_SEED` pins.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use kanso_core::dates::Date;
use kanso_core::tick::tick_task_at;
use kanso_store::SharedStatus;
use proptest::prelude::*;

use super::book::stream_key;
use super::book_tests::{body, note_of, notebook, shared, sync, texts, write};
use super::tests::{Device, device};
use crate::api::CollabError;
use crate::book::{open_history_link, unwrap};
use crate::book_copy;
use crate::book_model::{BookEndReason, BookKey};
use crate::codec::{unb64, unb64_array};
use crate::envelope::{self, parse_header};
use crate::memory::MemoryCollab;
use crate::model::Role;

fn account(d: &Device) -> String {
    d.engine.caller().account_id.clone()
}

/// The notebook keys a device holds now.
fn keys_of(d: &Device, share: &str) -> Vec<BookKey> {
    d.engine
        .book_ctx(share)
        .unwrap()
        .keys
        .into_values()
        .collect()
}

fn epoch_of(d: &Device, share: &str) -> u32 {
    d.store.shared_book(share).unwrap().unwrap().epoch
}

fn members_of(d: &Device, share: &str) -> Vec<String> {
    let ctx = d.engine.book_ctx(share).unwrap();
    let mut accounts: Vec<String> = ctx
        .chain
        .head
        .list
        .members
        .iter()
        .map(|m| m.account.clone())
        .collect();
    accounts.sort();
    accounts
}

/// How many of the server's entries written under `epoch` there are, and
/// how many of them open with one of `keys` (as the author's signature
/// checks, so only the key can be what fails). `reader` is a member whose
/// chain reaches that epoch.
fn opened_at(
    server: &MemoryCollab,
    reader: &Device,
    share: &str,
    epoch: u32,
    keys: &[BookKey],
) -> (usize, usize) {
    let ctx = reader.engine.book_ctx(share).unwrap();
    let (mut total, mut opened) = (0, 0);
    for (stream, base, bytes) in server.book_entries(share) {
        let Some((header, _)) = parse_header(&bytes) else {
            continue;
        };
        if header.epoch != epoch {
            continue;
        }
        total += 1;
        let Some(member) = ctx
            .chain
            .epochs
            .get(&epoch)
            .and_then(|list| list.member(&header.author_account))
        else {
            continue;
        };
        let sign = unb64_array::<32>(&member.sign_pub).unwrap();
        if keys.iter().any(|key| {
            envelope::open(&bytes, share, base, &sign, &stream_key(key, share, &stream)).is_ok()
        }) {
            opened += 1;
        }
    }
    (total, opened)
}

fn studio(server: &MemoryCollab, others: &[Role]) -> (Device, Vec<Device>, String, String) {
    let a = device(server, 1, 1);
    let rest: Vec<Device> = (0..others.len())
        .map(|i| device(server, 2 + i as u8, 1))
        .collect();
    let notes: Vec<(String, String)> = (0..3)
        .map(|i| (format!("Note {i}"), format!("one {i}\n")))
        .collect();
    let (book, _) = notebook(&a.store, "Studio", &notes);
    let pairs: Vec<(&Device, Role)> = rest.iter().zip(others.iter().copied()).collect();
    let share = shared(&a, &pairs, &book);
    let mut all: Vec<&Device> = vec![&a];
    all.extend(rest.iter());
    sync(&all, &share);
    (a, rest, share, book)
}

fn book_of(d: &Device, share: &str) -> String {
    d.store.shared_book(share).unwrap().unwrap().book_id
}

#[test]
fn a_removed_member_cannot_open_anything_written_after_the_removal() {
    let server = MemoryCollab::new();
    let (a, rest, share, book) = studio(&server, &[Role::Edit, Role::Edit]);
    let (b, c) = (&rest[0], &rest[1]);
    let held = keys_of(c, &share);
    assert_eq!(held.len(), 1);
    let c_identity = c.engine.identity().unwrap();
    a.engine.remove_book_member(&share, &account(c)).unwrap();
    assert_eq!(epoch_of(&a, &share), 2);
    assert_eq!(members_of(&a, &share), {
        let mut want = vec![account(&a), account(b)];
        want.sort();
        want
    });
    let id = note_of(&a, &book, "Note 0");
    write(&a, &id, "one 0\nafter the removal\n");
    sync(&[&a, b], &share);
    assert_eq!(epoch_of(b, &share), 2);
    assert_eq!(
        body(b, &note_of(b, &book_of(b, &share), "Note 0")),
        "one 0\nafter the removal\n"
    );
    // Everything the server holds under epoch 2 is closed to C, who has
    // every key of epoch 1; it opens with the key of epoch 2.
    let (total, opened) = opened_at(&server, &a, &share, 2, &held);
    assert!(total > 0);
    assert_eq!(opened, 0);
    let k2 = a.engine.book_ctx(&share).unwrap().keys[&2].clone();
    assert_eq!(
        opened_at(&server, &a, &share, 2, std::slice::from_ref(&k2)),
        (total, total)
    );
    // No wrap of epoch 2 is C's, none opens for C's identity, and the
    // history link needs epoch 2's key.
    let (wraps, links) = server.book_keying(&share);
    assert!(
        wraps
            .iter()
            .all(|(who, e, _)| !(*who == account(c) && *e == 2))
    );
    for (who, _, wrapped) in wraps.iter().filter(|(_, e, _)| *e == 2) {
        for role in Role::ALL {
            assert!(unwrap(&c_identity, wrapped, &share, 2, who, role).is_none());
        }
    }
    let link = unb64(&links.iter().find(|(e, _)| *e == 2).unwrap().1).unwrap();
    for key in &held {
        assert!(open_history_link(&share, 2, key, &link).is_none());
    }
    assert!(open_history_link(&share, 2, &k2, &link).is_some());
    // C's next sync finds the notebook gone and keeps what it had.
    let ended = c.engine.sync_book(&share).unwrap().ended.unwrap();
    assert_eq!(ended.reason, BookEndReason::Gone);
    assert_eq!(ended.copy_name, "Studio (no longer shared)");
    let c_book = book_of(c, &share);
    assert_eq!(ended.book_id, c_book);
    let kept = texts(c, &c_book);
    assert_eq!(kept.len(), 3);
    assert_eq!(kept["Note 0"], "one 0\n");
    for note in c
        .store
        .notes_in_books(std::slice::from_ref(&c_book))
        .unwrap()
    {
        assert!(!note.extra.contains_key(kanso_core::model::SHARE_FIELD));
    }
    assert_eq!(
        c.store
            .books()
            .unwrap()
            .into_iter()
            .find(|b| b.id == c_book)
            .unwrap()
            .name,
        "Studio (no longer shared)"
    );
    assert_eq!(
        c.store.shared_book(&share).unwrap().unwrap().status,
        SharedStatus::Removed
    );
    assert!(c.store.book_keys(&share).unwrap().is_empty());
}

#[test]
fn who_may_remove_whom() {
    let server = MemoryCollab::new();
    let (a, rest, share, _) = studio(&server, &[Role::Manager, Role::Manager, Role::Edit]);
    let (m, m2, e) = (&rest[0], &rest[1], &rest[2]);
    let refused = |r: Result<(), CollabError>| match r {
        Err(CollabError::Refused(why)) => why,
        other => panic!("not refused: {other:?}"),
    };
    // The owner stays, a manager can't remove a manager, an editor can't
    // remove anyone, and nobody removes themselves or a stranger.
    assert!(
        refused(m.engine.remove_book_member(&share, &account(&a)))
            .contains("owner can't be removed")
    );
    assert!(refused(m.engine.remove_book_member(&share, &account(m2))).contains("Only the owner"));
    assert!(
        refused(e.engine.remove_book_member(&share, &account(m))).contains("owner or a manager")
    );
    assert!(refused(m.engine.remove_book_member(&share, &account(m))).contains("leave"));
    assert!(refused(a.engine.remove_book_member(&share, "a_nobody")).contains("isn't in"));
    assert_eq!(epoch_of(&a, &share), 1);
    // A manager removes an editor; the owner removes a manager.
    m.engine.remove_book_member(&share, &account(e)).unwrap();
    assert_eq!(epoch_of(m, &share), 2);
    sync(&[&a, m, m2], &share);
    a.engine.remove_book_member(&share, &account(m2)).unwrap();
    sync(&[&a, m], &share);
    assert_eq!(epoch_of(m, &share), 3);
    assert_eq!(members_of(&a, &share), {
        let mut want = vec![account(&a), account(m)];
        want.sort();
        want
    });
}

#[test]
fn a_write_between_signing_and_sending_moves_the_head_and_the_list_is_signed_again() {
    let server = MemoryCollab::new();
    let (a, rest, share, book) = studio(&server, &[Role::Edit, Role::Edit]);
    let (b, c) = (&rest[0], &rest[1]);
    let b_book = book_of(b, &share);
    let b_note = note_of(b, &b_book, "Note 1");
    write(b, &b_note, "one 1\nfrom B\n");
    let ran = Arc::new(AtomicBool::new(false));
    {
        let (engine, flag, share) = (b.engine.clone(), ran.clone(), share.clone());
        server.before_book_list(move || {
            flag.store(true, Ordering::SeqCst);
            engine.sync_book(&share).unwrap();
        });
    }
    a.engine.remove_book_member(&share, &account(c)).unwrap();
    assert!(ran.load(Ordering::SeqCst));
    let list = a.engine.book_ctx(&share).unwrap().chain.head.list;
    assert_eq!(list.epoch, 2);
    assert_eq!(list.from_seq, Some(server.book_room(&share).unwrap().0));
    sync(&[&a, b], &share);
    assert_eq!(body(&a, &note_of(&a, &book, "Note 1")), "one 1\nfrom B\n");
    assert_eq!(texts(&a, &book), texts(b, &b_book));
}

#[test]
fn a_list_another_manager_sent_first_is_built_on_not_sent_over() {
    let server = MemoryCollab::new();
    let (a, rest, share, _) = studio(&server, &[Role::Manager, Role::Edit, Role::Edit]);
    let (m, e1, e2) = (&rest[0], &rest[1], &rest[2]);
    let ran = Arc::new(AtomicBool::new(false));
    {
        let (engine, flag, share, who) =
            (m.engine.clone(), ran.clone(), share.clone(), account(e2));
        server.before_book_list(move || {
            flag.store(true, Ordering::SeqCst);
            engine.remove_book_member(&share, &who).unwrap();
        });
    }
    a.engine.remove_book_member(&share, &account(e1)).unwrap();
    assert!(ran.load(Ordering::SeqCst));
    assert_eq!(epoch_of(&a, &share), 3);
    let mut want = vec![account(&a), account(m)];
    want.sort();
    assert_eq!(members_of(&a, &share), want);
    sync(&[&a, m], &share);
    assert_eq!(members_of(m, &share), want);
}

#[test]
fn a_listed_account_that_is_gone_is_cut_with_the_one_removed() {
    let server = MemoryCollab::new();
    let (a, rest, share, _) = studio(&server, &[Role::Edit, Role::Edit, Role::Edit]);
    let (b, c, d) = (&rest[0], &rest[1], &rest[2]);
    let published = server.take_identity(&account(d)).unwrap();
    a.engine.remove_book_member(&share, &account(c)).unwrap();
    let mut want = vec![account(&a), account(b)];
    want.sort();
    assert_eq!(members_of(&a, &share), want);
    assert_eq!(epoch_of(&a, &share), 2);
    server.put_identity(&account(d), published);
    sync(&[&a, b], &share);
    assert_eq!(members_of(b, &share), want);
}

#[test]
fn leaving_keeps_a_private_copy_and_the_owner_cuts_those_who_left() {
    let server = MemoryCollab::new();
    let (a, rest, share, _) = studio(&server, &[Role::Edit, Role::Edit]);
    let (b, c) = (&rest[0], &rest[1]);
    assert!(matches!(
        a.engine.leave_book(&share),
        Err(CollabError::Refused(_))
    ));
    let b_book = book_of(b, &share);
    let note = note_of(b, &b_book, "Note 2");
    write(b, &note, "one 2\nB's last word\n");
    let ended = b.engine.leave_book(&share).unwrap();
    assert_eq!(ended.reason, BookEndReason::Left);
    assert_eq!(
        ended.message,
        "You left \"Studio\". Your copy is in \"Studio (no longer shared)\"."
    );
    assert_eq!(ended.book_id, b_book);
    assert_eq!(ended.notes, 3);
    // The copy is B's: its text is what B had, and it is an ordinary note.
    assert_eq!(body(b, &note), "one 2\nB's last word\n");
    write(b, &note, "one 2\nprivate now\n");
    assert!(b.engine.sync_books().iter().all(|(id, _)| *id != share));
    assert!(
        !b.store
            .note(&note)
            .unwrap()
            .unwrap()
            .extra
            .contains_key(kanso_core::model::SHARE_FIELD)
    );
    // The signed list still names B until the owner cuts it, with a new key.
    assert!(members_of(&a, &share).contains(&account(b)));
    assert_eq!(a.engine.cut_left_members(&share).unwrap(), 1);
    assert!(!members_of(&a, &share).contains(&account(b)));
    assert_eq!(epoch_of(&a, &share), 2);
    assert_eq!(a.engine.cut_left_members(&share).unwrap(), 0);
    sync(&[&a, c], &share);
    assert_eq!(epoch_of(c, &share), 2);
    let (wraps, _) = server.book_keying(&share);
    assert!(
        wraps
            .iter()
            .all(|(who, e, _)| !(*who == account(b) && *e == 2))
    );
    // B's words from before it left reached the others.
    assert_eq!(
        body(&a, &note_of(&a, &book_of(&a, &share), "Note 2")),
        "one 2\nB's last word\n"
    );
}

#[test]
fn removing_someone_also_cuts_those_who_had_left() {
    let server = MemoryCollab::new();
    let (a, rest, share, _) = studio(&server, &[Role::Edit, Role::Edit, Role::Edit]);
    let (b, c, d) = (&rest[0], &rest[1], &rest[2]);
    b.engine.leave_book(&share).unwrap();
    a.engine.remove_book_member(&share, &account(c)).unwrap();
    let mut want = vec![account(&a), account(d)];
    want.sort();
    assert_eq!(members_of(&a, &share), want);
    assert_eq!(epoch_of(&a, &share), 2);
}

#[test]
fn stopping_leaves_everyone_a_private_copy() {
    let server = MemoryCollab::new();
    let (a, rest, share, book) = studio(&server, &[Role::Edit, Role::Edit]);
    let (b, c) = (&rest[0], &rest[1]);
    assert!(matches!(
        b.engine.stop_book(&share),
        Err(CollabError::Refused(_))
    ));
    let ended = a.engine.stop_book(&share).unwrap();
    assert_eq!(ended.reason, BookEndReason::Stopped);
    assert_eq!(
        ended.message,
        "\"Studio\" isn't shared any more. Everyone keeps a private copy."
    );
    // The owner's notebook is theirs again, under its own name.
    assert_eq!(ended.book_id, book);
    assert_eq!(texts(&a, &book).len(), 3);
    for note in a.store.notes_in_books(std::slice::from_ref(&book)).unwrap() {
        assert!(!note.extra.contains_key(kanso_core::model::SHARE_FIELD));
    }
    assert_eq!(
        a.store
            .books()
            .unwrap()
            .into_iter()
            .find(|b| b.id == book)
            .unwrap()
            .name,
        "Studio"
    );
    assert_eq!(
        a.store.shared_book(&share).unwrap().unwrap().status,
        SharedStatus::Stopped
    );
    assert!(server.book_room(&share).is_none());
    // Each member's next sync finds it gone and keeps its copy.
    for d in [b, c] {
        let before = book_of(d, &share);
        let ended = d.engine.sync_book(&share).unwrap().ended.unwrap();
        assert_eq!(ended.book_id, before);
        assert_eq!(ended.copy_name, "Studio (no longer shared)");
        assert_eq!(
            ended.message,
            "You can no longer open \"Studio\". Your copy is in \"Studio (no longer shared)\"."
        );
        assert_eq!(texts(d, &before).len(), 3);
        // Syncing again does nothing more, and the name isn't added to.
        assert!(d.engine.sync_books().is_empty());
        let name = d
            .store
            .books()
            .unwrap()
            .into_iter()
            .find(|b| b.id == before)
            .unwrap()
            .name;
        assert_eq!(name, "Studio (no longer shared)");
    }
}

#[test]
fn joining_a_notebook_again_makes_a_new_notebook_beside_the_copy() {
    let server = MemoryCollab::new();
    let (a, rest, share, _) = studio(&server, &[Role::Edit]);
    let b = &rest[0];
    let first = book_of(b, &share);
    b.engine.leave_book(&share).unwrap();
    // Invited again (the owner cut them, then listed them anew).
    a.engine.cut_left_members(&share).unwrap();
    super::book_tests::post_list(&server, &a, &share, &[(b, Role::Edit)], false);
    let joined = b.engine.join_book(&share).unwrap();
    assert_ne!(joined.book_id, first);
    assert_eq!(texts(b, &joined.book_id).len(), 3);
    assert_eq!(texts(b, &first).len(), 3);
}

#[test]
fn the_words_for_removing_leaving_and_stopping_are_these() {
    assert_eq!(
        book_copy::remove_title("Dan", "Studio"),
        "Remove Dan from \"Studio\"?"
    );
    assert_eq!(
        book_copy::remove_body("Dan"),
        "Dan keeps the notes they already have on their devices but sees nothing new. \
         Kanso changes the notebook's key now."
    );
    assert_eq!(
        book_copy::leave_title("Studio"),
        "Leave \"Studio\"? You'll keep a private copy."
    );
    assert_eq!(
        book_copy::ended_notice(BookEndReason::Removed, "Studio"),
        "You were removed from \"Studio\". Your copy is in \"Studio (no longer shared)\"."
    );
    assert_eq!(book_copy::copy_name("Studio"), "Studio (no longer shared)");
}

#[test]
fn two_members_ticking_the_same_repeating_task_offline_end_with_one_next_line() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let start = "- [ ] film Pixel review @2026-11-14T15:00 every week\n- [ ] other\n";
    let (book, _) = notebook(
        &a.store,
        "Studio",
        &[("Shoots".to_owned(), start.to_owned())],
    );
    let share = shared(&a, &[(&b, Role::Edit)], &book);
    sync(&[&a, &b], &share);
    let ticked = tick_task_at(start, 0, Date::new(2026, 11, 14).unwrap()).unwrap();
    let b_book = book_of(&b, &share);
    write(&a, &note_of(&a, &book, "Shoots"), &ticked);
    write(&b, &note_of(&b, &b_book, "Shoots"), &ticked);
    sync(&[&a, &b], &share);
    let (ta, tb) = (texts(&a, &book), texts(&b, &b_book));
    assert_eq!(ta, tb);
    let next = "- [ ] film Pixel review @2026-11-21T15:00 every week";
    assert_eq!(
        ta["Shoots"].lines().filter(|l| *l == next).count(),
        1,
        "{}",
        ta["Shoots"]
    );
    assert!(ta["Shoots"].contains("- [ ] other"));
}

#[test]
fn a_shared_note_loses_the_doubled_next_line_too() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let start = "- [ ] pay rent @2026-11-14 every month\n";
    let share = super::tests::shared(&a, &[(&b, Role::Edit)], start);
    let ticked = tick_task_at(start, 0, Date::new(2026, 11, 14).unwrap()).unwrap();
    let a_note = a.store.shared_note(&share).unwrap().unwrap().note_id;
    let b_note = b.store.shared_note(&share).unwrap().unwrap().note_id;
    super::tests::write_body(&a.store, &a_note, &ticked).unwrap();
    super::tests::write_body(&b.store, &b_note, &ticked).unwrap();
    super::tests::sync(&[&a, &b], &share);
    super::tests::sync(&[&a, &b], &share);
    let (ta, tb) = (
        super::tests::body_of(&a, &share),
        super::tests::body_of(&b, &share),
    );
    assert_eq!(ta, tb);
    let next = "- [ ] pay rent @2026-12-14 every month";
    assert_eq!(ta.lines().filter(|l| *l == next).count(), 1, "{ta}");
}

proptest! {
    #![proptest_config(crate::tests::config(6))]

    /// However many people are removed, one after another, with writes in
    /// between, nobody removed can open anything written after their own
    /// removal even holding every key they ever had, and those who stay
    /// converge on what everyone wrote.
    #[test]
    fn prop_removed_members_open_nothing_after_their_removal(
        victims in prop::sample::subsequence(vec![0usize, 1, 2], 1..=3),
        words in prop::collection::vec("[a-z]{1,5}", 4),
    ) {
        let server = MemoryCollab::new();
        let (a, rest, share, book) = studio(&server, &[Role::Edit; 4]);
        let keeper = &rest[3];
        let mut removed: Vec<(usize, u32, Vec<BookKey>)> = Vec::new();
        for (step, which) in victims.iter().enumerate() {
            let who = &rest[*which];
            let held = keys_of(who, &share);
            let epoch_before = epoch_of(&a, &share);
            let id = note_of(&a, &book, "Note 0");
            write(&a, &id, &format!("{}{}\n", body(&a, &id), words[step]));
            let kid = note_of(keeper, &book_of(keeper, &share), "Note 1");
            write(keeper, &kid, &format!("{}{}\n", body(keeper, &kid), words[step]));
            a.engine.remove_book_member(&share, &account(who)).unwrap();
            prop_assert_eq!(epoch_of(&a, &share), epoch_before + 1);
            // What the victim held, plus everything held back from it.
            removed.push((*which, epoch_before + 1, held));
            sync(&[&a, keeper], &share);
        }
        let id = note_of(&a, &book, "Note 2");
        write(&a, &id, "last\n");
        sync(&[&a, keeper], &share);
        let top = epoch_of(&a, &share);
        for (_, from, held) in &removed {
            for epoch in *from..=top {
                let (total, opened) = opened_at(&server, &a, &share, epoch, held);
                prop_assert_eq!(opened, 0, "epoch {} of {}: {} entries", epoch, top, total);
            }
        }
        prop_assert_eq!(texts(&a, &book), texts(keeper, &book_of(keeper, &share)));
    }

    /// Rotation with writes landing before, during and after it, from the
    /// owner and another member, in any order of syncs, converges.
    #[test]
    fn prop_rotation_under_concurrent_writes_converges(
        ops in prop::collection::vec((any::<bool>(), 0usize..3, "[a-z]{1,4}"), 1..7),
        remove_at in 0usize..7,
        order in prop::collection::vec(any::<bool>(), 1..5),
        during in any::<bool>(),
    ) {
        let server = MemoryCollab::new();
        let (a, rest, share, book) = studio(&server, &[Role::Edit, Role::Edit]);
        let (b, c) = (&rest[0], &rest[1]);
        let b_book = book_of(b, &share);
        let mut wrote = Vec::new();
        let mut removed = false;
        let do_remove = |removed: &mut bool| {
            if !*removed {
                *removed = true;
                a.engine.remove_book_member(&share, &account(c)).unwrap();
            }
        };
        for (i, (by_a, which, word)) in ops.iter().enumerate() {
            if i == remove_at && !during { do_remove(&mut removed); }
            let (d, k) = if *by_a { (&a, &book) } else { (b, &b_book) };
            let id = note_of(d, k, &format!("Note {which}"));
            write(d, &id, &format!("{}{}\n", body(d, &id), word));
            wrote.push((*which, format!("{word}\n")));
        }
        if during {
            let (engine, share2) = (b.engine.clone(), share.clone());
            server.before_book_list(move || { engine.sync_book(&share2).unwrap(); });
        }
        do_remove(&mut removed);
        for first_a in &order {
            let pair = if *first_a { [&a, b] } else { [b, &a] };
            for d in pair { d.engine.sync_book(&share).unwrap(); }
        }
        sync(&[&a, b], &share);
        let (ta, tb) = (texts(&a, &book), texts(b, &b_book));
        prop_assert_eq!(&ta, &tb);
        for (which, line) in wrote {
            let key = format!("Note {which}");
            prop_assert!(ta[&key].contains(&line));
        }
        prop_assert_eq!(epoch_of(&a, &share), 2);
        prop_assert_eq!(epoch_of(b, &share), 2);
    }
}
