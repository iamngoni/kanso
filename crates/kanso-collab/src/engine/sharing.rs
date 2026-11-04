//! Sharing a note and managing who is in it (docs/ACCOUNTS.md ›
//! Collaboration › Keys for shared notes, › The member list): the first
//! epoch, every later one (a person added, removed or given another role),
//! invite links and requests, invitations, leaving and stopping, and
//! bringing the server's view of the account's shares into the library.

use std::collections::{BTreeMap, HashSet};

use kanso_core::canvas::{self, Canvas};
use kanso_core::now_ms;
use kanso_store::{SharedCanvas, SharedKind, SharedNote, SharedStatus, SharedText};

use super::people::Pinned;
use super::{Engine, lock};
use crate::api::{
    AccountWrap, CollabError, Joined, NewEpoch, NewLink, NewShare, Result, ShareInfo,
};
use crate::canvas_doc::CanvasDoc;
use crate::codec::{b64, new_id};
use crate::doc::CollabDoc;
use crate::envelope;
use crate::identity::Verified;
use crate::keys::{LinkSecret, NoteKey, invite_link, parse_invite_link, unwrap, vault_seal, wrap};
use crate::members::{self, MAX_MEMBERS};
use crate::model::{
    Header, Invitation, Kind, LinkView, Member, MemberList, MemberView, Person, RequestView, Role,
    ShareView,
};
use crate::shared_doc::SharedDoc;

/// The longest an invite link lasts.
pub const MAX_LINK_DAYS: i64 = 30;

/// The note ID a shared note gets in a library that didn't share it: the
/// same on every device of the account.
pub fn shared_note_id(share_id: &str) -> String {
    format!("note:s{share_id}")
}

/// `keys` wrapped to each of `people` with their role. Wraps of `earlier`
/// epochs name their epoch; the new epoch's don't.
fn wraps(
    share_id: &str,
    keys: &[(u32, NoteKey)],
    people: &[(Verified, Role)],
    current: Option<u32>,
) -> Result<Vec<AccountWrap>> {
    let mut out = Vec::new();
    for (epoch, key) in keys {
        for (person, role) in people {
            let wrapped = wrap(
                key,
                share_id,
                *epoch,
                &person.account_id,
                *role,
                &person.box_pub,
            )
            .ok_or_else(|| {
                CollabError::Crypto(format!("{}'s key can't be used", person.username))
            })?;
            out.push(AccountWrap {
                account: person.account_id.clone(),
                wrapped,
                epoch: current.map(|_| *epoch),
            });
        }
    }
    Ok(out)
}

/// The `meta` key remembering an invite link's role on the device that
/// made it.
fn link_meta(link_id: &str) -> String {
    format!("collab.link.{link_id}")
}

fn member(identity: &Verified, role: Role) -> Member {
    Member {
        account: identity.account_id.clone(),
        username: identity.username.clone(),
        role,
        sign_pub: b64(&identity.sign_pub),
        box_pub: b64(&identity.box_pub),
    }
}

impl Engine {
    /// Why a note can't be shared, if it can't: it is locked, in a locked
    /// notebook, in the trash, or already shared.
    pub fn check_shareable(&self, note_id: &str) -> Result<()> {
        let note = self
            .store
            .note(note_id)?
            .ok_or_else(|| CollabError::NotShareable("The note no longer exists.".into()))?;
        if self.store.note_is_locked(note_id)? || self.store.note_in_locked_book(note_id)? {
            return Err(CollabError::NotShareable(
                "Locked notes, and notes in locked notebooks, can't be shared.".into(),
            ));
        }
        if note.book_id == kanso_core::model::TRASH_BOOK_ID {
            return Err(CollabError::NotShareable(
                "The note is in the trash.".into(),
            ));
        }
        if note.extra.contains_key(kanso_core::model::SHARE_FIELD) {
            return Err(CollabError::NotShareable(
                "The note is already shared.".into(),
            ));
        }
        Ok(())
    }

    /// Shares a note with people (by username, each with a role), and
    /// returns the share's ID. Their keys must already be pinned (looked up
    /// with [`Engine::person`]).
    pub fn share(&self, note_id: &str, invite: &[(String, Role)]) -> Result<String> {
        self.check_shareable(note_id)?;
        if invite.len() + 1 > MAX_MEMBERS {
            return Err(CollabError::Refused(format!(
                "A note can be shared with at most {} people.",
                MAX_MEMBERS - 1
            )));
        }
        let identity = self.identity()?;
        let note = self.store.note(note_id)?.expect("checked above");
        // Text or canvas, by the note as it is now; the owner-signed list
        // carries it from here on.
        let kind = if note.doctype == "canvas" {
            SharedKind::Canvas
        } else {
            SharedKind::Text
        };
        let doc = match kind {
            SharedKind::Text => SharedDoc::Text(CollabDoc::from_text(&note.title, &note.body)),
            SharedKind::Canvas => {
                let canvas = if note.body.trim().is_empty() {
                    Canvas::default()
                } else {
                    canvas::parse(&note.body).map_err(|why| {
                        CollabError::NotShareable(format!(
                            "The canvas can't be read, so it can't be shared yet: {why}"
                        ))
                    })?
                };
                SharedDoc::Canvas(CanvasDoc::from_canvas(&note.title, &canvas))
            }
        };
        let caller = &self.account.caller;
        let share_id = new_id();
        let key = NoteKey::generate();
        let me = Verified {
            account_id: caller.account_id.clone(),
            username: caller.username.clone(),
            sign_pub: identity.sign_pub(),
            box_pub: identity.box_pub(),
        };
        let mut people = vec![(me.clone(), Role::Owner)];
        for (username, role) in invite {
            if *role == Role::Owner {
                return Err(CollabError::Refused("A note has one owner.".into()));
            }
            let person = self.person(username)?;
            people.push((self.pinned(&person.account_id)?, *role));
        }
        let (list, list_sig, wraps) = self.sign_epoch(&share_id, 1, &key, &people, None, kind)?;
        let state = doc.encode_state();
        let header = Header {
            kind: Kind::Snapshot,
            epoch: 1,
            author_account: caller.account_id.clone(),
            author_device: caller.device_id.clone(),
            counter: 1,
        };
        let snapshot = envelope::seal(&identity, &key, &share_id, &header, Some(0), &state);
        self.api.create_share(
            caller,
            &NewShare {
                share_id: share_id.clone(),
                list: list.clone(),
                list_sig: list_sig.clone(),
                wraps,
                snapshot: b64(&snapshot),
            },
        )?;
        self.store.put_collab_key(
            &share_id,
            1,
            &vault_seal(&self.account.roots.vault_key, &share_id, 1, &key),
            Some((&list, &list_sig)),
        )?;
        let shared = SharedNote {
            share_id: share_id.clone(),
            note_id: note_id.to_owned(),
            owner_id: caller.account_id.clone(),
            owner_name: caller.username.clone(),
            role: Role::Owner.as_str().into(),
            status: SharedStatus::Active,
            kind,
            epoch: 1,
            list,
            list_sig,
            state,
            applied_seq: 0,
            since_snapshot: 0,
            bytes_since_snapshot: 0,
            counter: 1,
            needs_snapshot: false,
            updated_at: now_ms(),
        };
        self.store.put_shared_note(&shared)?;
        match &doc {
            SharedDoc::Text(_) => {
                self.store.write_shared_text(&SharedText {
                    note_id,
                    title: &note.title,
                    body: &note.body,
                    mark: Some(Engine::mark(&shared)),
                    new_in: &note.book_id,
                    expected_rev: None,
                    authors: &[],
                })?;
            }
            // What the library holds is the document as it was seeded.
            SharedDoc::Canvas(canvas) => {
                self.store.write_shared_canvas(&SharedCanvas {
                    note_id,
                    title: &note.title,
                    canvas: &canvas.to_canvas(),
                    mark: Some(Engine::mark(&shared)),
                    new_in: &note.book_id,
                    expected_rev: None,
                    authors: &[],
                })?;
            }
        }
        Ok(share_id)
    }

    /// Builds and signs an epoch's list, and wraps `key` to everyone in it.
    fn sign_epoch(
        &self,
        share_id: &str,
        epoch: u32,
        key: &NoteKey,
        people: &[(Verified, Role)],
        from_seq: Option<u64>,
        kind: SharedKind,
    ) -> Result<(String, String, Vec<AccountWrap>)> {
        let identity = self.identity()?;
        let mut list = MemberList {
            v: 1,
            share: share_id.to_owned(),
            epoch,
            owner: self.account.caller.account_id.clone(),
            key_commit: key.commit(share_id, epoch),
            at: now_ms(),
            from_seq,
            kind: (kind == SharedKind::Canvas).then(|| kind.as_str().to_owned()),
            members: people.iter().map(|(p, role)| member(p, *role)).collect(),
        };
        list.sort();
        if !members::is_well_formed(&list) {
            return Err(CollabError::Refused(
                "That list of people isn't possible.".into(),
            ));
        }
        let (bytes, sig) = members::sign(&identity, &list);
        let wraps = wraps(share_id, &[(epoch, key.clone())], people, None)?;
        Ok((bytes, sig, wraps))
    }

    /// Starts the next epoch with `people` (the owner left out: always in)
    /// under a new key, then writes its snapshot.
    fn rotate(&self, share_id: &str, people: Vec<(Verified, Role)>) -> Result<()> {
        let loaded = self.loaded(share_id)?;
        let new_epoch;
        let (identity, key, everyone, earlier_wraps, head, kind) = {
            let guard = lock(&loaded);
            if guard.role() != Role::Owner {
                return Err(CollabError::Refused(
                    "Only the note's owner can change who is in it.".into(),
                ));
            }
            let identity = self.identity()?;
            let caller = &self.account.caller;
            let me = Verified {
                account_id: caller.account_id.clone(),
                username: caller.username.clone(),
                sign_pub: identity.sign_pub(),
                box_pub: identity.box_pub(),
            };
            let mut everyone = vec![(me, Role::Owner)];
            everyone.extend(
                people
                    .into_iter()
                    .filter(|(p, _)| p.account_id != caller.account_id),
            );
            if everyone.len() > MAX_MEMBERS {
                return Err(CollabError::Refused(format!(
                    "A note can be shared with at most {} people.",
                    MAX_MEMBERS - 1
                )));
            }
            new_epoch = guard.note.epoch + 1;
            let key = NoteKey::generate();
            let newcomers: Vec<(Verified, Role)> = everyone
                .iter()
                .filter(|(p, _)| guard.current.member(&p.account_id).is_none())
                .cloned()
                .collect();
            let earlier: Vec<(u32, NoteKey)> =
                guard.keys.iter().map(|(e, k)| (*e, k.clone())).collect();
            // Newcomers get the earlier epochs' keys too, to read the
            // comments written before they came.
            let earlier_wraps = wraps(share_id, &earlier, &newcomers, Some(new_epoch))?;
            (
                identity,
                key,
                everyone,
                earlier_wraps,
                guard.note.applied_seq,
                guard.note.kind,
            )
        };
        // The list names the log's head the epoch starts from; the server
        // takes it only at that head, so a moved head means signing again.
        let mut head = head;
        let mut attempt = 0;
        let (list, list_sig) = loop {
            let (list, list_sig, mut new_wraps) =
                self.sign_epoch(share_id, new_epoch, &key, &everyone, Some(head), kind)?;
            new_wraps.extend(earlier_wraps.iter().cloned());
            // Kept before the server hears of it, so a lost answer loses
            // no key.
            self.store.put_collab_key(
                share_id,
                new_epoch,
                &vault_seal(&self.account.roots.vault_key, share_id, new_epoch, &key),
                Some((&list, &list_sig)),
            )?;
            // The network outside the note's lock: the editor never waits
            // on it.
            match self.api.new_epoch(
                &self.account.caller,
                share_id,
                &NewEpoch {
                    epoch: new_epoch,
                    list: list.clone(),
                    list_sig: list_sig.clone(),
                    wraps: new_wraps,
                },
            ) {
                Ok(_) => break (list, list_sig),
                Err(err) if err.code() == Some("head_moved") && attempt < 5 => {
                    attempt += 1;
                    head = err.extra("head").unwrap_or(head);
                }
                Err(err) => return Err(err),
            }
        };
        {
            let mut guard = lock(&loaded);
            if guard.note.epoch + 1 != new_epoch {
                return Err(CollabError::Refused(
                    "The note's people changed meanwhile. Try again.".into(),
                ));
            }
            let opened = members::open(&list, &list_sig, share_id, &identity.sign_pub(), 0)?;
            guard.keys.insert(new_epoch, key);
            guard.lists.insert(new_epoch, opened.clone());
            guard.current = opened;
            guard.note.epoch = new_epoch;
            guard.note.list = list;
            guard.note.list_sig = list_sig;
            guard.note.needs_snapshot = true;
            self.persist(&mut guard)?;
        }
        // The new epoch's snapshot, so newcomers can read the note: written
        // now if possible, else by the next sync.
        let _ = self.sync_note(share_id);
        self.refresh_view(share_id)?;
        Ok(())
    }

    /// The current people of a note (the owner left out), as `rotate` takes
    /// them: those still in it on the server (anyone who left, declined or
    /// whose account is gone drops out here), with the keys pinned for them.
    fn people(&self, share_id: &str) -> Result<Vec<(Verified, Role)>> {
        let loaded = self.loaded(share_id)?;
        let current = lock(&loaded).current.clone();
        let present = lock(&self.server_members).get(share_id).cloned();
        let mut out = Vec::new();
        for member in current.members.iter().filter(|m| m.role != Role::Owner) {
            if present
                .as_ref()
                .is_some_and(|p| !p.contains(&member.account))
            {
                continue;
            }
            match self.pinned(&member.account) {
                Ok(person) if crate::codec::b64(&person.sign_pub) == member.sign_pub => {
                    out.push((person, member.role));
                }
                // No keys of theirs to wrap to: they drop out too.
                _ => {}
            }
        }
        Ok(out)
    }

    /// Whether a member's keys are the ones pinned for them (this account's
    /// own always are).
    fn checked(&self, member: &Member) -> bool {
        member.account == self.account.caller.account_id
            || self
                .store
                .contact(&member.account)
                .ok()
                .flatten()
                .is_some_and(|c| c.sign_pub == member.sign_pub && c.box_pub == member.box_pub)
    }

    /// A member as the Share sheet shows them: their pinned contact, or, when
    /// their keys couldn't be checked, what the owner's list says, unverified.
    fn member_person(&self, member: &Member) -> Person {
        match self.person_of(&member.account) {
            Ok(person) if self.checked(member) => person,
            _ => Person {
                account_id: member.account.clone(),
                username: member.username.clone(),
                fingerprint: match (
                    crate::codec::unb64_array::<32>(&member.sign_pub),
                    crate::codec::unb64_array::<32>(&member.box_pub),
                ) {
                    (Some(sign), Some(boxed)) => crate::identity::fingerprint(&sign, &boxed),
                    _ => String::new(),
                },
                verified: false,
            },
        }
    }

    /// Invites someone (looked up with [`Engine::person`]) with a role.
    pub fn invite(&self, share_id: &str, username: &str, role: Role) -> Result<()> {
        if role == Role::Owner {
            return Err(CollabError::Refused("A note has one owner.".into()));
        }
        let person = self.person(username)?;
        let mut people = self.people(share_id)?;
        if people
            .iter()
            .any(|(p, _)| p.account_id == person.account_id)
        {
            return Err(CollabError::Refused(format!(
                "{} is already in the note.",
                person.username
            )));
        }
        people.push((self.pinned(&person.account_id)?, role));
        self.rotate(share_id, people)
    }

    /// Gives a member another role.
    pub fn set_role(&self, share_id: &str, account: &str, role: Role) -> Result<()> {
        if role == Role::Owner {
            return Err(CollabError::Refused("A note has one owner.".into()));
        }
        let mut people = self.people(share_id)?;
        let Some(entry) = people.iter_mut().find(|(p, _)| p.account_id == account) else {
            return Err(CollabError::Refused(
                "That person isn't in the note.".into(),
            ));
        };
        entry.1 = role;
        self.rotate(share_id, people)
    }

    /// Removes a member (or cancels an invitation). They keep what they
    /// already had; nothing written from now on reaches them.
    pub fn remove(&self, share_id: &str, account: &str) -> Result<()> {
        let mut people = self.people(share_id)?;
        let before = people.len();
        people.retain(|(p, _)| p.account_id != account);
        if people.len() == before {
            return Err(CollabError::Refused(
                "That person isn't in the note.".into(),
            ));
        }
        self.rotate(share_id, people)
    }

    /// Lets in someone who asked through a link, with the link's role.
    pub fn approve(&self, share_id: &str, account: &str) -> Result<()> {
        let view = self
            .share_view(share_id)
            .ok_or_else(|| CollabError::Refused("Open the Share sheet again.".into()))?;
        let request = view
            .requests
            .iter()
            .find(|r| r.person.account_id == account)
            .cloned()
            .ok_or_else(|| CollabError::Refused("That request is gone.".into()))?;
        let mut people = self.people(share_id)?;
        people.push((self.pinned(account)?, request.role));
        self.rotate(share_id, people)
    }

    pub fn decline_request(&self, share_id: &str, account: &str) -> Result<()> {
        self.api
            .decline_request(&self.account.caller, share_id, account)?;
        self.refresh_view(share_id)
    }

    /// Makes an invite link with a role, lasting `days` (at most 30).
    pub fn create_link(&self, share_id: &str, role: Role, days: i64) -> Result<String> {
        if role == Role::Owner {
            return Err(CollabError::Refused("A note has one owner.".into()));
        }
        let secret = LinkSecret::generate();
        let link_id = new_id();
        self.api.create_link(
            &self.account.caller,
            share_id,
            &NewLink {
                link_id: link_id.clone(),
                proof_hash: secret.proof_hash(),
                role,
                expires_at: now_ms() + days.clamp(1, MAX_LINK_DAYS) * 86_400_000,
            },
        )?;
        // The role is remembered here: approving gives the link's role,
        // whatever the server later says it was.
        self.store.set_meta(&link_meta(&link_id), role.as_str())?;
        self.refresh_view(share_id)?;
        Ok(invite_link(&self.account.caller.server, &link_id, &secret))
    }

    pub fn cancel_link(&self, share_id: &str, link_id: &str) -> Result<()> {
        self.api
            .delete_link(&self.account.caller, share_id, link_id)?;
        self.refresh_view(share_id)
    }

    /// Asks to join a note through an invite link; its owner approves.
    pub fn join(&self, link: &str) -> Result<Joined> {
        let (server, link_id, secret) = parse_invite_link(link)
            .ok_or_else(|| CollabError::Refused("That isn't a Kanso invite link.".into()))?;
        let mine = kanso_sync::origin(&self.account.caller.server).ok();
        if kanso_sync::origin(&server).ok() != mine {
            return Err(CollabError::Refused(
                "That link is for a note on another Kanso server.".into(),
            ));
        }
        self.identity()?;
        self.api
            .join(&self.account.caller, &link_id, &secret.proof())
    }

    /// Stops sharing a note this account owns. Everyone keeps a private copy
    /// of its text as it is.
    pub fn stop(&self, share_id: &str) -> Result<()> {
        match self.api.stop(&self.account.caller, share_id) {
            Ok(()) | Err(CollabError::Api { status: 404, .. }) => {}
            Err(err) => return Err(err),
        }
        self.keep_private_copy(share_id)
    }

    /// Leaves a note someone else shared, keeping a private copy.
    pub fn leave(&self, share_id: &str) -> Result<()> {
        match self.api.leave(&self.account.caller, share_id) {
            Ok(()) | Err(CollabError::Api { status: 404, .. }) => {}
            Err(err) => return Err(err),
        }
        self.keep_private_copy(share_id)
    }

    /// The note stops being shared here: its text stays, as a private note.
    pub(crate) fn keep_private_copy(&self, share_id: &str) -> Result<()> {
        self.unload(share_id);
        if let Some(note) = self.store.shared_note(share_id)?
            && let Some(local) = self.store.note(&note.note_id)?
        {
            self.store.write_shared_text(&SharedText {
                note_id: &note.note_id,
                title: &local.title,
                body: &local.body,
                mark: None,
                new_in: &local.book_id,
                expected_rev: None,
                authors: &[],
            })?;
        }
        self.store.forget_shared_note(share_id)?;
        lock(&self.views).remove(share_id);
        Ok(())
    }

    /// Accepts an invitation: the note comes in on the next sync.
    pub fn accept(&self, share_id: &str) -> Result<()> {
        self.api.accept(&self.account.caller, share_id)?;
        self.refresh()?;
        self.sync_note(share_id).map(|_| ())
    }

    pub fn decline(&self, share_id: &str) -> Result<()> {
        self.api.leave(&self.account.caller, share_id)?;
        lock(&self.invitations).retain(|i| i.share_id != share_id);
        Ok(())
    }

    /// Invitations waiting for this account, as last refreshed.
    pub fn invitations(&self) -> Vec<Invitation> {
        lock(&self.invitations).clone()
    }

    /// A shared note as the Share sheet shows it, as last refreshed.
    pub fn share_view(&self, share_id: &str) -> Option<ShareView> {
        lock(&self.views).get(share_id).cloned()
    }

    fn refresh_view(&self, share_id: &str) -> Result<()> {
        let infos = self.api.shares(&self.account.caller)?;
        if let Some(info) = infos.into_iter().find(|i| i.share_id == share_id) {
            self.take_info(&info)?;
        }
        Ok(())
    }

    /// Brings the server's view of the account's shares into the library:
    /// invitations; notes shared with it (made local the first time); new
    /// epochs' keys and lists; roles; and notes it is no longer in, which
    /// become private copies. Returns the shares that ended.
    pub fn refresh(&self) -> Result<Vec<String>> {
        let infos = self.api.shares(&self.account.caller)?;
        let mut invitations = Vec::new();
        let mut on_server = HashSet::new();
        let mut offline = None;
        for info in &infos {
            on_server.insert(info.share_id.clone());
            match self.take_info(info) {
                Ok(Some(invitation)) => invitations.push(invitation),
                Ok(None) => {}
                // Unreachable now: the rest would be too.
                Err(err) if err.is_offline() || err.status() == Some(401) => {
                    offline = Some(err);
                    break;
                }
                // One note's trouble (a changed key, a member whose account
                // is gone) doesn't stop the others.
                Err(_) => {}
            }
        }
        *lock(&self.invitations) = invitations;
        if let Some(err) = offline {
            return Err(err);
        }
        // Whoever left an owned note, declined it or whose account is gone
        // is still in its signed list: the next epoch leaves them out, so
        // no later key reaches them and nothing more is taken from them.
        for info in infos
            .iter()
            .filter(|i| i.owner == self.account.caller.account_id)
        {
            let left = lock(&self.views)
                .get(&info.share_id)
                .is_some_and(|view| view.members.iter().any(|m| m.left));
            if left && let Ok(people) = self.people(&info.share_id) {
                let _ = self.rotate(&info.share_id, people);
            }
        }
        let mut ended = Vec::new();
        for note in self.store.shared_notes()? {
            if note.status == SharedStatus::Active && !on_server.contains(&note.share_id) {
                self.keep_private_copy(&note.share_id)?;
                ended.push(note.share_id);
            }
        }
        Ok(ended)
    }

    /// Moves a note this device has to a newer epoch, or another role: in
    /// place when it is loaded, so nothing loaded goes stale.
    fn take_epoch(
        &self,
        share_id: &str,
        list_text: &str,
        list_sig: &str,
        lists: &BTreeMap<u32, MemberList>,
        role: Role,
        keys: &BTreeMap<u32, NoteKey>,
    ) -> Result<()> {
        let list = lists.values().next_back().expect("the current list is in");
        let loaded = self.loaded(share_id)?;
        let mut guard = lock(&loaded);
        for (epoch, key) in keys {
            guard.keys.insert(*epoch, key.clone());
        }
        for (epoch, opened) in lists {
            guard.lists.insert(*epoch, opened.clone());
        }
        guard.current = list.clone();
        guard.note.epoch = list.epoch;
        guard.note.list = list_text.to_owned();
        guard.note.list_sig = list_sig.to_owned();
        guard.note.role = role.as_str().into();
        guard.needs_refresh = false;
        self.persist(&mut guard)?;
        // The note's mark carries the role and the epoch (the floor the
        // account's other devices hold lists to).
        if let Some(note) = self.store.note(&guard.note.note_id)? {
            self.store.write_shared_text(&SharedText {
                note_id: &guard.note.note_id,
                title: &note.title,
                body: &note.body,
                mark: Some(Engine::mark(&guard.note)),
                new_in: &note.book_id,
                expected_rev: None,
                authors: &[],
            })?;
        }
        Ok(())
    }

    /// The owner's sign key: this account's own, pinned, or (the first
    /// time) looked up, its binding checked, and pinned.
    fn owner_key(&self, info: &ShareInfo) -> Result<[u8; 32]> {
        if info.owner == self.account.caller.account_id {
            return Ok(self.identity()?.sign_pub());
        }
        if self.store.contact(&info.owner)?.is_none() {
            let identity = self
                .api
                .lookup(&self.account.caller, &info.owner_username)?
                .ok_or_else(|| CollabError::NoSuchUser(info.owner_username.clone()))?;
            if identity.account_id != info.owner {
                return Err(CollabError::Crypto(
                    "the note's owner isn't the account the server named".into(),
                ));
            }
            if let Pinned::Changed { offered, .. } = self.pin(&identity)? {
                return Err(CollabError::KeyChanged(offered.username));
            }
        }
        self.owner_sign_pub(&info.owner)
    }

    /// The newest epoch of a note this account has seen: on this device,
    /// or (through the note's `kansoShare`, which private sync carries) on
    /// another of its devices. An older list is refused.
    fn epoch_floor(&self, share_id: &str) -> Result<u32> {
        let mut floor = self
            .store
            .shared_note(share_id)?
            .filter(|n| n.status == SharedStatus::Active)
            .map_or(0, |n| n.epoch);
        if let Some(id) = self.store.note_with_share(share_id)?
            && let Some(note) = self.store.note(&id)?
            && let Some(epoch) = note
                .extra
                .get(kanso_core::model::SHARE_FIELD)
                .and_then(|mark| mark.get("epoch"))
                .and_then(serde_json::Value::as_u64)
        {
            floor = floor.max(u32::try_from(epoch).unwrap_or(u32::MAX));
        }
        Ok(floor)
    }

    /// Takes one share's server view: an invitation, or the note brought up
    /// to date (keys, lists, role). Every list must be the owner's, signed;
    /// the current one no older than this account has seen; and every key
    /// must open with this account's role in its epoch's list and match the
    /// owner's commitment there. Keys of epochs without such a list are
    /// left out.
    pub(crate) fn take_info(&self, info: &ShareInfo) -> Result<Option<Invitation>> {
        let identity = self.identity()?;
        let me = &self.account.caller.account_id;
        let owner_pub = self.owner_key(info)?;
        let floor = self.epoch_floor(&info.share_id)?;
        let list = members::open(
            &info.list,
            &info.list_sig,
            &info.share_id,
            &owner_pub,
            floor,
        )?;
        if list.owner != info.owner {
            return Err(CollabError::List(members::ListError::BadSignature));
        }
        let role = list
            .role_of(me)
            .ok_or_else(|| CollabError::Refused("The note's list doesn't name you.".into()))?;
        let owner = self.person_of(&info.owner)?;
        if info.status == "invited" {
            // Only the owner is pinned until the invitation is accepted.
            return Ok(Some(Invitation {
                share_id: info.share_id.clone(),
                owner,
                role,
            }));
        }
        // Each member's keys pinned, checked against what their account
        // published; one that can't be checked (the account is gone) is
        // shown so and stays out of whatever this device wraps.
        for member in &list.members {
            match self.pin_member(member) {
                Err(err) if err.is_offline() || err.status() == Some(401) => return Err(err),
                _ => {}
            }
        }
        if info.owner == *me {
            lock(&self.server_members).insert(
                info.share_id.clone(),
                info.members.iter().map(|m| m.account.clone()).collect(),
            );
        }
        let mut lists = BTreeMap::new();
        let mut texts = BTreeMap::new();
        for earlier in &info.lists {
            if earlier.epoch >= list.epoch {
                continue;
            }
            if let Ok(opened) = members::open(
                &earlier.list,
                &earlier.list_sig,
                &info.share_id,
                &owner_pub,
                0,
            ) && opened.epoch == earlier.epoch
                && opened.owner == info.owner
            {
                texts.insert(
                    earlier.epoch,
                    (earlier.list.clone(), earlier.list_sig.clone()),
                );
                lists.insert(earlier.epoch, opened);
            }
        }
        texts.insert(list.epoch, (info.list.clone(), info.list_sig.clone()));
        lists.insert(list.epoch, list.clone());
        let mut keys = BTreeMap::new();
        for wrapped in &info.wraps {
            let Some(epoch_list) = lists.get(&wrapped.epoch) else {
                continue;
            };
            // An epoch from before this account came is wrapped to it
            // with the role it has now.
            let epoch_role = epoch_list.role_of(me).unwrap_or(role);
            let Some(key) = unwrap(
                &identity,
                &wrapped.wrapped,
                &info.share_id,
                wrapped.epoch,
                me,
                epoch_role,
            ) else {
                continue;
            };
            if key.commit(&info.share_id, wrapped.epoch) != epoch_list.key_commit {
                if wrapped.epoch == list.epoch {
                    return Err(CollabError::Crypto(
                        "the note's key doesn't match its owner's commitment".into(),
                    ));
                }
                continue;
            }
            keys.insert(wrapped.epoch, key);
        }
        if !keys.contains_key(&list.epoch) {
            return Err(CollabError::Crypto(
                "no key for the note's current epoch".into(),
            ));
        }
        // The epoch this account's present membership began: the same on
        // every one of its devices, so they name the note alike.
        let joined = lists
            .iter()
            .rev()
            .take_while(|(_, l)| l.role_of(me).is_some())
            .map(|(epoch, _)| *epoch)
            .last()
            .unwrap_or(list.epoch);
        lists.retain(|epoch, _| keys.contains_key(epoch));
        for (epoch, key) in &keys {
            let (text, sig) = &texts[epoch];
            self.store.put_collab_key(
                &info.share_id,
                *epoch,
                &vault_seal(&self.account.roots.vault_key, &info.share_id, *epoch, key),
                Some((text.as_str(), sig.as_str())),
            )?;
        }
        match self.store.shared_note(&info.share_id)? {
            Some(local) if local.status == SharedStatus::Active => {
                if local.epoch < list.epoch || local.role != role.as_str() {
                    self.take_epoch(
                        &info.share_id,
                        &info.list,
                        &info.list_sig,
                        &lists,
                        role,
                        &keys,
                    )?;
                }
            }
            _ => {
                // New here. The owner's other devices use the note private
                // sync brings; everyone else the note's own ID, or a fresh
                // one when a private copy from an earlier time has it.
                let note_id = match self.store.note_with_share(&info.share_id)? {
                    Some(id) => id,
                    None if info.owner == *me => return Ok(None),
                    // A private copy from an earlier time holds the note's
                    // own ID: this membership gets one named by the epoch it
                    // began, which every device of the account picks alike.
                    None => {
                        let id = shared_note_id(&info.share_id);
                        let taken = self.store.note(&id)?.is_some_and(|note| {
                            note.extra
                                .get(kanso_core::model::SHARE_FIELD)
                                .and_then(|mark| mark.get("shareId"))
                                .and_then(serde_json::Value::as_str)
                                != Some(info.share_id.as_str())
                        });
                        if taken { format!("{id}.{joined}") } else { id }
                    }
                };
                self.store.put_shared_note(&SharedNote {
                    share_id: info.share_id.clone(),
                    note_id,
                    owner_id: info.owner.clone(),
                    owner_name: info.owner_username.clone(),
                    role: role.as_str().into(),
                    status: SharedStatus::Active,
                    kind: SharedKind::parse(list.kind.as_deref().unwrap_or("text")),
                    epoch: list.epoch,
                    list: info.list.clone(),
                    list_sig: info.list_sig.clone(),
                    state: Vec::new(),
                    applied_seq: 0,
                    since_snapshot: 0,
                    bytes_since_snapshot: 0,
                    counter: 0,
                    needs_snapshot: false,
                    updated_at: now_ms(),
                })?;
                self.unload(&info.share_id);
            }
        }
        // The Share sheet's view.
        let local = self.store.shared_note(&info.share_id)?;
        let members = list
            .members
            .iter()
            .map(|m| {
                let invited = info
                    .members
                    .iter()
                    .any(|s| s.account == m.account && s.status == "invited");
                // Only the owner's view lists who is on the server.
                let left = info.owner == *me
                    && m.role != Role::Owner
                    && !info.members.iter().any(|s| s.account == m.account);
                Ok(MemberView {
                    person: self.member_person(m),
                    role: m.role,
                    invited,
                    left,
                    unverified: !self.checked(m),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let links: Vec<LinkView> = info
            .links
            .iter()
            .map(|l| LinkView {
                link_id: l.link_id.clone(),
                role: l.role,
                expires_at: l.expires_at,
            })
            .collect();
        let mut requests = Vec::new();
        for request in &info.requests {
            let person = match self.pin(&request.identity()) {
                Ok(Pinned::Known(_)) => self.person_of(&request.account)?,
                _ => continue,
            };
            let role = self
                .store
                .meta(&link_meta(&request.link_id))?
                .and_then(|role| Role::parse(&role))
                .unwrap_or(request.role);
            requests.push(RequestView {
                person,
                link_id: request.link_id.clone(),
                role,
                at: request.at,
            });
        }
        if let Some(local) = local {
            lock(&self.views).insert(
                info.share_id.clone(),
                ShareView {
                    share_id: info.share_id.clone(),
                    note_id: local.note_id,
                    role,
                    owner,
                    members,
                    links,
                    requests,
                    epoch: list.epoch,
                },
            );
        }
        Ok(None)
    }
}
