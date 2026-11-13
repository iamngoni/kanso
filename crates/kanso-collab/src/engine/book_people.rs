//! Taking people out of a shared notebook, and the key that goes with it
//! (issue #390 › Shared notebooks: keys). Whoever is cut still holds the
//! notebook key of the epoch they were in, so removal starts the next
//! epoch: a fresh `K_b`, wrapped only to those who stay, with a history
//! link so the people who stay read what was written before. Entries
//! written under the new key are unreadable to the person cut however many
//! old keys they hold.
//!
//! The server takes the new list only at the log's real head (`head_moved`
//! otherwise) and only on top of the chain's head (`rev_conflict`
//! otherwise); the list is signed again and sent again until one of them
//! lands. A listed account that has been deleted (`member_gone`) is cut as
//! well.

use std::collections::{BTreeMap, BTreeSet};

use kanso_core::now_ms;

use super::Engine;
use super::book::Ctx;
use crate::api::{AccountWrap, CollabError, NewBookList, Result};
use crate::book::{history_link, wrap};
use crate::book_members::{self, is_well_formed, link_hash, open_next};
use crate::book_members_model::BookMemberList;
use crate::book_model::BookKey;
use crate::codec::{b64, unb64, unb64_array};
use crate::model::{Member, Role};

/// Times a list is signed and sent again before giving up.
const ATTEMPTS: usize = 8;

impl Engine {
    /// Removes a member (or cancels an invitation). They keep what they
    /// already had on their devices; nothing written from now on is
    /// readable to them. Owners remove anyone but themselves; managers
    /// anyone below Manager.
    pub fn remove_book_member(&self, share_id: &str, account: &str) -> Result<()> {
        self.remove_book_members(share_id, &[account.to_owned()])
    }

    /// Removes several people in one new epoch.
    pub fn remove_book_members(&self, share_id: &str, accounts: &[String]) -> Result<()> {
        let mut cut: BTreeSet<String> = accounts.iter().cloned().collect();
        // The new epoch is paid for already: those who have left, whom the
        // signed list still names, go with it.
        if let Ok(ctx) = self.book_ctx(share_id)
            && let Ok(left) = self.left_members(&ctx)
        {
            cut.extend(left);
        }
        self.rotate_book(share_id, cut, BTreeMap::new(), true)
    }

    /// Cuts every member who has left (or declined) from the signed list,
    /// which still names them, with a new key. How many were cut. For the
    /// owner and managers.
    pub fn cut_left_members(&self, share_id: &str) -> Result<usize> {
        let ctx = self.book_ctx(share_id)?;
        let left = self.left_members(&ctx)?;
        if left.is_empty() {
            return Ok(0);
        }
        let count = left.len();
        self.rotate_book(share_id, left, BTreeMap::new(), false)?;
        Ok(count)
    }

    /// Those the signed list names who have left, as the server says (it
    /// tells only the owner and managers).
    fn left_members(&self, ctx: &Ctx) -> Result<BTreeSet<String>> {
        let info = self.api.book(
            &self.account.caller,
            &ctx.row.share_id,
            ctx.chain.head.list.rev,
        )?;
        let me = &self.account.caller.account_id;
        let mine = ctx.role(me);
        Ok(info
            .members
            .into_iter()
            .filter(|m| m.status == "left" && m.account != ctx.chain.head.list.owner)
            .filter(|m| m.role < Role::Manager || mine == Some(Role::Owner))
            .filter(|m| ctx.chain.head.list.member(&m.account).is_some())
            .map(|m| m.account)
            .collect())
    }

    /// Starts the next epoch without `cut` and with the roles in `demote`
    /// lowered: signs the list, wraps the new key to everyone who stays,
    /// and sends it, again on a moved head or chain. `explicit` is a
    /// person's request (the accounts must be members), against the
    /// housekeeping of [`Engine::cut_left_members`].
    pub(super) fn rotate_book(
        &self,
        share_id: &str,
        mut cut: BTreeSet<String>,
        demote: BTreeMap<String, Role>,
        explicit: bool,
    ) -> Result<()> {
        let me = self.account.caller.account_id.clone();
        let identity = self.identity()?;
        let mut head_hint: Option<u64> = None;
        let mut minted: Option<(u32, BookKey)> = None;
        for attempt in 0..ATTEMPTS {
            let ctx = self.book_ctx(share_id)?;
            let head = ctx.chain.head.clone();
            let my_role = ctx
                .role(&me)
                .filter(|role| role.may_manage())
                .ok_or_else(|| {
                    CollabError::Refused(
                        "Only the notebook's owner or a manager can remove people.".into(),
                    )
                })?;
            for account in cut.clone() {
                let Some(member) = head.list.member(&account) else {
                    // Gone from the chain since (another list cut them):
                    // nothing left to do for them, unless it was asked for
                    // and never was in.
                    if explicit && attempt == 0 {
                        return Err(CollabError::Refused(
                            "That person isn't in the notebook.".into(),
                        ));
                    }
                    cut.remove(&account);
                    continue;
                };
                if member.role == Role::Owner {
                    return Err(CollabError::Refused(
                        "The owner can't be removed: stop sharing the notebook instead.".into(),
                    ));
                }
                if account == me {
                    return Err(CollabError::Refused(
                        "To go, leave the notebook instead.".into(),
                    ));
                }
                if member.role >= Role::Manager && my_role != Role::Owner {
                    return Err(CollabError::Refused(
                        "Only the owner can remove a manager.".into(),
                    ));
                }
            }
            // Demotions of people still listed, checked as removals are.
            let mut lowered = BTreeMap::new();
            for (account, role) in &demote {
                let Some(member) = head.list.member(account) else {
                    if explicit && attempt == 0 {
                        return Err(CollabError::Refused(
                            "That person isn't in the notebook.".into(),
                        ));
                    }
                    continue;
                };
                if member.role == Role::Owner || *role == Role::Owner {
                    return Err(CollabError::Refused(
                        "The owner's role can't change.".into(),
                    ));
                }
                if *account == me {
                    return Err(CollabError::Refused(
                        "Someone else changes your role.".into(),
                    ));
                }
                if (member.role >= Role::Manager || *role >= Role::Manager)
                    && my_role != Role::Owner
                {
                    return Err(CollabError::Refused(
                        "Only the owner can change a manager's role.".into(),
                    ));
                }
                if *role < member.role {
                    lowered.insert(account.clone(), *role);
                }
            }
            if cut.is_empty() && lowered.is_empty() {
                return Ok(());
            }
            let epoch = head.list.epoch + 1;
            let previous = ctx.keys.get(&head.list.epoch).cloned().ok_or_else(|| {
                CollabError::Crypto("this device has no key for the notebook".into())
            })?;
            // The same key until the epoch it is for changes under us.
            let key = match &minted {
                Some((e, key)) if *e == epoch => key.clone(),
                _ => {
                    let key = BookKey::generate();
                    minted = Some((epoch, key.clone()));
                    key
                }
            };
            let from_seq = match head_hint.take() {
                Some(head) => head,
                None => self.log_head(share_id, ctx.row.head)?,
            };
            let mut list = BookMemberList {
                v: 1,
                share: share_id.to_owned(),
                rev: head.list.rev + 1,
                epoch,
                owner: head.list.owner.clone(),
                signer: me.clone(),
                prev: Some(head.hash.clone()),
                key_commit: key.commit(share_id, epoch),
                at: now_ms(),
                from_seq: Some(from_seq),
                members: head
                    .list
                    .members
                    .iter()
                    .filter(|m| !cut.contains(&m.account))
                    .map(|m| Member {
                        role: lowered.get(&m.account).copied().unwrap_or(m.role),
                        ..m.clone()
                    })
                    .collect(),
            };
            list.sort();
            if !is_well_formed(&list) {
                return Err(CollabError::Refused(
                    "That list of people isn't possible.".into(),
                ));
            }
            let signed = book_members::sign(&identity, &list);
            // The rules every member will apply to it, checked here first.
            open_next(&head, &signed, from_seq)?;
            let wraps = wraps_for(&key, share_id, epoch, &list.members)?;
            let link = history_link(share_id, epoch, &key, &previous);
            let our_hash = unb64(&signed.list).map(|bytes| link_hash(&bytes));
            match self.api.new_book_list(
                &self.account.caller,
                share_id,
                &NewBookList {
                    rev: list.rev,
                    list: signed.list.clone(),
                    list_sig: signed.sig.clone(),
                    wraps,
                    history_link: Some(b64(&link)),
                },
            ) {
                Ok(()) => return self.take_new_epoch(share_id),
                // Someone wrote meanwhile: signed again from the head the
                // server names.
                Err(err) if err.code() == Some("head_moved") => {
                    head_hint = err.extra("head");
                }
                // Another list took the place: build on it, unless it is
                // this one (an answer that was lost).
                Err(err) if err.code() == Some("rev_conflict") => {
                    let mut now = self.book_ctx(share_id)?;
                    self.refresh_chain(&mut now)?;
                    if our_hash.as_deref() == Some(now.chain.head.hash.as_str()) {
                        return Ok(());
                    }
                }
                // Someone the list keeps no longer has an account: cut
                // them too.
                Err(err) if err.code() == Some("member_gone") => {
                    let account = err.extra_str("account").map(str::to_owned).ok_or(err)?;
                    if head.list.member(&account).is_none() {
                        return Err(CollabError::Protocol(
                            "the server named someone the notebook doesn't list".into(),
                        ));
                    }
                    cut.insert(account);
                }
                Err(err) => return Err(err),
            }
        }
        Err(CollabError::Refused(
            "The notebook kept changing. Try again.".into(),
        ))
    }

    /// Takes in the epoch this device just started: the chain, and the key
    /// (opened from the wrap the server keeps for this account).
    pub(super) fn take_new_epoch(&self, share_id: &str) -> Result<()> {
        let mut ctx = self.book_ctx(share_id)?;
        self.refresh_chain(&mut ctx)
    }
}

/// One wrap of `key` for every member of a list.
pub(super) fn wraps_for(
    key: &BookKey,
    share_id: &str,
    epoch: u32,
    members: &[Member],
) -> Result<Vec<AccountWrap>> {
    members
        .iter()
        .map(|m| {
            let box_pub = unb64_array::<32>(&m.box_pub)
                .ok_or_else(|| CollabError::Crypto("a member's key isn't well-formed".into()))?;
            let wrapped =
                wrap(key, share_id, epoch, &m.account, m.role, &box_pub).ok_or_else(|| {
                    CollabError::Crypto(format!("{}'s key can't be used", m.username))
                })?;
            Ok(AccountWrap {
                account: m.account.clone(),
                wrapped,
                epoch: None,
            })
        })
        .collect()
}
