//! Who is in a shared notebook, as the apps change and show it, against the
//! in-memory server that follows the Worker: inviting after sharing,
//! raising and lowering roles, declining, the member list, who last edited
//! each note (by the signed list's name) and what waits to be sent.
//! Property tests run on the seed `PROPTEST_RNG_SEED` pins.

use std::collections::BTreeMap;

use proptest::prelude::*;

use super::book_roster::{may_change_role, name_in_chain, ordered_members};
use super::book_tests::{member_of, note_of, notebook, shared, sync, write};
use super::tests::{Device, device};
use crate::api::CollabError;
use crate::book_members::{self, open_next};
use crate::book_members_model::BookMemberList;
use crate::book_model::MemberStanding;
use crate::memory::MemoryCollab;
use crate::model::Role;

fn account(d: &Device) -> String {
    d.engine.caller().account_id.clone()
}

fn epoch_of(d: &Device, share: &str) -> u32 {
    d.store.shared_book(share).unwrap().unwrap().epoch
}

fn role_in(d: &Device, share: &str, of: &Device) -> Option<Role> {
    d.engine
        .book_ctx(share)
        .unwrap()
        .chain
        .head
        .list
        .role_of(&account(of))
}

fn studio(owner: &Device) -> String {
    notebook(
        &owner.store,
        "Studio",
        &[
            ("Brief".into(), "Shoot on Friday".into()),
            ("Script".into(), "Scene one".into()),
        ],
    )
    .0
}

#[test]
fn an_invite_after_sharing_stays_in_the_epoch_and_lets_them_read() {
    let server = MemoryCollab::new();
    let (owner, a, b) = (
        device(&server, 1, 1),
        device(&server, 2, 1),
        device(&server, 3, 1),
    );
    let book = studio(&owner);
    let share = shared(&owner, &[(&a, Role::Edit)], &book);
    owner.engine.person("user3").unwrap();
    owner
        .engine
        .invite_to_book(&share, &[("user3".into(), Role::Comment)])
        .unwrap();
    assert_eq!(epoch_of(&owner, &share), 1);
    let invited = b.engine.book_invitations().unwrap();
    assert_eq!(invited.len(), 1);
    assert_eq!(invited[0].role, Role::Comment);
    let joined = b.engine.join_book(&share).unwrap();
    assert_eq!(joined.notes, 2);
    assert_eq!(role_in(&b, &share, &b), Some(Role::Comment));
    // Asking again for someone in it already is refused, in words.
    let again = owner
        .engine
        .invite_to_book(&share, &[("user2".into(), Role::View)]);
    assert!(matches!(again, Err(CollabError::Refused(words)) if words.contains("already")));
}

#[test]
fn a_manager_invites_below_manager_only() {
    let server = MemoryCollab::new();
    let (owner, m, c) = (
        device(&server, 1, 1),
        device(&server, 2, 1),
        device(&server, 3, 1),
    );
    let book = studio(&owner);
    let share = shared(&owner, &[(&m, Role::Manager)], &book);
    m.engine.person("user3").unwrap();
    let refused = m
        .engine
        .invite_to_book(&share, &[("user3".into(), Role::Manager)]);
    assert!(matches!(refused, Err(CollabError::Refused(_))));
    m.engine
        .invite_to_book(&share, &[("user3".into(), Role::Edit)])
        .unwrap();
    c.engine.join_book(&share).unwrap();
    assert_eq!(role_in(&c, &share, &c), Some(Role::Edit));
}

#[test]
fn raising_a_role_keeps_the_epoch_and_lowering_it_moves_it() {
    let server = MemoryCollab::new();
    let (owner, a) = (device(&server, 1, 1), device(&server, 2, 1));
    let book = studio(&owner);
    let share = shared(&owner, &[(&a, Role::View)], &book);
    owner
        .engine
        .set_book_role(&share, &account(&a), Role::Edit)
        .unwrap();
    assert_eq!(epoch_of(&owner, &share), 1);
    sync(&[&a], &share);
    assert_eq!(role_in(&a, &share, &a), Some(Role::Edit));
    // With the new role, A's edit reaches the owner.
    let note = note_of(&a, &book_of(&a, &share), "Brief");
    write(&a, &note, "Shoot on Saturday");
    sync(&[&a, &owner], &share);
    let mine = note_of(&owner, &book, "Brief");
    assert_eq!(
        owner.store.note(&mine).unwrap().unwrap().body,
        "Shoot on Saturday"
    );

    owner
        .engine
        .set_book_role(&share, &account(&a), Role::View)
        .unwrap();
    assert_eq!(epoch_of(&owner, &share), 2);
    sync(&[&a], &share);
    assert_eq!(role_in(&a, &share, &a), Some(Role::View));
    assert_eq!(epoch_of(&a, &share), 2);
}

#[test]
fn roles_change_only_as_the_rank_rules_allow() {
    let server = MemoryCollab::new();
    let (owner, m, e) = (
        device(&server, 1, 1),
        device(&server, 2, 1),
        device(&server, 3, 1),
    );
    let book = studio(&owner);
    let share = shared(&owner, &[(&m, Role::Manager), (&e, Role::Edit)], &book);
    // A manager can't touch the owner, make a manager, or change their own.
    for (who, role) in [
        (account(&owner), Role::Edit),
        (account(&e), Role::Manager),
        (account(&m), Role::Edit),
    ] {
        let refused = m.engine.set_book_role(&share, &who, role);
        assert!(matches!(refused, Err(CollabError::Refused(_))), "{role:?}");
    }
    m.engine
        .set_book_role(&share, &account(&e), Role::Comment)
        .unwrap();
    // An editor can't change roles at all.
    sync(&[&e], &share);
    let refused = e.engine.set_book_role(&share, &account(&m), Role::View);
    assert!(matches!(refused, Err(CollabError::Refused(_))));
}

#[test]
fn a_declined_invitation_shows_as_left_and_is_cut() {
    let server = MemoryCollab::new();
    let (owner, a) = (device(&server, 1, 1), device(&server, 2, 1));
    let book = studio(&owner);
    owner.engine.person("user2").unwrap();
    let share = owner
        .engine
        .share_book(&book, &[("user2".into(), Role::Edit)], false, &mut |_| {})
        .unwrap()
        .share_id;
    let view = owner.engine.book_view_fresh(&share).unwrap();
    let them = view.members.iter().find(|m| !m.me).unwrap();
    assert_eq!(them.standing, MemberStanding::Invited);
    a.engine.decline_book(&share).unwrap();
    assert!(a.engine.book_invitations().unwrap().is_empty());
    let view = owner.engine.book_view_fresh(&share).unwrap();
    let them = view.members.iter().find(|m| !m.me).unwrap();
    assert_eq!(them.standing, MemberStanding::Left);
    assert_eq!(owner.engine.cut_left_members(&share).unwrap(), 1);
    let view = owner.engine.book_view(&share).unwrap();
    assert_eq!(view.members.len(), 1);
    assert_eq!(view.my_role, Some(Role::Owner));
    assert_eq!(view.name, "Studio");
}

#[test]
fn edited_by_names_the_author_from_the_signed_list() {
    let server = MemoryCollab::new();
    let (owner, a) = (device(&server, 1, 1), device(&server, 2, 1));
    let book = studio(&owner);
    let share = shared(&owner, &[(&a, Role::Edit)], &book);
    let theirs = note_of(&a, &book_of(&a, &share), "Script");
    write(&a, &theirs, "Scene one, take two");
    sync(&[&a, &owner], &share);
    let mine = note_of(&owner, &book, "Script");
    let edits = owner.engine.book_edits();
    let edit = edits.iter().find(|e| e.note_id == mine).unwrap();
    assert_eq!(edit.name, "user2");
    assert_eq!(edit.account, account(&a));
    assert!(!edit.mine);
    // On A's device the same edit is A's own.
    let on_a = a.engine.book_edits();
    assert!(on_a.iter().any(|e| e.note_id == theirs && e.mine));
    // The owner edits next: the owner's device says so.
    write(&owner, &mine, "Scene one, take three");
    sync(&[&owner, &a], &share);
    let edit = a
        .engine
        .book_edits()
        .into_iter()
        .find(|e| e.note_id == theirs)
        .unwrap();
    assert_eq!(edit.name, "user1");
}

#[test]
fn changes_wait_until_a_sync_sends_them() {
    let server = MemoryCollab::new();
    let (owner, a) = (device(&server, 1, 1), device(&server, 2, 1));
    let book = studio(&owner);
    let share = shared(&owner, &[(&a, Role::Edit)], &book);
    assert_eq!(owner.engine.book_waiting(&share).unwrap(), 0);
    let brief = note_of(&owner, &book, "Brief");
    let script = note_of(&owner, &book, "Script");
    write(&owner, &brief, "Shoot on Sunday");
    write(&owner, &script, "Scene two");
    assert_eq!(owner.engine.book_waiting(&share).unwrap(), 2);
    owner.engine.sync_book(&share).unwrap();
    assert_eq!(owner.engine.book_waiting(&share).unwrap(), 0);
}

/// The notebook a joined member holds `share` in.
fn book_of(d: &Device, share: &str) -> String {
    d.store.shared_book(share).unwrap().unwrap().book_id
}

const ROLES: [Role; 5] = [
    Role::View,
    Role::Comment,
    Role::Edit,
    Role::Manager,
    Role::Owner,
];

proptest! {
    // Each case makes accounts and a notebook on the in-memory server.
    #![proptest_config(crate::tests::config(40))]

    /// `may_change_role` says yes exactly when the chain's own rules take
    /// the list that makes the change: what the apps offer is what every
    /// member will accept.
    #[test]
    fn prop_may_change_role_agrees_with_the_chain(
        mine in prop::sample::select(vec![Role::Manager, Role::Owner]),
        from in prop::sample::select(ROLES.to_vec()),
        to in prop::sample::select(ROLES.to_vec()),
    ) {
        prop_assume!(from != to);
        let server = MemoryCollab::new();
        let (owner, signer, them) = (
            device(&server, 1, 1),
            device(&server, 2, 1),
            device(&server, 3, 1),
        );
        // The target can't be a second owner; an owner target is the owner.
        let target = if from == Role::Owner { &owner } else { &them };
        let signer = if mine == Role::Owner { &owner } else { &signer };
        let mut members = vec![member_of(&owner, Role::Owner)];
        if mine == Role::Manager {
            members.push(member_of(signer, Role::Manager));
        }
        if from != Role::Owner {
            members.push(member_of(&them, from));
        }
        let mut genesis = BookMemberList {
            v: 1,
            share: "s".repeat(22),
            rev: 1,
            epoch: 1,
            owner: account(&owner),
            signer: account(&owner),
            prev: None,
            key_commit: "a".repeat(64),
            at: 1,
            from_seq: None,
            members,
        };
        genesis.sort();
        let identity = owner.engine.identity().unwrap();
        let head = book_members::open_genesis(
            &book_members::sign(&identity, &genesis),
            &genesis.share,
            &identity.sign_pub(),
        )
        .unwrap();
        let lowered = to < from;
        let mut next = BookMemberList {
            rev: 2,
            signer: account(signer),
            prev: Some(head.hash.clone()),
            epoch: if lowered { 2 } else { 1 },
            key_commit: if lowered { "b".repeat(64) } else { "a".repeat(64) },
            from_seq: if lowered { Some(0) } else { None },
            members: genesis
                .members
                .iter()
                .map(|m| {
                    let mut m = m.clone();
                    if m.account == account(target) {
                        m.role = to;
                    }
                    m
                })
                .collect(),
            ..genesis.clone()
        };
        next.sort();
        let signed = book_members::sign(&signer.engine.identity().unwrap(), &next);
        let taken = open_next(&head, &signed, 0).is_ok();
        prop_assert_eq!(may_change_role(mine, from, to), taken);
    }

    /// The member list shows everyone once, the owner first and roles
    /// never rising down the list; this account is marked, and a standing
    /// shows only where the server gave one (the owner is always in).
    #[test]
    fn prop_members_are_ordered(
        roles in prop::collection::vec(prop::sample::select(vec![Role::View, Role::Comment, Role::Edit, Role::Manager]), 0..8),
        names in prop::collection::vec("[a-z]{1,6}", 8),
        known in prop::collection::vec(any::<bool>(), 8),
        me in 0usize..9,
    ) {
        let server = MemoryCollab::new();
        let owner = device(&server, 1, 1);
        let mut list = BookMemberList {
            v: 1,
            share: "s".repeat(22),
            rev: 1,
            epoch: 1,
            owner: account(&owner),
            signer: account(&owner),
            prev: None,
            key_commit: "a".repeat(64),
            at: 1,
            from_seq: None,
            members: vec![member_of(&owner, Role::Owner)],
        };
        let mut standing = BTreeMap::new();
        for (i, role) in roles.iter().enumerate() {
            let mut m = member_of(&owner, *role);
            m.account = format!("a_{i:032}");
            m.username = names[i].clone();
            if known[i] {
                standing.insert(m.account.clone(), MemberStanding::Invited);
            }
            list.members.push(m);
        }
        let me_account = list.members.get(me).map(|m| m.account.clone()).unwrap_or_default();
        let shown = ordered_members(&list, &me_account, &standing);
        prop_assert_eq!(shown.len(), list.members.len());
        prop_assert_eq!(shown[0].role, Role::Owner);
        prop_assert_eq!(shown[0].standing, MemberStanding::Member);
        for pair in shown.windows(2) {
            prop_assert!(pair[0].role >= pair[1].role);
        }
        for view in &shown {
            prop_assert_eq!(view.me, view.account == me_account);
            if view.role != Role::Owner {
                let expected = standing.get(&view.account).copied().unwrap_or(MemberStanding::Unknown);
                prop_assert_eq!(view.standing, expected);
            }
        }
    }

}

proptest! {
    #![proptest_config(crate::tests::config(12))]

    /// A name comes only from a signed list: the newest that names the
    /// account, and nothing for an account none names.
    #[test]
    fn prop_names_come_from_the_chain(renamed in "[a-z]{1,8}", stranger in "[a-z]{1,8}") {
        let server = MemoryCollab::new();
        let (owner, a) = (device(&server, 1, 1), device(&server, 2, 1));
        let book = studio(&owner);
        let share = shared(&owner, &[(&a, Role::Edit)], &book);
        let mut chain = owner.engine.book_ctx(&share).unwrap().chain;
        prop_assert_eq!(name_in_chain(&chain, &account(&a)), Some("user2".to_owned()));
        prop_assert_eq!(name_in_chain(&chain, &format!("a_{stranger}")), None);
        // Cut from the head, still named by the epoch they were in.
        let mut old = chain.head.list.clone();
        for m in &mut old.members {
            if m.account == account(&a) {
                m.username = renamed.clone();
            }
        }
        chain.epochs.insert(1, old);
        chain.head.list.members.retain(|m| m.account != account(&a));
        prop_assert_eq!(name_in_chain(&chain, &account(&a)), Some(renamed));
    }
}
