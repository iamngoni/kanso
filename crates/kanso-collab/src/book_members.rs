//! The notebook's signed member list, as a chain (issue #390 › Shared
//! notebooks: keys). The owner signs the first list. Each later list names
//! the hash of the one before it and is signed by the owner, or by a
//! manager that previous list names, changing only people below Manager.
//! Clients check the whole chain back to the owner's key, so the server,
//! which only stores and relays it, cannot forge a membership.
//!
//! Signatures are the identity's Ed25519 ([`crate::identity`]) over a
//! domain-separated message, `"kanso-book-members" || 0x01 || lp(share) ||
//! list bytes`, which no note member list or other signature of this crate
//! shares. A chain link is named by `hex(SHA-256("kanso-book-members-link"
//! || 0x01 || list bytes))`.
//!
//! Ownership does not move along a chain: the owner is the genesis list's
//! owner throughout.

use std::collections::BTreeMap;

use crate::book_members_model::{
    BookChainHead, BookListError, BookMemberList, BookPin, SignedBookList,
};
use crate::codec::{FORMAT_V1, b64, hex, put_lp, sha256, unb64, unb64_array};
use crate::identity::{Identity, verify};
use crate::model::{Member, Role};

/// At most this many people in a notebook, its owner included.
pub const MAX_BOOK_MEMBERS: usize = 25;

/// How far a new epoch's `from_seq` may jump past the previous epoch's. A
/// list that names a log position far beyond any real log would make
/// clients drop every later write; clients that know the log's head pass it
/// in as well, which tightens this.
pub const MAX_SEQ_JUMP: u64 = 1_000_000;

fn signed_message(share_id: &str, list_bytes: &[u8]) -> Vec<u8> {
    let mut message = b"kanso-book-members".to_vec();
    message.push(FORMAT_V1);
    put_lp(&mut message, share_id.as_bytes());
    message.extend_from_slice(list_bytes);
    message
}

fn link_hash(list_bytes: &[u8]) -> String {
    hex(&sha256(&[
        b"kanso-book-members-link",
        &[FORMAT_V1],
        list_bytes,
    ]))
}

/// A list's shape rules: members sorted by account and unique, exactly one
/// owner who is the list's `owner`, at most [`MAX_BOOK_MEMBERS`], well-formed
/// keys, and the chain fields that go with its position.
pub fn is_well_formed(list: &BookMemberList) -> bool {
    let sorted = list
        .members
        .windows(2)
        .all(|pair| pair[0].account < pair[1].account);
    let owners: Vec<&Member> = list
        .members
        .iter()
        .filter(|m| m.role == Role::Owner)
        .collect();
    list.v == 1
        && !list.share.is_empty()
        && list.rev >= 1
        && (list.rev == 1) == list.prev.is_none()
        && list.prev.as_ref().is_none_or(|p| p.len() == 64)
        && list.epoch >= 1
        && (list.epoch == 1) == list.from_seq.is_none()
        && sorted
        && list.members.len() <= MAX_BOOK_MEMBERS
        && owners.len() == 1
        && owners[0].account == list.owner
        && list.key_commit.len() == 64
        && list.members.iter().all(|m| {
            unb64_array::<32>(&m.sign_pub).is_some() && unb64_array::<32>(&m.box_pub).is_some()
        })
}

/// Signs `list` as `signer`. The list's `signer` field must name the
/// signer's account; nothing here checks the list against a chain.
pub fn sign(signer: &Identity, list: &BookMemberList) -> SignedBookList {
    let bytes = serde_json::to_vec(list).expect("a member list serializes");
    let signature = signer.sign(&signed_message(&list.share, &bytes));
    SignedBookList {
        list: b64(&bytes),
        sig: b64(&signature),
    }
}

struct Opened {
    list: BookMemberList,
    bytes: Vec<u8>,
    signature: [u8; 64],
}

fn parse(signed: &SignedBookList) -> Result<Opened, BookListError> {
    let bytes = unb64(&signed.list).ok_or(BookListError::Malformed)?;
    let signature = unb64_array::<64>(&signed.sig).ok_or(BookListError::Malformed)?;
    let list: BookMemberList =
        serde_json::from_slice(&bytes).map_err(|_| BookListError::Malformed)?;
    if !is_well_formed(&list) {
        return Err(BookListError::Malformed);
    }
    Ok(Opened {
        list,
        bytes,
        signature,
    })
}

fn check_signature(opened: &Opened, sign_pub: &str) -> Result<(), BookListError> {
    let key = unb64_array::<32>(sign_pub).ok_or(BookListError::BadSignature)?;
    let message = signed_message(&opened.list.share, &opened.bytes);
    if verify(&key, &message, &opened.signature) {
        Ok(())
    } else {
        Err(BookListError::BadSignature)
    }
}

/// Checks the owner's genesis list for `share_id` against the owner's
/// pinned signing key.
pub fn open_genesis(
    signed: &SignedBookList,
    share_id: &str,
    owner_sign_pub: &[u8; 32],
) -> Result<BookChainHead, BookListError> {
    let opened = parse(signed)?;
    let list = &opened.list;
    if list.share != share_id {
        return Err(BookListError::WrongShare);
    }
    if list.rev != 1 || list.epoch != 1 {
        return Err(BookListError::Broken);
    }
    if list.signer != list.owner {
        return Err(BookListError::NotAllowed);
    }
    let owner = list
        .members
        .iter()
        .find(|m| m.role == Role::Owner)
        .ok_or(BookListError::Malformed)?;
    if unb64_array::<32>(&owner.sign_pub) != Some(*owner_sign_pub) {
        return Err(BookListError::WrongOwner);
    }
    check_signature(&opened, &owner.sign_pub)?;
    Ok(BookChainHead {
        hash: link_hash(&opened.bytes),
        list: opened.list,
    })
}

/// Checks `signed` as the list that follows `head`. `log_head` is the
/// newest log sequence the client has seen for the notebook, when it knows
/// one: no epoch may begin beyond it.
pub fn open_next(
    head: &BookChainHead,
    signed: &SignedBookList,
    log_head: Option<u64>,
) -> Result<BookChainHead, BookListError> {
    let prev = &head.list;
    let opened = parse(signed)?;
    let next = &opened.list;
    if next.share != prev.share {
        return Err(BookListError::WrongShare);
    }
    if next.owner != prev.owner {
        return Err(BookListError::WrongOwner);
    }
    // A different list in the place of the head, off the same parent: two
    // signers each extended the chain. Honest clients never see this, so
    // it is a hard error, not a reason to pick one.
    if next.rev == prev.rev && next.prev == prev.prev && link_hash(&opened.bytes) != head.hash {
        return Err(BookListError::Forked);
    }
    if next.rev != prev.rev.saturating_add(1) || next.prev.as_deref() != Some(head.hash.as_str()) {
        return Err(BookListError::Broken);
    }
    // The signer is named by the list before this one, as owner or manager.
    let signer = prev
        .members
        .iter()
        .find(|m| m.account == next.signer)
        .filter(|m| m.role.may_manage())
        .ok_or(BookListError::NotAllowed)?;
    check_signature(&opened, &signer.sign_pub)?;

    let cuts = rank_rules(prev, next, signer.role)?;
    // A removal or demotion starts a new epoch (the person it cuts still
    // holds the old key). A list in the same epoch keeps its commitment and
    // log position; a new epoch swaps the commitment and never moves the
    // log position back.
    let same_epoch = next.epoch == prev.epoch;
    let new_epoch = next.epoch == prev.epoch.saturating_add(1);
    let epoch_ok = if same_epoch {
        !cuts && next.key_commit == prev.key_commit && next.from_seq == prev.from_seq
    } else {
        let base = prev.from_seq.unwrap_or(0);
        let bound = log_head.map_or(u64::MAX, |h| h.max(base));
        new_epoch
            && next.key_commit != prev.key_commit
            && next.from_seq >= prev.from_seq
            && next
                .from_seq
                .is_some_and(|f| f <= bound && f.saturating_sub(base) <= MAX_SEQ_JUMP)
    };
    if !epoch_ok {
        return Err(BookListError::BadEpoch);
    }
    Ok(BookChainHead {
        hash: link_hash(&opened.bytes),
        list: opened.list,
    })
}

/// What a signer of `signer_role` may change between `prev` and `next`.
/// Returns whether anyone was removed or demoted, in which case the list
/// must start a new epoch. Anyone who stays keeps their username and both
/// keys: only a role changes. A new key is a removal and a fresh addition.
fn rank_rules(
    prev: &BookMemberList,
    next: &BookMemberList,
    signer_role: Role,
) -> Result<bool, BookListError> {
    let old: BTreeMap<&str, &Member> = prev
        .members
        .iter()
        .map(|m| (m.account.as_str(), m))
        .collect();
    let new: BTreeMap<&str, &Member> = next
        .members
        .iter()
        .map(|m| (m.account.as_str(), m))
        .collect();
    // Nobody changes the owner's entry; ownership does not move in a chain.
    let owner = prev.owner.as_str();
    if old.get(owner) != new.get(owner) {
        return Err(BookListError::RankViolation);
    }
    let mut cuts = false;
    for account in old.keys().chain(new.keys()) {
        let (before, after) = (old.get(account), new.get(account));
        if before == after {
            continue;
        }
        if let (Some(b), Some(a)) = (before, after)
            && (a.sign_pub != b.sign_pub || a.box_pub != b.box_pub || a.username != b.username)
        {
            return Err(BookListError::RankViolation);
        }
        // A manager's changes stop below Manager, on both sides: they can't
        // touch a manager's entry or give anyone that role.
        if signer_role != Role::Owner
            && (before.is_some_and(|m| m.role >= Role::Manager)
                || after.is_some_and(|m| m.role >= Role::Manager))
        {
            return Err(BookListError::RankViolation);
        }
        cuts |= match (before, after) {
            (Some(_), None) => true,
            (Some(b), Some(a)) => a.role < b.role,
            _ => false,
        };
    }
    Ok(cuts)
}

/// Checks a whole chain, genesis first, against the owner's pinned key and
/// returns its newest list. `pin` is the newest list the client has already
/// trusted ([`BookChainHead::pin`], kept per notebook): the chain must pass
/// through that exact list ([`BookListError::Forked`] if it holds another at
/// that revision) and may not end before it ([`BookListError::Stale`]).
pub fn verify_chain(
    chain: &[SignedBookList],
    share_id: &str,
    owner_sign_pub: &[u8; 32],
    pin: Option<&BookPin>,
    log_head: Option<u64>,
) -> Result<BookChainHead, BookListError> {
    let (first, rest) = chain.split_first().ok_or(BookListError::Empty)?;
    let mut head = open_genesis(first, share_id, owner_sign_pub)?;
    check_pin(&head, pin)?;
    for signed in rest {
        head = open_next(&head, signed, log_head)?;
        check_pin(&head, pin)?;
    }
    if pin.is_some_and(|p| head.list.rev < p.rev) {
        return Err(BookListError::Stale);
    }
    Ok(head)
}

fn check_pin(head: &BookChainHead, pin: Option<&BookPin>) -> Result<(), BookListError> {
    match pin {
        Some(p) if p.rev == head.list.rev && p.hash != head.hash => Err(BookListError::Forked),
        _ => Ok(()),
    }
}

/// Checks the lists after one a client already trusts.
pub fn extend_chain(
    trusted: &BookChainHead,
    rest: &[SignedBookList],
    log_head: Option<u64>,
) -> Result<BookChainHead, BookListError> {
    let mut head = trusted.clone();
    for signed in rest {
        head = open_next(&head, signed, log_head)?;
    }
    Ok(head)
}

impl BookChainHead {
    /// What a client keeps per notebook to recognize its history later.
    pub fn pin(&self) -> BookPin {
        BookPin {
            rev: self.list.rev,
            hash: self.hash.clone(),
        }
    }
}

impl BookMemberList {
    pub fn member(&self, account: &str) -> Option<&Member> {
        self.members.iter().find(|m| m.account == account)
    }

    pub fn role_of(&self, account: &str) -> Option<Role> {
        self.member(account).map(|m| m.role)
    }

    /// The accounts holding the Manager role.
    pub fn managers(&self) -> impl Iterator<Item = &str> {
        self.members
            .iter()
            .filter(|m| m.role == Role::Manager)
            .map(|m| m.account.as_str())
    }

    /// Sorts members by account, as lists must be.
    pub fn sort(&mut self) {
        self.members.sort_by(|a, b| a.account.cmp(&b.account));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::book_model::BookKey;
    use proptest::prelude::*;

    /// A fixed identity per account, so two notebooks can share an owner.
    fn ident(account: &str) -> Identity {
        Identity::from_secrets(
            sha256(&[b"sign", account.as_bytes()]),
            sha256(&[b"box", account.as_bytes()]),
        )
    }

    fn entry(account: &str, role: Role) -> Member {
        let identity = ident(account);
        Member {
            account: account.into(),
            username: format!("u{account}"),
            role,
            sign_pub: b64(&identity.sign_pub()),
            box_pub: b64(&identity.box_pub()),
        }
    }

    /// A notebook's chain, built the way clients write it: each step is
    /// signed by the owner or a manager, and an epoch starts whenever
    /// someone is cut.
    struct Studio {
        share: String,
        chain: Vec<SignedBookList>,
        heads: Vec<BookChainHead>,
        spawned: u32,
    }

    impl Studio {
        fn new(share: &str) -> Studio {
            let list = BookMemberList {
                v: 1,
                share: share.into(),
                rev: 1,
                epoch: 1,
                owner: "a_0".into(),
                signer: "a_0".into(),
                prev: None,
                key_commit: BookKey::generate().commit(share, 1),
                at: 1,
                from_seq: None,
                members: vec![entry("a_0", Role::Owner)],
            };
            let signed = sign(&ident("a_0"), &list);
            let head = open_genesis(&signed, share, &ident("a_0").sign_pub()).unwrap();
            Studio {
                share: share.into(),
                chain: vec![signed],
                heads: vec![head],
                spawned: 0,
            }
        }

        fn head(&self) -> &BookChainHead {
            self.heads.last().unwrap()
        }

        fn owner_pub(&self) -> [u8; 32] {
            ident("a_0").sign_pub()
        }

        /// The next list from the head, with the members changed and the
        /// chain fields set.
        fn next_list(&self, signer: &str, change: impl FnOnce(&mut Vec<Member>)) -> BookMemberList {
            let prev = &self.head().list;
            let mut members = prev.members.clone();
            change(&mut members);
            members.sort_by(|a, b| a.account.cmp(&b.account));
            let cuts = prev.members.iter().any(|b| {
                members
                    .iter()
                    .find(|a| a.account == b.account)
                    .is_none_or(|a| a.role < b.role || a.box_pub != b.box_pub)
            });
            let epoch = prev.epoch + u32::from(cuts);
            BookMemberList {
                v: 1,
                share: self.share.clone(),
                rev: prev.rev + 1,
                epoch,
                owner: prev.owner.clone(),
                signer: signer.into(),
                prev: Some(self.head().hash.clone()),
                key_commit: if cuts {
                    BookKey::generate().commit(&self.share, epoch)
                } else {
                    prev.key_commit.clone()
                },
                at: i64::from(prev.rev) + 1,
                from_seq: if cuts {
                    Some(u64::from(prev.rev))
                } else {
                    prev.from_seq
                },
                members,
            }
        }

        fn push_list(&mut self, signer: &str, list: &BookMemberList) -> Result<(), BookListError> {
            let signed = sign(&ident(signer), list);
            let head = open_next(self.head(), &signed, None)?;
            self.chain.push(signed);
            self.heads.push(head);
            Ok(())
        }

        fn push(
            &mut self,
            signer: &str,
            change: impl FnOnce(&mut Vec<Member>),
        ) -> Result<(), BookListError> {
            let list = self.next_list(signer, change);
            self.push_list(signer, &list)
        }

        fn add(&mut self, signer: &str, role: Role) -> Result<String, BookListError> {
            self.spawned += 1;
            let account = format!("a_{}", self.spawned);
            let member = entry(&account, role);
            self.push(signer, |m| m.push(member))?;
            Ok(account)
        }

        fn remove(&mut self, signer: &str, account: &str) -> Result<(), BookListError> {
            self.push(signer, |m| m.retain(|x| x.account != account))
        }

        fn set_role(
            &mut self,
            signer: &str,
            account: &str,
            role: Role,
        ) -> Result<(), BookListError> {
            self.push(signer, |m| {
                m.iter_mut().find(|x| x.account == account).unwrap().role = role;
            })
        }

        fn who(&self, role: Role) -> Vec<String> {
            self.head()
                .list
                .members
                .iter()
                .filter(|m| m.role == role)
                .map(|m| m.account.clone())
                .collect()
        }

        fn verify(&self) -> Result<BookChainHead, BookListError> {
            verify_chain(&self.chain, &self.share, &self.owner_pub(), None, None)
        }
    }

    /// One random, legal step: (kind, which, who signs).
    type Op = (u8, prop::sample::Index, prop::sample::Index);

    fn ops(min: usize, max: usize) -> impl Strategy<Value = Vec<Op>> {
        prop::collection::vec(
            (
                0u8..5,
                any::<prop::sample::Index>(),
                any::<prop::sample::Index>(),
            ),
            min..max,
        )
    }

    /// Applies `ops` as legal steps (skipping ones that don't apply).
    fn build(share: &str, ops: &[Op]) -> Studio {
        let mut studio = Studio::new(share);
        for (kind, which, who) in ops {
            let mut signers = vec!["a_0".to_string()];
            signers.extend(studio.who(Role::Manager));
            let any_signer = signers[who.index(signers.len())].clone();
            let others: Vec<(String, Role)> = studio
                .head()
                .list
                .members
                .iter()
                .filter(|m| m.role != Role::Owner)
                .map(|m| (m.account.clone(), m.role))
                .collect();
            let target = (!others.is_empty()).then(|| others[which.index(others.len())].clone());
            let full = studio.head().list.members.len() >= MAX_BOOK_MEMBERS;
            let low = [Role::View, Role::Comment, Role::Edit];
            let result = match (kind, target) {
                (0, _) if !full => studio.add(&any_signer, low[which.index(3)]).map(|_| ()),
                (1, _) if !full => studio.add("a_0", Role::Manager).map(|_| ()),
                (2, Some((t, role))) => {
                    let signer = if role == Role::Manager {
                        "a_0".into()
                    } else {
                        any_signer
                    };
                    studio.remove(&signer, &t)
                }
                (3, Some((t, role))) => {
                    let signer = if role == Role::Manager {
                        "a_0".into()
                    } else {
                        any_signer
                    };
                    studio.set_role(&signer, &t, low[who.index(3)])
                }
                (4, Some((t, _))) => studio.set_role("a_0", &t, Role::Manager),
                _ => Ok(()),
            };
            assert_eq!(result, Ok(()), "a legal step was refused");
        }
        studio
    }

    fn tamper(signed: &SignedBookList, at: prop::sample::Index) -> SignedBookList {
        let mut bytes = unb64(&signed.list).unwrap();
        let i = at.index(bytes.len());
        bytes[i] ^= 0x01;
        SignedBookList {
            list: b64(&bytes),
            sig: signed.sig.clone(),
        }
    }

    proptest! {
        #![proptest_config(crate::tests::config(24))]

        /// A chain of legal steps verifies, whole and from any trusted point,
        /// and its head is the last list.
        #[test]
        fn prop_valid_chains_verify(ops in ops(0, 16), from in any::<prop::sample::Index>()) {
            let studio = build("s", &ops);
            let head = studio.verify().unwrap();
            prop_assert_eq!(&head, studio.head());
            prop_assert!(head.list.members.len() <= MAX_BOOK_MEMBERS);
            let at = from.index(studio.chain.len());
            prop_assert_eq!(extend_chain(&studio.heads[at], &studio.chain[at + 1..], None).unwrap(), head);
        }

        /// A list signed by someone who is not owner or manager in the
        /// previous list is refused, whoever they claim to be.
        #[test]
        fn prop_non_manager_lists_are_refused(ops in ops(0, 10), pick in any::<prop::sample::Index>()) {
            let mut studio = build("s", &ops);
            let mut weak: Vec<String> = studio.head().list.members.iter()
                .filter(|m| !m.role.may_manage()).map(|m| m.account.clone()).collect();
            weak.push("a_outsider".into());
            let signer = weak[pick.index(weak.len())].clone();
            let extra = entry("a_new", Role::Edit);
            let list = studio.next_list(&signer, |m| m.push(extra));
            prop_assert_eq!(studio.push_list(&signer, &list), Err(BookListError::NotAllowed));
            // Claiming to be the owner doesn't help: the signature isn't theirs.
            let mut claimed = list.clone();
            claimed.signer = "a_0".into();
            prop_assert_eq!(studio.push_list(&signer, &claimed), Err(BookListError::BadSignature));
            let forged = sign(&ident("a_forger"), &claimed);
            prop_assert_eq!(open_next(studio.head(), &forged, None), Err(BookListError::BadSignature));
        }

        /// A changed list or signature anywhere in a chain breaks it, and a
        /// genesis signed by another key can't stand in for the owner's.
        #[test]
        fn prop_forged_chains_are_refused(ops in ops(0, 10), link in any::<prop::sample::Index>(), at in any::<prop::sample::Index>()) {
            let studio = build("s", &ops);
            let i = link.index(studio.chain.len());
            let mut chain = studio.chain.clone();
            chain[i] = tamper(&chain[i], at);
            prop_assert!(verify_chain(&chain, "s", &studio.owner_pub(), None, None).is_err());
            if studio.chain.len() > 1 {
                let mut chain = studio.chain.clone();
                chain[i].sig = chain[(i + 1) % chain.len()].sig.clone();
                prop_assert!(verify_chain(&chain, "s", &studio.owner_pub(), None, None).is_err());
            }
            let mut list = studio.heads[0].list.clone();
            list.members = vec![entry("a_0", Role::Owner)];
            let forged = sign(&ident("a_other"), &list);
            prop_assert_eq!(open_genesis(&forged, "s", &studio.owner_pub()), Err(BookListError::BadSignature));
        }

        /// Swapping any two lists breaks the chain.
        #[test]
        fn prop_reordered_chains_are_refused(ops in ops(2, 12), a in any::<prop::sample::Index>(), b in any::<prop::sample::Index>()) {
            let studio = build("s", &ops);
            let n = studio.chain.len();
            prop_assume!(n >= 3);
            let (i, j) = (a.index(n), b.index(n));
            prop_assume!(i != j);
            let mut chain = studio.chain.clone();
            chain.swap(i, j);
            prop_assert!(verify_chain(&chain, "s", &studio.owner_pub(), None, None).is_err());
        }

        /// A list played again is refused; so is a chain that stops short of
        /// what a client has seen, or skips a list.
        #[test]
        fn prop_replayed_and_truncated_chains_are_refused(ops in ops(2, 12), a in any::<prop::sample::Index>(), cut in any::<prop::sample::Index>()) {
            let studio = build("s", &ops);
            let n = studio.chain.len();
            prop_assume!(n >= 3);
            let mut replayed = studio.chain.clone();
            replayed.push(studio.chain[a.index(n)].clone());
            prop_assert_eq!(verify_chain(&replayed, "s", &studio.owner_pub(), None, None), Err(BookListError::Broken));
            let keep = 1 + cut.index(n - 1);
            let prefix = &studio.chain[..keep];
            prop_assert!(verify_chain(prefix, "s", &studio.owner_pub(), None, None).is_ok());
            prop_assert_eq!(verify_chain(prefix, "s", &studio.owner_pub(), Some(&studio.head().pin()), None), Err(BookListError::Stale));
            let mut skipped = studio.chain.clone();
            skipped.remove(1 + cut.index(n - 2));
            prop_assert!(verify_chain(&skipped, "s", &studio.owner_pub(), None, None).is_err());
            prop_assert_eq!(verify_chain(&[], "s", &studio.owner_pub(), None, None), Err(BookListError::Empty));
        }

        /// A manager the owner removed or demoted can't sign what follows,
        /// and a list they signed before can't be grafted on after.
        #[test]
        fn prop_removed_managers_signatures_are_refused(ops in ops(0, 8), demote in any::<bool>(), role in prop::sample::select(vec![Role::View, Role::Comment, Role::Edit])) {
            let mut studio = build("s", &ops);
            let m = studio.add("a_0", Role::Manager).unwrap();
            let fork = studio.next_list(&m, |l| l.push(entry("a_fork", Role::Edit)));
            let signed_fork = sign(&ident(&m), &fork);
            prop_assert!(open_next(studio.head(), &signed_fork, None).is_ok());
            if demote { studio.set_role("a_0", &m, role).unwrap(); } else { studio.remove("a_0", &m).unwrap(); }
            let later = studio.next_list(&m, |l| l.push(entry("a_late", Role::Edit)));
            prop_assert_eq!(studio.push_list(&m, &later), Err(BookListError::NotAllowed));
            prop_assert_eq!(open_next(studio.head(), &signed_fork, None), Err(BookListError::Forked));
            let mut chain = studio.chain.clone();
            chain.push(signed_fork);
            prop_assert!(verify_chain(&chain, "s", &studio.owner_pub(), None, None).is_err());
        }

        /// A chain, or any list of one, belongs to its notebook only, even
        /// with the same owner.
        #[test]
        fn prop_cross_notebook_replay_is_refused(ops in ops(0, 8), other_ops in ops(1, 6)) {
            let s = build("s", &ops);
            let t = build("t", &other_ops);
            prop_assert_eq!(verify_chain(&s.chain, "t", &s.owner_pub(), None, None), Err(BookListError::WrongShare));
            prop_assert_eq!(verify_chain(&t.chain, "s", &s.owner_pub(), None, None), Err(BookListError::WrongShare));
            let mut spliced = t.chain.clone();
            spliced[0] = s.chain[0].clone();
            prop_assert!(verify_chain(&spliced, "t", &s.owner_pub(), None, None).is_err());
            if s.chain.len() > 1 {
                prop_assert_eq!(open_next(t.head(), &s.chain[1], None), Err(BookListError::WrongShare));
            }
            // Re-labelling a signed list for another notebook breaks its signature.
            let mut list = s.heads[0].list.clone();
            list.share = "t".into();
            let relabelled = SignedBookList { list: b64(&serde_json::to_vec(&list).unwrap()), sig: s.chain[0].sig.clone() };
            prop_assert_eq!(open_genesis(&relabelled, "t", &s.owner_pub()), Err(BookListError::BadSignature));
        }

        /// Whatever a manager signs, it never gives anyone Manager or Owner
        /// and never touches a manager or the owner.
        #[test]
        fn prop_manager_cannot_escalate(ops in ops(0, 10), pick in any::<prop::sample::Index>(), role in prop::sample::select(vec![Role::Manager, Role::Owner])) {
            let mut studio = build("s", &ops);
            let m = studio.add("a_0", Role::Manager).unwrap();
            studio.add("a_0", Role::Edit).unwrap();
            let people: Vec<String> = studio.head().list.members.iter()
                .filter(|x| x.account != m && x.role != Role::Owner).map(|x| x.account.clone()).collect();
            let target = people[pick.index(people.len())].clone();
            prop_assume!(studio.head().list.role_of(&target) != Some(role));
            let before = studio.head().clone();
            let list = studio.next_list(&m, |l| {
                l.iter_mut().find(|x| x.account == target).unwrap().role = role;
            });
            prop_assert!(studio.push_list(&m, &list).is_err(), "manager changed {} to {:?}", target, role);
            if studio.head().list.members.len() < MAX_BOOK_MEMBERS {
                let list = studio.next_list(&m, |l| l.push(entry("a_new", role)));
                prop_assert!(studio.push_list(&m, &list).is_err());
            }
            for victim in studio.who(Role::Manager).into_iter().chain(["a_0".to_string()]) {
                let list = studio.next_list(&m, |l| l.retain(|x| x.account != victim));
                prop_assert!(studio.push_list(&m, &list).is_err());
            }
            prop_assert_eq!(studio.head(), &before);
        }
    }

    /// Anyone who stays in a list keeps their name and keys, for a manager
    /// and for the owner: a swapped key would let the signer write as them.
    /// A new key is a removal and a fresh addition.
    #[test]
    fn prop_rekeying_a_member_is_refused() {
        let mut runner = proptest::test_runner::TestRunner::new(crate::tests::config(24));
        runner
            .run(
                &(
                    ops(0, 10),
                    any::<prop::sample::Index>(),
                    0u8..3,
                    any::<bool>(),
                ),
                |(ops, pick, field, by_owner)| {
                    let mut studio = build("s", &ops);
                    let m = studio.add("a_0", Role::Manager).unwrap();
                    studio.add("a_0", Role::Edit).unwrap();
                    let people: Vec<String> = studio
                        .head()
                        .list
                        .members
                        .iter()
                        .filter(|x| x.role < Role::Manager)
                        .map(|x| x.account.clone())
                        .collect();
                    let target = people[pick.index(people.len())].clone();
                    let signer = if by_owner { "a_0" } else { m.as_str() };
                    let other = entry("a_rekey", Role::Edit);
                    let list = studio.next_list(signer, |l| {
                        let t = l.iter_mut().find(|x| x.account == target).unwrap();
                        match field {
                            0 => t.sign_pub = other.sign_pub.clone(),
                            1 => t.box_pub = other.box_pub.clone(),
                            _ => t.username = "someone-else".into(),
                        }
                    });
                    prop_assert_eq!(
                        studio.push_list(signer, &list),
                        Err(BookListError::RankViolation)
                    );
                    // Remove and re-add with the new keys is how it's done.
                    studio.remove("a_0", &target).unwrap();
                    let again = Member {
                        account: target.clone(),
                        ..other
                    };
                    prop_assert!(studio.push("a_0", |l| l.push(again)).is_ok());
                    Ok(())
                },
            )
            .unwrap();
    }

    /// A client that trusts a head only takes chains that pass through it.
    #[test]
    fn prop_forks_below_the_trusted_head_are_refused() {
        let mut runner = proptest::test_runner::TestRunner::new(crate::tests::config(24));
        runner
            .run(&(ops(2, 10), any::<prop::sample::Index>()), |(ops, cut)| {
                let studio = build("s", &ops);
                let n = studio.chain.len();
                prop_assume!(n >= 3);
                let pin = studio.head().pin();
                let owner = studio.owner_pub();
                prop_assert_eq!(
                    verify_chain(&studio.chain, "s", &owner, Some(&pin), None).unwrap(),
                    studio.head().clone()
                );
                // Diverge after list `keep`, then grow past the pinned revision.
                let keep = 1 + cut.index(n - 1);
                let mut alt = Studio {
                    share: studio.share.clone(),
                    chain: studio.chain[..keep].to_vec(),
                    heads: studio.heads[..keep].to_vec(),
                    spawned: 100,
                };
                while alt.chain.len() < n + 1 {
                    alt.add("a_0", Role::Edit).unwrap();
                }
                prop_assert_eq!(
                    verify_chain(&alt.chain, "s", &owner, Some(&pin), None),
                    Err(BookListError::Forked)
                );
                // Without a pin the fork is an ordinary chain.
                prop_assert!(verify_chain(&alt.chain, "s", &owner, None, None).is_ok());
                Ok(())
            })
            .unwrap();
    }

    /// The server hides the removal of a manager and serves a fork the
    /// removed manager keeps extending: a client that saw the removal
    /// refuses it.
    #[test]
    fn a_hidden_removal_is_a_fork() {
        let mut studio = Studio::new("s");
        let m = studio.add("a_0", Role::Manager).unwrap();
        let before = studio.chain.len();
        studio.remove("a_0", &m).unwrap();
        let pin = studio.head().pin();
        let mut alt = Studio {
            share: "s".into(),
            chain: studio.chain[..before].to_vec(),
            heads: studio.heads[..before].to_vec(),
            spawned: 100,
        };
        for _ in 0..3 {
            alt.add(&m, Role::Edit).unwrap();
        }
        assert!(alt.chain.len() > studio.chain.len());
        assert_eq!(
            verify_chain(&alt.chain, "s", &studio.owner_pub(), Some(&pin), None),
            Err(BookListError::Forked)
        );
        // A chain that ends before the trusted head is stale, not a fork.
        assert_eq!(
            verify_chain(
                &studio.chain[..before],
                "s",
                &studio.owner_pub(),
                Some(&pin),
                None
            ),
            Err(BookListError::Stale)
        );
    }

    /// Two managers who each sign the list after the same one: whichever a
    /// client sees second is a fork.
    #[test]
    fn two_managers_signing_the_same_revision_fork_the_history() {
        let mut studio = Studio::new("s");
        let m1 = studio.add("a_0", Role::Manager).unwrap();
        let m2 = studio.add("a_0", Role::Manager).unwrap();
        let first = studio.next_list(&m1, |l| l.push(entry("a_x", Role::Edit)));
        let second = studio.next_list(&m2, |l| l.push(entry("a_y", Role::Edit)));
        let (a, b) = (sign(&ident(&m1), &first), sign(&ident(&m2), &second));
        let base = studio.head().clone();
        let seen_first = open_next(&base, &a, None).unwrap();
        assert_eq!(open_next(&seen_first, &b, None), Err(BookListError::Forked));
        let seen_second = open_next(&base, &b, None).unwrap();
        assert_eq!(
            open_next(&seen_second, &a, None),
            Err(BookListError::Forked)
        );
        // The same list again is a replay, not a fork.
        assert_eq!(open_next(&seen_first, &a, None), Err(BookListError::Broken));
    }

    #[test]
    fn a_new_epochs_log_position_is_bounded() {
        let mut studio = Studio::new("s");
        let e = studio.add("a_0", Role::Edit).unwrap();
        let rev = studio.head().list.rev;
        let cut = |from_seq: u64| {
            let mut list = studio.next_list("a_0", |l| l.retain(|x| x.account != e));
            list.from_seq = Some(from_seq);
            sign(&ident("a_0"), &list)
        };
        let head = studio.head().clone();
        assert_eq!(
            open_next(&head, &cut(u64::MAX), None),
            Err(BookListError::BadEpoch)
        );
        assert_eq!(
            open_next(&head, &cut(MAX_SEQ_JUMP + 1), None),
            Err(BookListError::BadEpoch)
        );
        assert!(open_next(&head, &cut(MAX_SEQ_JUMP), None).is_ok());
        // A client that has seen the log's head holds the epoch to it.
        assert_eq!(
            open_next(&head, &cut(6), Some(5)),
            Err(BookListError::BadEpoch)
        );
        assert!(open_next(&head, &cut(5), Some(5)).is_ok());
        assert!(open_next(&head, &cut(u64::from(rev)), Some(u64::from(rev))).is_ok());
    }

    #[test]
    fn manager_can_run_the_people_below_it() {
        let mut studio = Studio::new("s");
        let m = studio.add("a_0", Role::Manager).unwrap();
        let e = studio.add(&m, Role::Edit).unwrap();
        studio.set_role(&m, &e, Role::Comment).unwrap();
        let before = studio.head().list.epoch;
        studio.remove(&m, &e).unwrap();
        assert_eq!(studio.head().list.epoch, before + 1);
        assert_eq!(studio.head().list.role_of(&m), Some(Role::Manager));
        assert_eq!(
            studio.head().list.managers().collect::<Vec<_>>(),
            [m.as_str()]
        );
        assert!(studio.verify().is_ok());
    }

    #[test]
    fn a_manager_cannot_grant_manager_or_remove_the_owner() {
        let mut studio = Studio::new("s");
        let m = studio.add("a_0", Role::Manager).unwrap();
        let e = studio.add(&m, Role::Edit).unwrap();
        let list = studio.next_list(&m, |l| {
            l.iter_mut().find(|x| x.account == e).unwrap().role = Role::Manager
        });
        assert_eq!(
            studio.push_list(&m, &list),
            Err(BookListError::RankViolation)
        );
        let list = studio.next_list(&m, |l| l.retain(|x| x.account != "a_0"));
        assert!(studio.push_list(&m, &list).is_err());
        let list = studio.next_list(&m, |l| l.retain(|x| x.account != m));
        assert_eq!(
            studio.push_list(&m, &list),
            Err(BookListError::RankViolation)
        );
        // The owner may.
        studio.set_role("a_0", &e, Role::Manager).unwrap();
        assert_eq!(studio.who(Role::Manager).len(), 2);
    }

    #[test]
    fn epoch_rules() {
        let mut studio = Studio::new("s");
        let e = studio.add("a_0", Role::Edit).unwrap();
        // A removal in the same epoch leaves the removed one holding the key.
        let mut list = studio.next_list("a_0", |l| l.retain(|x| x.account != e));
        list.epoch = studio.head().list.epoch;
        list.key_commit = studio.head().list.key_commit.clone();
        list.from_seq = studio.head().list.from_seq;
        assert_eq!(studio.push_list("a_0", &list), Err(BookListError::BadEpoch));
        // The same epoch can't swap the key commitment; a new one must.
        let mut list = studio.next_list("a_0", |l| l.push(entry("a_x", Role::View)));
        list.key_commit = BookKey::generate().commit("s", 1);
        assert_eq!(studio.push_list("a_0", &list), Err(BookListError::BadEpoch));
        let mut list = studio.next_list("a_0", |l| l.push(entry("a_x", Role::View)));
        list.epoch = 2;
        list.from_seq = Some(3);
        assert_eq!(studio.push_list("a_0", &list), Err(BookListError::BadEpoch));
        list.key_commit = BookKey::generate().commit("s", 2);
        assert!(studio.push_list("a_0", &list).is_ok());
        // Epochs step by one and the log position never goes back.
        let mut list = studio.next_list("a_0", |l| l.push(entry("a_y", Role::View)));
        list.epoch = 4;
        list.from_seq = Some(9);
        list.key_commit = BookKey::generate().commit("s", 4);
        assert_eq!(studio.push_list("a_0", &list), Err(BookListError::BadEpoch));
        let mut list = studio.next_list("a_0", |l| l.push(entry("a_y", Role::View)));
        list.epoch = 3;
        list.from_seq = Some(1);
        list.key_commit = BookKey::generate().commit("s", 3);
        assert_eq!(studio.push_list("a_0", &list), Err(BookListError::BadEpoch));
    }

    #[test]
    fn size_and_shape_limits() {
        let mut studio = Studio::new("s");
        for _ in 1..MAX_BOOK_MEMBERS {
            studio.add("a_0", Role::View).unwrap();
        }
        assert_eq!(studio.head().list.members.len(), 25);
        let list = studio.next_list("a_0", |l| l.push(entry("a_z", Role::View)));
        assert_eq!(
            studio.push_list("a_0", &list),
            Err(BookListError::Malformed)
        );
        // Two owners, or unsorted members, are malformed.
        let mut studio = Studio::new("s");
        let list = studio.next_list("a_0", |l| l.push(entry("a_9", Role::Owner)));
        assert_eq!(
            studio.push_list("a_0", &list),
            Err(BookListError::Malformed)
        );
        let mut list = studio.next_list("a_0", |l| {
            l.push(entry("a_1", Role::View));
            l.push(entry("a_2", Role::View));
        });
        list.members.reverse();
        assert_eq!(
            studio.push_list("a_0", &list),
            Err(BookListError::Malformed)
        );
    }

    /// The signature is for notebook lists only: one made in a note member
    /// list's context, or without the notebook bound in, doesn't pass.
    #[test]
    fn signing_context_is_domain_separated() {
        let studio = Studio::new("s");
        let bytes = unb64(&studio.chain[0].list).unwrap();
        let owner = ident("a_0");
        for context in [&b"kanso-members"[..], &b"kanso-book-members"[..]] {
            let mut message = context.to_vec();
            message.push(FORMAT_V1);
            message.extend_from_slice(&bytes);
            let forged = SignedBookList {
                list: studio.chain[0].list.clone(),
                sig: b64(&owner.sign(&message)),
            };
            assert_eq!(
                open_genesis(&forged, "s", &owner.sign_pub()),
                Err(BookListError::BadSignature)
            );
        }
        assert!(open_genesis(&studio.chain[0], "s", &owner.sign_pub()).is_ok());
    }

    #[test]
    fn the_genesis_names_the_pinned_owner() {
        let studio = Studio::new("s");
        assert_eq!(
            open_genesis(&studio.chain[0], "s", &ident("a_x").sign_pub()),
            Err(BookListError::WrongOwner)
        );
        assert_eq!(
            open_genesis(&studio.chain[0], "t", &studio.owner_pub()),
            Err(BookListError::WrongShare)
        );
    }
}
