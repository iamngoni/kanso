//! Shared notebooks, the owner's and the member's side (issue #390 › Shared
//! notebooks): sharing an existing notebook, joining one, and checking the
//! notebook's chain of signed member lists against what this device already
//! trusts. [`super::book_sync`] pulls and pushes.
//!
//! A shared notebook is one manifest log and one log per note, all under
//! the notebook key of the epoch they were written in. This device keeps,
//! per notebook, the checked chain head, the keys it holds (the history
//! links give the earlier epochs' keys), the CRDT state of every log and the
//! position it merged up to; the text of each note is also written to the
//! library as an ordinary note carrying a `kansoShare` mark, so search and
//! everything else that reads notes keeps working.

use std::collections::BTreeMap;

use kanso_core::{DocType, now_ms};
use kanso_store::{BookStream, Origin, Put, SharedBook, SharedStatus, SharedText};
use serde_json::{Value, json};

use super::{Engine, STATE_LIMIT};
use crate::api::{
    AccountWrap, BookInfo, BookListEntry, CollabError, MANIFEST_STREAM, NewBook, Result,
};
use crate::book::{unwrap, vault_open, vault_seal, walk_history, wrap};
use crate::book_manifest::ManifestDoc;
use crate::book_members::{self, MAX_BOOK_MEMBERS, open_genesis, verify_chain};
use crate::book_members_model::{
    BookChainHead, BookListError, BookMemberList, BookPin, SignedBookList,
};
use crate::book_model::{BookKey, BookProgress, BookShared, BookSync, JoinedBook, StoredChain};
use crate::codec::{b64, new_id, unb64};
use crate::doc::CollabDoc;
use crate::envelope;
use crate::identity::Verified;
use crate::keys::NoteKey;
use crate::model::{Header, Kind, Member, Role};

/// Notes uploaded per batch while sharing a notebook.
pub const SHARE_BATCH: usize = 50;

/// The notebook fields, besides its name, a shared notebook carries in its
/// manifest (as JSON text, one key each).
pub const BOOK_FIELDS: [&str; 4] = [
    "kansoProperties",
    "kansoBoard",
    "kansoSchedule",
    "kansoTemplates",
];

/// A notebook as this device holds it for one step: its row, the keys it
/// has and its checked chain.
pub(crate) struct Ctx {
    pub(crate) row: SharedBook,
    pub(crate) keys: BTreeMap<u32, BookKey>,
    pub(crate) chain: StoredChain,
}

impl Ctx {
    /// This account's role in the notebook now; nothing if it was removed.
    pub(crate) fn role(&self, me: &str) -> Option<Role> {
        self.chain.head.list.role_of(me)
    }

    /// Where an entry written under `epoch` at `seq` stops counting: the
    /// next epoch began at its list's `from_seq`.
    pub(crate) fn past_its_epoch(&self, epoch: u32, seq: u64) -> bool {
        self.chain
            .epochs
            .get(&(epoch + 1))
            .and_then(|list| list.from_seq)
            .is_some_and(|from| seq > from)
    }
}

/// The key a log's envelopes are sealed with in an epoch.
pub(crate) fn stream_key(key: &BookKey, share_id: &str, stream: &str) -> NoteKey {
    if stream == MANIFEST_STREAM {
        key.manifest_key()
    } else {
        key.note_key(share_id, stream)
    }
}

/// The note a log's text goes to in a library that has no note for it: the
/// same on every device of an account.
pub(crate) fn book_note_id(stream: &str) -> String {
    format!("note:b{stream}")
}

/// The notebook a joined notebook is in a library that didn't share it.
pub(crate) fn joined_book_id(share_id: &str) -> String {
    format!("book:b{share_id}")
}

/// The `kansoShare` a note of a shared notebook carries.
pub(crate) fn book_mark(row: &SharedBook, stream: &str) -> Value {
    json!({
        "shareId": row.share_id,
        "role": row.role,
        "owner": row.owner_id,
        "ownerName": row.owner_name,
        "book": true,
        "stream": stream,
    })
}

fn signed(entries: &[BookListEntry]) -> Vec<SignedBookList> {
    entries
        .iter()
        .map(|l| SignedBookList {
            list: l.list.clone(),
            sig: l.list_sig.clone(),
        })
        .collect()
}

fn parse_list(entry: &BookListEntry) -> Option<BookMemberList> {
    serde_json::from_slice(&unb64(&entry.list)?).ok()
}

impl Engine {
    // ------------------------------------------------------------- context

    /// A joined notebook's row, keys and chain, loaded from the library.
    pub(crate) fn book_ctx(&self, share_id: &str) -> Result<Ctx> {
        let row = self
            .store
            .shared_book(share_id)?
            .filter(|b| b.status == SharedStatus::Active)
            .ok_or_else(|| CollabError::Refused("The notebook isn't shared here.".into()))?;
        let mut keys = BTreeMap::new();
        for sealed in self.store.book_keys(share_id)? {
            if let Some(key) = vault_open(
                &self.account.roots.vault_key,
                share_id,
                sealed.epoch,
                &sealed.sealed,
            ) {
                keys.insert(sealed.epoch, key);
            }
        }
        let chain: StoredChain = serde_json::from_str(&row.chain)
            .map_err(|_| CollabError::Protocol("a notebook's saved chain can't be read".into()))?;
        Ok(Ctx { row, keys, chain })
    }

    pub(crate) fn save_ctx(&self, ctx: &mut Ctx) -> Result<()> {
        ctx.row.chain = serde_json::to_string(&ctx.chain).expect("a chain serializes");
        ctx.row.updated_at = now_ms();
        self.store.put_shared_book(&ctx.row)?;
        Ok(())
    }

    /// Seals `plaintext` as this device's next envelope of `kind` for a log,
    /// under the notebook's current epoch.
    pub(crate) fn seal_book(
        &self,
        ctx: &mut Ctx,
        stream: &str,
        kind: Kind,
        base_seq: Option<u64>,
        plaintext: &[u8],
    ) -> Result<Vec<u8>> {
        let identity = self.identity()?;
        let key = ctx
            .keys
            .get(&ctx.row.epoch)
            .ok_or_else(|| CollabError::Crypto("this device has no key for the notebook".into()))?
            .clone();
        ctx.row.counter += 1;
        let header = Header {
            kind,
            epoch: ctx.row.epoch,
            author_account: self.account.caller.account_id.clone(),
            author_device: self.account.caller.device_id.clone(),
            counter: ctx.row.counter,
        };
        let sealed = envelope::seal(
            &identity,
            &stream_key(&key, &ctx.row.share_id, stream),
            &ctx.row.share_id,
            &header,
            base_seq,
            plaintext,
        );
        self.save_ctx(ctx)?;
        Ok(sealed)
    }

    // --------------------------------------------------------------- chain

    /// Every list of a notebook's chain, from rev 1, with the answer's
    /// other fields as the first page gave them.
    fn fetch_chain(&self, share_id: &str) -> Result<(BookInfo, Vec<BookListEntry>)> {
        let caller = &self.account.caller;
        let mut info = self.api.book(caller, share_id, 1)?;
        let mut lists = std::mem::take(&mut info.lists);
        let mut more = info.more;
        while more {
            let next = lists.last().map_or(1, |l| l.rev + 1);
            let page = self.api.book(caller, share_id, next)?;
            if page.lists.is_empty() {
                return Err(CollabError::Protocol(
                    "an empty page of member lists".into(),
                ));
            }
            more = page.more;
            lists.extend(page.lists);
        }
        for (i, entry) in lists.iter().enumerate() {
            if entry.rev as usize != i + 1 {
                return Err(CollabError::Protocol(
                    "the member lists don't follow each other".into(),
                ));
            }
        }
        Ok((info, lists))
    }

    /// Checks a chain back to the owner's genesis list. `Ok(None)` when the
    /// server's chain is shorter than the pin: the caller keeps its head.
    /// A different history at a pinned revision is an error that stays one.
    fn check_chain(
        &self,
        share_id: &str,
        lists: &[BookListEntry],
        owner_sign_pub: &[u8; 32],
        pin: Option<&BookPin>,
        log_head: u64,
    ) -> Result<Option<StoredChain>> {
        let head = match verify_chain(&signed(lists), share_id, owner_sign_pub, pin, log_head) {
            Ok(head) => head,
            Err(BookListError::Stale) => return Ok(None),
            Err(err) => return Err(err.into()),
        };
        let mut epochs = BTreeMap::new();
        for entry in lists {
            let list = parse_list(entry).ok_or(BookListError::Malformed)?;
            epochs.insert(list.epoch, list);
        }
        Ok(Some(StoredChain { head, epochs }))
    }

    /// The newest log sequence the server has for the notebook (the manifest
    /// page's `head`), never lower than `known`.
    fn log_head(&self, share_id: &str, known: u64) -> Result<u64> {
        let page = self
            .api
            .book_log(&self.account.caller, share_id, MANIFEST_STREAM, known, 1)?;
        Ok(page.head.max(known))
    }

    /// Opens the wraps and history links the server gave for the keys this
    /// device lacks, each checked against its epoch's key commitment, and
    /// keeps them sealed in the library.
    fn take_keys(
        &self,
        share_id: &str,
        info: &BookInfo,
        chain: &StoredChain,
        keys: &mut BTreeMap<u32, BookKey>,
    ) -> Result<()> {
        let identity = self.identity()?;
        let me = &self.account.caller.account_id;
        for w in &info.wraps {
            if keys.contains_key(&w.epoch) {
                continue;
            }
            let Some(list) = chain.epochs.get(&w.epoch) else {
                continue;
            };
            let Some(role) = list.role_of(me) else {
                continue;
            };
            let Some(key) = unwrap(&identity, &w.wrapped, share_id, w.epoch, me, role) else {
                continue;
            };
            if key.commit(share_id, w.epoch) != list.key_commit {
                return Err(CollabError::Crypto(
                    "the notebook's key isn't the one its member list names".into(),
                ));
            }
            keys.insert(w.epoch, key);
        }
        let top = chain.head.list.epoch;
        if let Some(newest) = keys.get(&top).cloned() {
            let links: BTreeMap<u32, Vec<u8>> = info
                .epochs
                .iter()
                .filter_map(|l| Some((l.epoch, unb64(&l.history_link)?)))
                .collect();
            let history =
                walk_history(share_id, top, &newest, |e| links.get(&e).map(Vec::as_slice));
            for (epoch, key) in history.keys {
                // A link that opens to a key its epoch's list doesn't name
                // ends the walk's worth there.
                let named = chain
                    .epochs
                    .get(&epoch)
                    .is_none_or(|list| key.commit(share_id, epoch) == list.key_commit);
                if named {
                    keys.entry(epoch).or_insert(key);
                }
            }
        }
        for (epoch, key) in keys.iter() {
            self.store.put_book_key(
                share_id,
                *epoch,
                &vault_seal(&self.account.roots.vault_key, share_id, *epoch, key),
            )?;
        }
        Ok(())
    }

    /// Pins another member's keys. Only keys that differ from the ones pinned
    /// stop a sync: someone whose account is gone, or who can't be looked up
    /// now, doesn't (the signed chain vouches for the keys).
    fn pin_other(&self, member: &Member) -> Result<()> {
        match self.pin_member(member) {
            Err(err @ CollabError::KeyChanged(_)) => Err(err),
            _ => Ok(()),
        }
    }

    /// Brings the notebook's chain up to the server's: checked against the
    /// head this device trusts (an older chain keeps it; a forked one is an
    /// error), with the keys of any new epoch. Marks the notebook removed
    /// when the chain no longer lists this account.
    pub(crate) fn refresh_chain(&self, ctx: &mut Ctx) -> Result<()> {
        let share_id = ctx.row.share_id.clone();
        let (info, lists) = self.fetch_chain(&share_id)?;
        let owner_pub = self.owner_sign_pub(&ctx.row.owner_id)?;
        let log_head = self.log_head(&share_id, ctx.row.head)?;
        ctx.row.head = ctx.row.head.max(log_head);
        let pin = ctx.chain.head.pin();
        let Some(chain) = self.check_chain(&share_id, &lists, &owner_pub, Some(&pin), log_head)?
        else {
            return Ok(());
        };
        let me = self.account.caller.account_id.clone();
        if chain.head.list.member(&me).is_none() {
            ctx.chain = chain;
            ctx.row.status = SharedStatus::Removed;
            return self.save_ctx(ctx);
        }
        self.take_keys(&share_id, &info, &chain, &mut ctx.keys)?;
        let epoch = chain.head.list.epoch;
        if !ctx.keys.contains_key(&epoch) {
            return Err(CollabError::Crypto(
                "this device has no key for the notebook's newest epoch".into(),
            ));
        }
        for member in &chain.head.list.members {
            if member.account != me {
                self.pin_other(member)?;
            }
        }
        ctx.row.epoch = epoch;
        ctx.row.role = chain
            .head
            .list
            .role_of(&me)
            .unwrap_or(Role::View)
            .as_str()
            .to_owned();
        ctx.chain = chain;
        self.save_ctx(ctx)
    }

    // ------------------------------------------------------------- joining

    /// Joins a notebook this account is invited to, or opens one it is
    /// already in on this device (its owner's other device, say): the chain
    /// is checked back to the owner's genesis list, the notebook key opened
    /// and checked against its commitment, and the notebook pulled.
    pub fn join_book(&self, share_id: &str) -> Result<JoinedBook> {
        if let Some(row) = self.store.shared_book(share_id)?
            && row.status == SharedStatus::Active
        {
            let sync = self.sync_book(share_id)?;
            return Ok(JoinedBook {
                share_id: share_id.to_owned(),
                book_id: row.book_id,
                notes: sync.notes_written,
            });
        }
        let caller = &self.account.caller;
        let (info, lists) = self.fetch_chain(share_id)?;
        let genesis = lists
            .first()
            .and_then(parse_list)
            .ok_or(BookListError::Malformed)?;
        let owner_entry = genesis
            .members
            .iter()
            .find(|m| m.role == Role::Owner)
            .ok_or(BookListError::Malformed)?;
        let owner = self.pin_member(owner_entry)?;
        // The whole chain first, without the log bound a member could read:
        // nothing is accepted for a notebook whose chain doesn't hold.
        let early = self
            .check_chain(share_id, &lists, &owner.sign_pub, None, u64::MAX)?
            .ok_or(BookListError::Empty)?;
        let role = early
            .head
            .list
            .role_of(&caller.account_id)
            .ok_or_else(|| CollabError::Refused("You aren't in that notebook.".into()))?;
        for member in &early.head.list.members {
            if member.account != caller.account_id {
                self.pin_other(member)?;
            }
        }
        if info.status == "invited" {
            self.api.accept_book(caller, share_id)?;
        }
        let head = self.log_head(share_id, 0)?;
        let chain = self
            .check_chain(share_id, &lists, &owner.sign_pub, None, head)?
            .ok_or(BookListError::Empty)?;
        let mut keys = BTreeMap::new();
        self.take_keys(share_id, &info, &chain, &mut keys)?;
        let epoch = chain.head.list.epoch;
        if !keys.contains_key(&epoch) {
            return Err(CollabError::Crypto(
                "this device has no key for the notebook".into(),
            ));
        }
        // A device of the owner's that has the notebook's notes already
        // (private sync brought them) keeps them where they are.
        let book_id = self
            .store
            .notes_marked_for_book(share_id)?
            .into_iter()
            .map(|(_, _, book)| book)
            .find(|book| !book.is_empty())
            .unwrap_or_else(|| joined_book_id(share_id));
        let mut ctx = Ctx {
            row: SharedBook {
                share_id: share_id.to_owned(),
                book_id: book_id.clone(),
                owner_id: info.owner.clone(),
                owner_name: owner.username.clone(),
                role: role.as_str().to_owned(),
                status: SharedStatus::Active,
                epoch,
                chain: String::new(),
                head,
                counter: 0,
                uploading: false,
                updated_at: now_ms(),
            },
            keys,
            chain,
        };
        self.save_ctx(&mut ctx)?;
        let sync = self.sync_book(share_id)?;
        Ok(JoinedBook {
            share_id: share_id.to_owned(),
            book_id,
            notes: sync.notes_written,
        })
    }

    // ------------------------------------------------------------- sharing

    /// Why a notebook can't be shared, if it can't, and what sharing it
    /// would leave out.
    pub fn check_book_shareable(&self, book_id: &str) -> Result<BookCheck> {
        let book = self
            .store
            .books()?
            .into_iter()
            .find(|b| b.id == book_id)
            .ok_or_else(|| CollabError::NotShareable("The notebook no longer exists.".into()))?;
        if self.store.locked_books()?.contains(book_id) {
            return Err(CollabError::NotShareable(
                "Locked notebooks can't be shared.".into(),
            ));
        }
        if self.store.shared_book_of(book_id)?.is_some() {
            return Err(CollabError::NotShareable(
                "The notebook is already shared.".into(),
            ));
        }
        let mut check = BookCheck {
            name: book.name,
            ..BookCheck::default()
        };
        for note in self.store.notes_in_books(&[book_id.to_owned()])? {
            if self.store.note_is_locked(&note.id)? {
                check.locked.push(note.id);
            } else if note.extra.contains_key(kanso_core::model::SHARE_FIELD)
                || note.doctype == "canvas"
                || note.body.len() > STATE_LIMIT
            {
                check.left_out.push(note.title);
            } else {
                check.eligible.push(note.id);
            }
        }
        Ok(check)
    }

    /// Shares a notebook with people (by username, each with a role) and
    /// returns what was done. Locked notes can't be in it: with `move_locked`
    /// they go to a private "<name> (locked)" notebook, without it sharing is
    /// refused while any exist. Notes go up in batches of [`SHARE_BATCH`],
    /// `progress` told after each; a crash or an offline spell leaves a
    /// notebook that [`Engine::resume_book_share`] finishes, skipping the
    /// notes that are up already. Their keys must already be pinned (looked
    /// up with [`Engine::person`]).
    pub fn share_book(
        &self,
        book_id: &str,
        invite: &[(String, Role)],
        move_locked: bool,
        progress: &mut dyn FnMut(BookProgress),
    ) -> Result<BookShared> {
        let check = self.check_book_shareable(book_id)?;
        if !check.locked.is_empty() && !move_locked {
            return Err(CollabError::NotShareable(format!(
                "{} locked notes can't be in a shared notebook: move them out first.",
                check.locked.len()
            )));
        }
        if invite.len() + 1 > MAX_BOOK_MEMBERS {
            return Err(CollabError::Refused(format!(
                "A notebook can be shared with at most {} people.",
                MAX_BOOK_MEMBERS - 1
            )));
        }
        let identity = self.identity()?;
        let caller = &self.account.caller;
        let me = Verified {
            account_id: caller.account_id.clone(),
            username: caller.username.clone(),
            sign_pub: identity.sign_pub(),
            box_pub: identity.box_pub(),
        };
        let mut people = vec![(me.clone(), Role::Owner)];
        for (username, role) in invite {
            if *role == Role::Owner {
                return Err(CollabError::Refused("A notebook has one owner.".into()));
            }
            let person = self.person(username)?;
            people.push((self.pinned(&person.account_id)?, *role));
        }
        let share_id = new_id();
        let key = BookKey::generate();
        let mut list = BookMemberList {
            v: 1,
            share: share_id.clone(),
            rev: 1,
            epoch: 1,
            owner: caller.account_id.clone(),
            signer: caller.account_id.clone(),
            prev: None,
            key_commit: key.commit(&share_id, 1),
            at: now_ms(),
            from_seq: None,
            members: people
                .iter()
                .map(|(p, role)| Member {
                    account: p.account_id.clone(),
                    username: p.username.clone(),
                    role: *role,
                    sign_pub: b64(&p.sign_pub),
                    box_pub: b64(&p.box_pub),
                })
                .collect(),
        };
        list.sort();
        if !book_members::is_well_formed(&list) {
            return Err(CollabError::Refused(
                "That list of people isn't possible.".into(),
            ));
        }
        let signed_list = book_members::sign(&identity, &list);
        let mut wraps = Vec::new();
        for (person, role) in &people {
            let wrapped = wrap(
                &key,
                &share_id,
                1,
                &person.account_id,
                *role,
                &person.box_pub,
            )
            .ok_or_else(|| {
                CollabError::Crypto(format!("{}'s key can't be used", person.username))
            })?;
            wraps.push(AccountWrap {
                account: person.account_id.clone(),
                wrapped,
                epoch: None,
            });
        }
        self.api.create_book(
            caller,
            &NewBook {
                share_id: share_id.clone(),
                list: signed_list.list.clone(),
                list_sig: signed_list.sig.clone(),
                wraps,
            },
        )?;
        let head: BookChainHead = open_genesis(&signed_list, &share_id, &identity.sign_pub())?;
        let chain = StoredChain {
            epochs: BTreeMap::from([(1, list)]),
            head,
        };
        self.store.put_book_key(
            &share_id,
            1,
            &vault_seal(&self.account.roots.vault_key, &share_id, 1, &key),
        )?;
        let mut ctx = Ctx {
            row: SharedBook {
                share_id: share_id.clone(),
                book_id: book_id.to_owned(),
                owner_id: caller.account_id.clone(),
                owner_name: caller.username.clone(),
                role: Role::Owner.as_str().to_owned(),
                status: SharedStatus::Active,
                epoch: 1,
                chain: String::new(),
                head: 0,
                counter: 0,
                uploading: true,
                updated_at: now_ms(),
            },
            keys: BTreeMap::from([(1, key)]),
            chain,
        };
        self.save_ctx(&mut ctx)?;
        let mut moved_locked = 0;
        if !check.locked.is_empty() {
            let private = self.store.put(
                DocType::Book,
                json!({"name": format!("{} (locked)", check.name)}),
                Put::Upsert,
                Origin::Collab,
            )?;
            for note_id in &check.locked {
                self.store.put(
                    DocType::Note,
                    json!({"_id": note_id, "bookId": private.id}),
                    Put::Merge,
                    Origin::Collab,
                )?;
                moved_locked += 1;
            }
        }
        self.upload_book(&share_id, progress)?;
        let shared = self
            .store
            .book_streams(&share_id)?
            .iter()
            .filter(|s| s.note_id.is_some())
            .count();
        Ok(BookShared {
            share_id,
            shared,
            moved_locked,
            left_out: check.left_out,
        })
    }

    /// Finishes sharing a notebook that stopped part way: the notes not up
    /// yet go up, the ones that are stay.
    pub fn resume_book_share(
        &self,
        share_id: &str,
        progress: &mut dyn FnMut(BookProgress),
    ) -> Result<()> {
        self.upload_book(share_id, progress)
    }

    /// Gives every note of the notebook that has no log yet one (its state
    /// as the library has it), marks it shared, and lists it in the manifest.
    /// Returns the logs made.
    pub(crate) fn adopt_local_notes(&self, ctx: &mut Ctx) -> Result<Vec<BookStream>> {
        let share_id = ctx.row.share_id.clone();
        let have: std::collections::HashSet<String> = self
            .store
            .book_streams(&share_id)?
            .into_iter()
            .filter_map(|s| s.note_id)
            .collect();
        let mut made = Vec::new();
        for note in self
            .store
            .notes_in_books(std::slice::from_ref(&ctx.row.book_id))?
        {
            let marked_here = note
                .extra
                .get(kanso_core::model::SHARE_FIELD)
                .and_then(|m| m.get("shareId"))
                .and_then(Value::as_str)
                == Some(share_id.as_str());
            if have.contains(&note.id)
                || (note.extra.contains_key(kanso_core::model::SHARE_FIELD) && !marked_here)
                || note.doctype == "canvas"
                || note.body.len() > STATE_LIMIT
                || self.store.note_is_locked(&note.id)?
            {
                continue;
            }
            let stream = new_id();
            let doc = CollabDoc::from_note_text(&note.title, &note.body);
            let row = BookStream {
                share_id: share_id.clone(),
                stream: stream.clone(),
                note_id: Some(note.id.clone()),
                state: doc.encode_state(),
                applied_seq: 0,
                on_server: false,
                base: serde_json::to_string(&[&note.title, &note.body]).expect("serializes"),
            };
            self.store.put_book_stream(&row)?;
            self.store.write_shared_text(&SharedText {
                note_id: &note.id,
                title: &note.title,
                body: &note.body,
                mark: Some(book_mark(&ctx.row, &stream)),
                new_in: &note.book_id,
                expected_rev: None,
                authors: &[],
            })?;
            made.push(row);
        }
        Ok(made)
    }

    /// Puts a log's first snapshot on the server, unless it is there (a
    /// crash after the upload, before this device wrote it down).
    pub(crate) fn upload_snapshot(&self, ctx: &mut Ctx, stream: &mut BookStream) -> Result<()> {
        let caller = &self.account.caller;
        let page = self
            .api
            .book_log(caller, &ctx.row.share_id, &stream.stream, 0, 1)?;
        if page.snapshot.is_none() && page.entries.is_empty() {
            let sealed =
                self.seal_book(ctx, &stream.stream, Kind::Snapshot, Some(0), &stream.state)?;
            match self
                .api
                .book_snapshot(caller, &ctx.row.share_id, &stream.stream, 0, &sealed)
            {
                Ok(_) => {}
                // Written meanwhile (by this device, before a crash): up.
                Err(err) if err.code() == Some("head_moved") => {}
                Err(err) => return Err(err),
            }
        }
        stream.on_server = true;
        self.store.put_book_stream(stream)?;
        Ok(())
    }

    /// Uploads a notebook being shared: its manifest, then its notes in
    /// batches, each batch listed in the manifest once it is up.
    fn upload_book(&self, share_id: &str, progress: &mut dyn FnMut(BookProgress)) -> Result<()> {
        let mut ctx = self.book_ctx(share_id)?;
        let me = self.account.caller.account_id.clone();
        self.adopt_local_notes(&mut ctx)?;
        let mut manifest_row = match self.store.book_stream(share_id, MANIFEST_STREAM)? {
            Some(row) => row,
            None => {
                let row = BookStream {
                    share_id: share_id.to_owned(),
                    stream: MANIFEST_STREAM.to_owned(),
                    note_id: None,
                    state: ManifestDoc::new().encode_state(),
                    applied_seq: 0,
                    on_server: true,
                    base: String::new(),
                };
                self.store.put_book_stream(&row)?;
                row
            }
        };
        let manifest = ManifestDoc::load(&manifest_row.state)
            .map_err(|e| CollabError::Crypto(e.to_string()))?;
        if let Some(update) = manifest.set_fields(&self.local_fields(&ctx)?) {
            self.store
                .push_book_outbox(share_id, MANIFEST_STREAM, &update)?;
        }
        manifest_row.base = serde_json::to_string(&self.local_fields(&ctx)?).expect("serializes");
        let all: Vec<BookStream> = self
            .store
            .book_streams(share_id)?
            .into_iter()
            .filter(|s| s.note_id.is_some())
            .collect();
        let total = all.len();
        let mut done = all.iter().filter(|s| s.on_server).count();
        progress(BookProgress { done, total });
        // Every note that is up is listed, those of an earlier attempt too
        // (a crash may have come between a note going up and its listing).
        let mut listed: Vec<(String, i64)> = all
            .iter()
            .filter(|s| s.on_server)
            .map(|s| (s.stream.clone(), now_ms()))
            .collect();
        let pending: Vec<BookStream> = all.into_iter().filter(|s| !s.on_server).collect();
        if let Some(update) = manifest.add_notes(&listed, &me) {
            self.store
                .push_book_outbox(share_id, MANIFEST_STREAM, &update)?;
        }
        for batch in pending.chunks(SHARE_BATCH) {
            for stream in batch {
                let mut stream = stream.clone();
                self.upload_snapshot(&mut ctx, &mut stream)?;
                listed.push((stream.stream.clone(), now_ms()));
            }
            if let Some(update) = manifest.add_notes(&listed, &me) {
                self.store
                    .push_book_outbox(share_id, MANIFEST_STREAM, &update)?;
            }
            manifest_row.state = manifest.encode_state();
            self.store.put_book_stream(&manifest_row)?;
            self.push_book(&mut ctx, &mut BookSync::default())?;
            done += batch.len();
            progress(BookProgress { done, total });
        }
        manifest_row.state = manifest.encode_state();
        self.store.put_book_stream(&manifest_row)?;
        self.push_book(&mut ctx, &mut BookSync::default())?;
        ctx.row.uploading = false;
        self.save_ctx(&mut ctx)?;
        // Everything is up: the notes' logs and the manifest are read back
        // the way any member reads them, so the sequence numbers are known.
        self.sync_book(share_id)?;
        Ok(())
    }

    /// The notebook's fields as the library has them.
    pub(crate) fn local_fields(&self, ctx: &Ctx) -> Result<BTreeMap<String, String>> {
        let mut out = BTreeMap::new();
        let Some(book) = self
            .store
            .books()?
            .into_iter()
            .find(|b| b.id == ctx.row.book_id)
        else {
            return Ok(out);
        };
        out.insert("name".to_owned(), book.name);
        for field in BOOK_FIELDS {
            if let Some(value) = book.extra.get(field) {
                out.insert(field.to_owned(), value.to_string());
            }
        }
        Ok(out)
    }

    /// Shared notebooks this library holds.
    pub fn shared_books(&self) -> Result<Vec<SharedBook>> {
        Ok(self.store.shared_books()?)
    }
}

/// What sharing a notebook would do, found before doing it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BookCheck {
    pub name: String,
    /// Notes that would go in.
    pub eligible: Vec<String>,
    /// Locked notes, which can't (see [`Engine::share_book`]).
    pub locked: Vec<String>,
    /// Titles of notes left out: already shared on their own, or canvases.
    pub left_out: Vec<String>,
}
