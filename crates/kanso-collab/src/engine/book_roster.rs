//! Who is in a shared notebook, for the apps (issue #390 › Shared
//! notebooks): inviting more people after it was shared, changing a role,
//! declining an invitation, the member list as this device trusts it, who
//! last changed each note, and what this device still has to send.
//!
//! Adding someone or raising a role stays in the epoch: the next signed list
//! names them, and the epoch's key is wrapped to them (the wrap binds the
//! role). Lowering a role starts a new epoch, as a removal does
//! ([`super::book_people`]): the person still holds the old key. Every list
//! is checked here with the rules every member applies
//! ([`crate::book_members::open_next`]) before it is sent.

use std::collections::{BTreeMap, BTreeSet};

use kanso_core::now_ms;
use kanso_store::SharedStatus;

use super::Engine;
use super::book::Ctx;
use super::book_people::{Rotated, wraps_for};
use super::book_sync::{read_base, split_outgoing};
use crate::api::{CollabError, MANIFEST_STREAM, NewBookList, Result};
use crate::book_members::{self, is_well_formed, link_hash, open_next};
use crate::book_members_model::BookMemberList;
use crate::book_model::{BookMemberView, BookView, MemberStanding, NoteEdit, StoredChain};
use crate::codec::{b64, unb64};
use crate::model::{Member, Role};

/// A change to a notebook's list: given the notebook and this account's
/// role, the members it should have next, or `None` for nothing to do.
type ListChange<'a> = dyn FnMut(&Ctx, Role) -> Result<Option<Vec<Member>>> + 'a;

/// Times a list is signed and sent again before giving up.
const ATTEMPTS: usize = 8;

/// The `meta` key holding who last changed each note of a notebook:
/// `{stream: [account, at]}`.
pub(crate) fn edits_key(share_id: &str) -> String {
    format!("collab.book.edited.{share_id}")
}

/// The username a notebook's signed lists give `account`: the newest list
/// that names them. `None` for an account no list names.
pub fn name_in_chain(chain: &StoredChain, account: &str) -> Option<String> {
    if let Some(member) = chain.head.list.member(account) {
        return Some(member.username.clone());
    }
    chain
        .epochs
        .values()
        .rev()
        .find_map(|list| list.member(account))
        .map(|m| m.username.clone())
}

/// A notebook's members in the order the apps show them: the owner, then
/// by role (highest first), then by username.
pub fn ordered_members(
    list: &BookMemberList,
    me: &str,
    standing: &BTreeMap<String, MemberStanding>,
) -> Vec<BookMemberView> {
    let mut out: Vec<BookMemberView> = list
        .members
        .iter()
        .map(|m| BookMemberView {
            account: m.account.clone(),
            username: m.username.clone(),
            role: m.role,
            standing: if m.role == Role::Owner {
                MemberStanding::Member
            } else {
                standing
                    .get(&m.account)
                    .copied()
                    .unwrap_or(MemberStanding::Unknown)
            },
            me: m.account == me,
        })
        .collect();
    out.sort_by(|a, b| {
        b.role
            .cmp(&a.role)
            .then_with(|| a.username.cmp(&b.username))
            .then_with(|| a.account.cmp(&b.account))
    });
    out
}

/// Whether someone of `mine` may give `to` to a member who is `from` now:
/// the owner changes anyone's role but their own and never makes another
/// owner; a manager changes only roles below Manager, to roles below
/// Manager. The rule the signed chain enforces ([`crate::book_members`]).
pub fn may_change_role(mine: Role, from: Role, to: Role) -> bool {
    if from == Role::Owner || to == Role::Owner {
        return false;
    }
    match mine {
        Role::Owner => true,
        Role::Manager => from < Role::Manager && to < Role::Manager,
        _ => false,
    }
}

/// The refusal for a role someone else changed while this one was asked.
pub(super) fn changed_meanwhile(name: &str) -> CollabError {
    CollabError::Refused(format!(
        "{name}'s role changed meanwhile. Look at it again and choose."
    ))
}

impl Engine {
    // ------------------------------------------------------------- inviting

    /// Invites more people to a notebook that is shared already, by
    /// username, each with a role. Their keys must be pinned (looked up
    /// with [`Engine::person`]). Owners invite anyone at any role but
    /// Owner; managers invite at roles below Manager.
    pub fn invite_to_book(&self, share_id: &str, invite: &[(String, Role)]) -> Result<()> {
        if invite.is_empty() {
            return Ok(());
        }
        let mut people = Vec::new();
        for (username, role) in invite {
            if *role == Role::Owner {
                return Err(CollabError::Refused("A notebook has one owner.".into()));
            }
            let person = self.person(username)?;
            people.push((self.pinned(&person.account_id)?, *role));
        }
        self.amend_book(share_id, &mut |ctx, mine| {
            let mut members = ctx.chain.head.list.members.clone();
            for (person, role) in &people {
                if *role >= Role::Manager && mine != Role::Owner {
                    return Err(CollabError::Refused(
                        "Only the owner can make someone a manager.".into(),
                    ));
                }
                if members.iter().any(|m| m.account == person.account_id) {
                    return Err(CollabError::Refused(format!(
                        "{} is in the notebook already.",
                        person.username
                    )));
                }
                members.push(Member {
                    account: person.account_id.clone(),
                    username: person.username.clone(),
                    role: *role,
                    sign_pub: b64(&person.sign_pub),
                    box_pub: b64(&person.box_pub),
                });
            }
            Ok(Some(members))
        })
    }

    /// Gives a member another role. Raising one stays in the epoch (with a
    /// wrap for the new role); lowering one starts a new epoch, since they
    /// still hold the key their old role opened.
    pub fn set_book_role(&self, share_id: &str, account: &str, role: Role) -> Result<()> {
        // What to do is decided on the server's list, not the one this
        // device last held: it may say they have the role asked already
        // when someone changed it since.
        let mut ctx = self.book_ctx(share_id)?;
        self.refresh_chain(&mut ctx)?;
        let me = self.account.caller.account_id.clone();
        let mine = ctx.role(&me).filter(|r| r.may_manage()).ok_or_else(|| {
            CollabError::Refused("Only the notebook's owner or a manager can change roles.".into())
        })?;
        let member = ctx
            .chain
            .head
            .list
            .member(account)
            .cloned()
            .ok_or_else(|| CollabError::Refused("That person isn't in the notebook.".into()))?;
        if member.role == role {
            return Ok(());
        }
        if account == me {
            return Err(CollabError::Refused(
                "Someone else changes your role.".into(),
            ));
        }
        if !may_change_role(mine, member.role, role) {
            return Err(CollabError::Refused(if member.role == Role::Owner {
                "The owner's role can't change.".into()
            } else {
                "Only the owner can change a manager's role, or make a manager.".into()
            }));
        }
        // Raise or lower as the list this device holds says; when a newer
        // list moved them the other way meanwhile, the other path once.
        let mut lower = role < member.role;
        for _ in 0..2 {
            let turned = if lower {
                self.lower_role(share_id, account, role)?
            } else {
                self.raise_role(share_id, account, role)?
            };
            if !turned {
                return Ok(());
            }
            lower = !lower;
        }
        Err(changed_meanwhile(&member.username))
    }

    /// Lowers a member's role with a new epoch. `true` when a newer list
    /// put them below `role` meanwhile: it is a raise now.
    fn lower_role(&self, share_id: &str, account: &str, role: Role) -> Result<bool> {
        let rotated = self.rotate_book_inner(
            share_id,
            BTreeSet::new(),
            BTreeMap::from([(account.to_owned(), role)]),
            true,
        )?;
        Ok(rotated == Rotated::RoleMoved(account.to_owned()))
    }

    /// Raises a member's role in the epoch. `true` when a newer list put
    /// them above `role` meanwhile: it is a lowering now.
    fn raise_role(&self, share_id: &str, account: &str, role: Role) -> Result<bool> {
        let me = self.account.caller.account_id.clone();
        let mut above = false;
        self.amend_book(share_id, &mut |ctx, mine| {
            above = false;
            let mut members = ctx.chain.head.list.members.clone();
            let Some(member) = members.iter_mut().find(|m| m.account == account) else {
                return Err(CollabError::Refused(
                    "That person isn't in the notebook.".into(),
                ));
            };
            if member.role == role {
                return Ok(None);
            }
            if member.role > role {
                above = true;
                return Ok(None);
            }
            if account == me || !may_change_role(mine, member.role, role) {
                return Err(changed_meanwhile(&member.username));
            }
            member.role = role;
            Ok(Some(members))
        })?;
        Ok(above)
    }

    /// Declines an invitation to a notebook this device hasn't joined. The
    /// owner and managers see it as declined until one of their devices
    /// takes the account off the signed list.
    pub fn decline_book(&self, share_id: &str) -> Result<()> {
        let in_it = || CollabError::Refused("You're in that notebook: leave it instead.".into());
        if self
            .store
            .shared_book(share_id)?
            .is_some_and(|row| row.status == SharedStatus::Active)
        {
            return Err(in_it());
        }
        // Another device of the account may have accepted it already: what
        // the server says decides, never this device's library alone.
        match self
            .api
            .books(&self.account.caller)?
            .into_iter()
            .find(|b| b.share_id == share_id)
        {
            Some(book) if book.status == "invited" => {}
            Some(_) => return Err(in_it()),
            // Nothing waits: declined or cancelled already.
            None => return Ok(()),
        }
        match self.api.leave_book(&self.account.caller, share_id) {
            Ok(()) | Err(CollabError::Api { status: 404, .. }) => Ok(()),
            Err(err) => Err(err),
        }
    }

    /// Sends the list `change` makes of the chain's head, in the same epoch:
    /// those added, or given a higher role, get a wrap of the epoch's key.
    /// `change` gets the notebook and this account's role, and says `None`
    /// when nothing is left to do. Again on a list that took the place
    /// meanwhile.
    fn amend_book(&self, share_id: &str, change: &mut ListChange<'_>) -> Result<()> {
        let me = self.account.caller.account_id.clone();
        let identity = self.identity()?;
        for _ in 0..ATTEMPTS {
            let ctx = self.book_ctx(share_id)?;
            let head = ctx.chain.head.clone();
            let mine = ctx.role(&me).filter(|r| r.may_manage()).ok_or_else(|| {
                CollabError::Refused("Only the notebook's owner or a manager can do that.".into())
            })?;
            let Some(members) = change(&ctx, mine)? else {
                return Ok(());
            };
            let key = ctx.keys.get(&head.list.epoch).cloned().ok_or_else(|| {
                CollabError::Crypto("this device has no key for the notebook".into())
            })?;
            let mut list = BookMemberList {
                rev: head.list.rev + 1,
                signer: me.clone(),
                prev: Some(head.hash.clone()),
                at: now_ms(),
                members,
                ..head.list.clone()
            };
            list.sort();
            if !is_well_formed(&list) {
                return Err(CollabError::Refused(
                    if list.members.len() > book_members::MAX_BOOK_MEMBERS {
                        format!(
                            "A notebook can be shared with at most {} people.",
                            book_members::MAX_BOOK_MEMBERS - 1
                        )
                    } else {
                        "That list of people isn't possible.".into()
                    },
                ));
            }
            let fresh: Vec<Member> = list
                .members
                .iter()
                .filter(|m| {
                    head.list
                        .member(&m.account)
                        .is_none_or(|old| old.role != m.role)
                })
                .cloned()
                .collect();
            let signed = book_members::sign(&identity, &list);
            // The rules every member will apply to it, checked here first.
            open_next(&head, &signed, ctx.row.head)?;
            let wraps = wraps_for(&key, share_id, list.epoch, &fresh)?;
            let our_hash = unb64(&signed.list).map(|bytes| link_hash(&bytes));
            match self.api.new_book_list(
                &self.account.caller,
                share_id,
                &NewBookList {
                    rev: list.rev,
                    list: signed.list.clone(),
                    list_sig: signed.sig.clone(),
                    wraps,
                    history_link: None,
                },
            ) {
                Ok(()) => return self.take_new_epoch(share_id),
                Err(err) if err.code() == Some("rev_conflict") => {
                    let mut now = self.book_ctx(share_id)?;
                    self.refresh_chain(&mut now)?;
                    if our_hash.as_deref() == Some(now.chain.head.hash.as_str()) {
                        return Ok(());
                    }
                }
                // Someone the list keeps no longer has an account: they are
                // cut with a new key, and the change goes on the new head.
                Err(err) if err.code() == Some("member_gone") => {
                    let account = err.extra_str("account").map(str::to_owned).ok_or(err)?;
                    let Some(gone) = head.list.member(&account) else {
                        return Err(CollabError::Protocol(
                            "the server named someone the notebook doesn't list".into(),
                        ));
                    };
                    let name = gone.username.clone();
                    if let Err(cut) = self.rotate_book(
                        share_id,
                        BTreeSet::from([account]),
                        BTreeMap::new(),
                        false,
                    ) {
                        return Err(CollabError::Refused(format!(
                            "{name} no longer has a Kanso account, and Kanso couldn't take \
                             them out of the notebook ({cut}). Remove {name} from it first."
                        )));
                    }
                }
                Err(err) => return Err(err),
            }
        }
        Err(CollabError::Refused(
            "The notebook kept changing. Try again.".into(),
        ))
    }

    // ---------------------------------------------------------------- views

    /// The notebook's people as this device trusts them, without asking the
    /// server: nobody's standing is known.
    pub fn book_view(&self, share_id: &str) -> Result<BookView> {
        let ctx = self.book_ctx(share_id)?;
        Ok(self.view_of(&ctx, &BTreeMap::new()))
    }

    /// The notebook's people after bringing its chain up to the server's,
    /// with who has accepted (the server tells the owner and managers).
    /// Offline, the list this device trusts.
    pub fn book_view_fresh(&self, share_id: &str) -> Result<BookView> {
        let mut ctx = self.book_ctx(share_id)?;
        match self.refresh_chain(&mut ctx) {
            Ok(()) => {}
            Err(err) if err.is_offline() => return Ok(self.view_of(&ctx, &BTreeMap::new())),
            Err(err) => return Err(err),
        }
        let me = &self.account.caller.account_id;
        let standing = if ctx.role(me).is_some_and(|r| r.may_manage()) {
            match self
                .api
                .book(&self.account.caller, share_id, ctx.chain.head.list.rev)
            {
                Ok(info) => info
                    .members
                    .into_iter()
                    .map(|m| (m.account, MemberStanding::parse(&m.status)))
                    .collect(),
                Err(err) if err.is_offline() => BTreeMap::new(),
                Err(err) => return Err(err),
            }
        } else {
            BTreeMap::new()
        };
        Ok(self.view_of(&ctx, &standing))
    }

    fn view_of(&self, ctx: &Ctx, standing: &BTreeMap<String, MemberStanding>) -> BookView {
        let me = &self.account.caller.account_id;
        let name = self
            .store
            .books()
            .ok()
            .and_then(|books| books.into_iter().find(|b| b.id == ctx.row.book_id))
            .map(|b| b.name)
            .unwrap_or_default();
        BookView {
            share_id: ctx.row.share_id.clone(),
            book_id: ctx.row.book_id.clone(),
            name,
            owner_id: ctx.row.owner_id.clone(),
            owner_name: ctx.row.owner_name.clone(),
            my_role: ctx.role(me),
            epoch: ctx.chain.head.list.epoch,
            members: ordered_members(&ctx.chain.head.list, me, standing),
            uploading: ctx.row.uploading,
        }
    }

    // ---------------------------------------------------------------- edits

    /// Keeps who last changed a note of a notebook (an edit this device
    /// merged, or one it wrote).
    pub(crate) fn note_edited(&self, share_id: &str, stream: &str, account: &str) {
        if stream == MANIFEST_STREAM {
            return;
        }
        let key = edits_key(share_id);
        let mut map: BTreeMap<String, (String, i64)> = self
            .store
            .meta(&key)
            .ok()
            .flatten()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();
        map.insert(stream.to_owned(), (account.to_owned(), now_ms()));
        let text = serde_json::to_string(&map).expect("a map serializes");
        if let Err(err) = self.store.set_meta(&key, &text) {
            eprintln!("kanso: keeping who edited a shared note: {err}");
        }
    }

    /// Who last changed each note of the notebooks joined here, by note.
    /// Names come from each notebook's signed member lists; an account no
    /// list names is left out.
    pub fn book_edits(&self) -> Vec<NoteEdit> {
        let me = &self.account.caller.account_id;
        let mut out = Vec::new();
        for row in self.store.shared_books().unwrap_or_default() {
            if row.status != SharedStatus::Active {
                continue;
            }
            let Ok(ctx) = self.book_ctx(&row.share_id) else {
                continue;
            };
            let map: BTreeMap<String, (String, i64)> = self
                .store
                .meta(&edits_key(&row.share_id))
                .ok()
                .flatten()
                .and_then(|text| serde_json::from_str(&text).ok())
                .unwrap_or_default();
            for (stream, (account, at)) in map {
                let Some(note_id) = self
                    .store
                    .book_stream(&row.share_id, &stream)
                    .ok()
                    .flatten()
                    .and_then(|s| s.note_id)
                else {
                    continue;
                };
                let Some(name) = name_in_chain(&ctx.chain, &account) else {
                    continue;
                };
                out.push(NoteEdit {
                    note_id,
                    share_id: row.share_id.clone(),
                    mine: account == *me,
                    account,
                    name,
                    at,
                });
            }
        }
        out
    }

    /// How many notes of a notebook have changes this device hasn't sent:
    /// queued, or written in the library since the last sync took them in.
    pub fn book_waiting(&self, share_id: &str) -> Result<usize> {
        let mut streams: BTreeSet<String> = self
            .store
            .book_outbox(share_id)?
            .iter()
            .map(|o| split_outgoing(&o.stream).0.to_owned())
            .collect();
        for row in self.store.book_streams(share_id)? {
            let Some(note_id) = &row.note_id else {
                continue;
            };
            let (Some(base), Some(note)) = (read_base(&row), self.store.note(note_id)?) else {
                continue;
            };
            if (note.title, note.body) != base {
                streams.insert(row.stream.clone());
            }
        }
        Ok(streams.len())
    }
}
