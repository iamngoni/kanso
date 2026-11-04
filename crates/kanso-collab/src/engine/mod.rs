//! Shared notes against a library (docs/ACCOUNTS.md › Collaboration): the
//! account's identity, the people it shares with, sharing and its epochs,
//! the note's log (pull, push, compaction), the open note's editor, comments
//! and mentions. [`live`] keeps an open note's room connected.
//!
//! Every shared note has one lock ([`Loaded`]); network calls happen outside
//! it, so the editor's [`Engine::sync_editor`] never waits on the network.
//!
//! **The text invariant.** For each loaded note the engine remembers `base`,
//! the text it last read from or wrote to the library. Whatever differs
//! between the library's text and `base` is a local edit (the local API,
//! MCP, any writer but the open editor); it is rebased onto whatever the
//! CRDT merged since `base` ([`rebase`]) and becomes Yrs operations. The
//! open editor has a base of its own, `editor_base`, the text it last got
//! from [`Engine::sync_editor`], and its edits are taken the same way. A
//! local edit therefore never undoes a remote one, and the library's text is
//! the CRDT's after every merge. While a shared note is open, the editor's
//! text reaches the library through [`Engine::sync_editor`] only.

mod canvas;
pub mod live;
mod log;
mod people;
mod sharing;

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, MutexGuard};

use kanso_core::canvas::Canvas;
use kanso_core::now_ms;
use kanso_store::{SharedNote, SharedStatus, SharedText, Store, StoreError};
use serde_json::{Value, json};

use crate::api::{Caller, CollabApi, CollabError, Result};
use crate::codec::{new_id, unb64_array};
use crate::comments::{self, Logged};
use crate::doc::{CollabDoc, Field, remap, splices};
use crate::envelope;
use crate::identity::Identity;
use crate::keys::{NoteKey, vault_open};
use crate::members;
use crate::model::{
    Anchor, CURSOR_COLOURS, CanvasPointer, CommentEvent, CommentOp, CommentView, EditorMerge,
    Header, Invitation, Kind, Member, MemberList, PresenceMessage, RemoteCursor, Role, ShareView,
};
use crate::shared_doc::SharedDoc;

pub use live::LiveHandle;

/// Updates or bytes since the last snapshot after which an editor writes one.
pub const COMPACT_AFTER_UPDATES: u64 = 500;
pub const COMPACT_AFTER_BYTES: u64 = 1024 * 1024;
/// The most a shared note's state may hold: a snapshot of it (with its
/// envelope) must stay under the server's 4 MiB.
pub const STATE_LIMIT: usize = envelope::MAX_SNAPSHOT - 4096;
/// The most text one update inserts: well under the server's 256 KiB cap
/// on an update, whatever Yrs adds around it.
pub const UPDATE_PART: usize = 64 * 1024;
/// A remote cursor not heard from for this long is no longer shown.
pub const CURSOR_TTL_MS: i64 = 30_000;

/// The signed-in account the engine works for: who it is to the server, and
/// the keys its data key gives.
pub struct Account {
    pub caller: Caller,
    pub roots: kanso_sync::CollabRoots,
}

impl Account {
    pub fn of(session: &kanso_sync::Session) -> Account {
        Account {
            caller: Caller::of(session),
            roots: session.collab_roots(),
        }
    }
}

/// One shared note, loaded: its CRDT, keys and lists, and what the live room
/// delivered that isn't merged yet.
pub(crate) struct Loaded {
    pub(crate) note: SharedNote,
    pub(crate) doc: SharedDoc,
    pub(crate) keys: BTreeMap<u32, NoteKey>,
    pub(crate) lists: BTreeMap<u32, MemberList>,
    /// The list of `note.epoch`.
    pub(crate) current: MemberList,
    /// The text the engine last read from or wrote to the library.
    pub(crate) base: (String, String),
    /// The text the open editor last got from `sync_editor`.
    pub(crate) editor_base: Option<(String, String)>,
    /// Stored entries the room delivered ahead of the log, by sequence.
    pub(crate) inbox: BTreeMap<u64, Vec<u8>>,
    /// The editor has the note open: merges wait for `sync_editor`.
    pub(crate) attached: bool,
    /// Other people's cursors, by (account, device).
    pub(crate) cursors: HashMap<(String, String), (PresenceMessage, i64)>,
    /// This device's caret, to send as presence: (anchor, head) offsets.
    pub(crate) caret: Option<(usize, usize)>,
    /// Outbox entries sent on the live room: counter → outbox ID, until the
    /// room echoes them.
    pub(crate) in_flight: HashMap<u64, i64>,
    /// A snapshot the log gave that isn't merged yet: (base_seq, envelope).
    pub(crate) pending_snapshot: Option<(u64, Vec<u8>)>,
    /// An entry came under an epoch this device has no key for yet.
    pub(crate) needs_refresh: bool,
    /// This account's comments the room took whose mentions aren't told
    /// yet (the editor's thread never waits on the network).
    pub(crate) pending_mentions: Vec<CommentEvent>,
    /// The note stopped being shared here: nothing writes for it again.
    pub(crate) ended: bool,
    /// The note is starting again from the log: the library isn't written
    /// until the log has merged.
    pub(crate) reloading: bool,
    /// A canvas: the canvas the open view last got from the engine, which
    /// its next edit is measured from.
    pub(crate) canvas_base: Option<Canvas>,
    /// A canvas: the CRDT changed since the library's body was written.
    pub(crate) canvas_dirty: bool,
    /// A canvas: when (ms) the library's body was last written.
    pub(crate) canvas_written_at: i64,
    /// A canvas: the projection is past what a note can hold.
    pub(crate) canvas_full: bool,
    /// A canvas: this device's pointer, to send as presence.
    pub(crate) pointer: Option<CanvasPointer>,
}

impl Loaded {
    pub(crate) fn role(&self) -> Role {
        Role::parse(&self.note.role).unwrap_or(Role::View)
    }

    /// The member who wrote under `epoch`, as that epoch's owner-signed
    /// list has them. Nobody, for an epoch whose list this device lacks.
    pub(crate) fn author(&self, epoch: u32, account: &str) -> Option<&Member> {
        self.lists.get(&epoch).and_then(|list| list.member(account))
    }

    /// The text document of a text note.
    pub(crate) fn text_doc(&self) -> &CollabDoc {
        self.doc.text().expect("a text note")
    }

    fn text(&self) -> (String, String) {
        match self.doc.text() {
            Some(doc) => (doc.text(Field::Title), doc.text(Field::Body)),
            None => (self.doc.title(), String::new()),
        }
    }
}

/// The engine of one signed-in account over one library.
pub struct Engine {
    pub(crate) store: Arc<Store>,
    pub(crate) api: Arc<dyn CollabApi>,
    pub(crate) account: Account,
    identity: Mutex<Option<Arc<Identity>>>,
    notes: Mutex<HashMap<String, Arc<Mutex<Loaded>>>>,
    pub(crate) views: Mutex<HashMap<String, ShareView>>,
    pub(crate) invitations: Mutex<Vec<Invitation>>,
    /// Who the server has in each note this account owns.
    pub(crate) server_members: Mutex<HashMap<String, std::collections::HashSet<String>>>,
    /// What the person should hear about.
    pub(crate) problems: Mutex<Vec<crate::model::Problem>>,
}

/// The `meta` key holding this account's published identity (sealed).
const IDENTITY_META: &str = "collab.identity";

pub(crate) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl Engine {
    pub fn new(store: Arc<Store>, api: Arc<dyn CollabApi>, account: Account) -> Engine {
        Engine {
            store,
            api,
            account,
            identity: Mutex::new(None),
            notes: Mutex::new(HashMap::new()),
            views: Mutex::new(HashMap::new()),
            invitations: Mutex::new(Vec::new()),
            server_members: Mutex::new(HashMap::new()),
            problems: Mutex::new(Vec::new()),
        }
    }

    pub fn caller(&self) -> &Caller {
        &self.account.caller
    }

    pub fn store(&self) -> &Arc<Store> {
        &self.store
    }

    // ------------------------------------------------------------ identity

    /// This account's identity keys: kept in the library (sealed), else
    /// fetched from the server, else made now and published.
    pub fn identity(&self) -> Result<Arc<Identity>> {
        if let Some(identity) = lock(&self.identity).clone() {
            return Ok(identity);
        }
        let caller = &self.account.caller;
        let key = &self.account.roots.identity_key;
        let kept = self
            .store
            .meta(&self.identity_meta())?
            .and_then(|json| serde_json::from_str::<crate::identity::Published>(&json).ok());
        let identity = match kept {
            Some(published) => Identity::open(key, &caller.account_id, &published)?,
            None => {
                let published = match self.api.get_identity(caller)? {
                    Some(published) => published,
                    None => {
                        let fresh = Identity::generate();
                        let published = fresh.publish(key, &caller.account_id, &caller.username);
                        match self.api.put_identity(caller, &published) {
                            Ok(()) => published,
                            // Another device published first: use theirs.
                            Err(err) if err.code() == Some("identity_exists") => self
                                .api
                                .get_identity(caller)?
                                .ok_or_else(|| CollabError::Protocol("no identity".into()))?,
                            Err(err) => return Err(err),
                        }
                    }
                };
                let identity = Identity::open(key, &caller.account_id, &published)?;
                self.store.set_meta(
                    &self.identity_meta(),
                    &serde_json::to_string(&published).expect("serializes"),
                )?;
                identity
            }
        };
        let identity = Arc::new(identity);
        *lock(&self.identity) = Some(identity.clone());
        Ok(identity)
    }

    /// Per account: a library re-bound to another account starts over.
    fn identity_meta(&self) -> String {
        format!("{IDENTITY_META}.{}", self.account.caller.account_id)
    }

    /// This account's fingerprint, for Settings › Account.
    pub fn fingerprint(&self) -> Result<String> {
        Ok(self.identity()?.fingerprint())
    }

    // --------------------------------------------------------- loaded notes

    /// A shared note's lock, loading it from the library the first time.
    pub(crate) fn loaded(&self, share_id: &str) -> Result<Arc<Mutex<Loaded>>> {
        if let Some(loaded) = lock(&self.notes).get(share_id) {
            return Ok(loaded.clone());
        }
        let note = self
            .store
            .shared_note(share_id)?
            .filter(|n| n.status == SharedStatus::Active)
            .ok_or_else(|| CollabError::Refused("The note isn't shared.".into()))?;
        let owner_key = self.owner_sign_pub(&note.owner_id)?;
        let mut keys = BTreeMap::new();
        let mut lists = BTreeMap::new();
        for sealed in self.store.collab_keys(share_id)? {
            if let Some(key) = vault_open(
                &self.account.roots.vault_key,
                share_id,
                sealed.epoch,
                &sealed.sealed,
            ) {
                keys.insert(sealed.epoch, key);
            }
            if let (Some(list), Some(sig)) = (&sealed.list, &sealed.list_sig)
                && let Ok(opened) = members::open(list, sig, share_id, &owner_key, 0)
            {
                lists.insert(sealed.epoch, opened);
            }
        }
        let current = members::open(&note.list, &note.list_sig, share_id, &owner_key, 0)?;
        lists.insert(current.epoch, current.clone());
        let doc = if note.state.is_empty() {
            SharedDoc::new(note.kind)
        } else {
            SharedDoc::load(note.kind, &note.state)
                .map_err(|e| CollabError::Crypto(e.to_string()))?
        };
        // A note new to this device takes its text from the log: what the
        // library has (a blank from private sync, a private copy) isn't an
        // edit to it.
        let base = match self.store.note(&note.note_id)? {
            Some(local) if note.state.is_empty() => (local.title, local.body),
            _ => (
                doc.title(),
                doc.text().map(|d| d.text(Field::Body)).unwrap_or_default(),
            ),
        };
        let loaded = Arc::new(Mutex::new(Loaded {
            note,
            doc,
            keys,
            lists,
            current,
            base,
            editor_base: None,
            inbox: BTreeMap::new(),
            attached: false,
            cursors: HashMap::new(),
            caret: None,
            in_flight: HashMap::new(),
            pending_snapshot: None,
            needs_refresh: false,
            pending_mentions: Vec::new(),
            ended: false,
            reloading: false,
            canvas_base: None,
            canvas_dirty: false,
            canvas_written_at: 0,
            canvas_full: false,
            pointer: None,
        }));
        Ok(lock(&self.notes)
            .entry(share_id.to_owned())
            .or_insert(loaded)
            .clone())
    }

    /// Forgets a loaded note (it stopped being shared, or its row changed
    /// under it). Anyone still holding it writes nothing more for it.
    pub(crate) fn unload(&self, share_id: &str) {
        if let Some(loaded) = lock(&self.notes).remove(share_id) {
            lock(&loaded).ended = true;
        }
    }

    /// The owner's pinned sign key (this account's own when it is the owner).
    pub(crate) fn owner_sign_pub(&self, owner_id: &str) -> Result<[u8; 32]> {
        if owner_id == self.account.caller.account_id {
            return Ok(self.identity()?.sign_pub());
        }
        let contact = self
            .store
            .contact(owner_id)?
            .ok_or_else(|| CollabError::Refused("The note's owner isn't known here.".into()))?;
        unb64_array::<32>(&contact.sign_pub)
            .ok_or_else(|| CollabError::Crypto("a pinned key isn't well-formed".into()))
    }

    /// Saves a loaded note's row (state, sequence, counters).
    pub(crate) fn persist(&self, loaded: &mut Loaded) -> Result<()> {
        if loaded.ended {
            return Ok(());
        }
        loaded.note.state = loaded.doc.encode_state();
        loaded.note.updated_at = now_ms();
        self.store.put_shared_note(&loaded.note)?;
        Ok(())
    }

    /// The note's `kansoShare`.
    pub(crate) fn mark(note: &SharedNote) -> Value {
        json!({
            "shareId": note.share_id,
            "role": note.role,
            "owner": note.owner_id,
            "ownerName": note.owner_name,
            "epoch": note.epoch,
        })
    }

    /// Makes the library's text the CRDT's, after taking in any local edit
    /// made since `base` (rebased onto what was merged meanwhile). `authors`
    /// are who made the merged changes.
    pub(crate) fn settle(&self, loaded: &mut Loaded, authors: &[String]) -> Result<()> {
        if loaded.ended || loaded.reloading {
            return Ok(());
        }
        if loaded.doc.canvas().is_some() {
            // An open canvas writes the library at its own pace.
            if !authors.is_empty() {
                self.store
                    .add_shared_authors(&loaded.note.note_id, authors)?;
                loaded.canvas_dirty = true;
            }
            let force = !loaded.attached;
            return self.settle_canvas(loaded, force);
        }
        let note_id = loaded.note.note_id.clone();
        if self.store.note_is_locked(&note_id)? || self.store.note_in_locked_book(&note_id)? {
            self.add_problem(&loaded.note.share_id, kanso_store::LOCKED_SHARED);
            return self.persist(loaded);
        }
        for _ in 0..5 {
            let (rev, local) = match self.store.note(&loaded.note.note_id)? {
                Some(note) => (
                    note.rev.clone(),
                    Some((note.title.clone(), note.body.clone())),
                ),
                None => (None, None),
            };
            if let Some(local) = &local {
                self.take_local(loaded, local)?;
            }
            let (title, body) = loaded.text();
            // The library holds a note's text up to its limits; what is
            // past them stays in the CRDT and is never taken as deleted.
            let (title, body) = clamp_text(title, body);
            if (title.len(), body.len())
                != (
                    loaded.text_doc().text(Field::Title).len(),
                    loaded.text_doc().text(Field::Body).len(),
                )
            {
                self.add_problem(
                    &loaded.note.share_id,
                    "The shared note is longer than a note can be here: this library keeps the start of it.",
                );
            }
            let inbox = self
                .store
                .role_book(kanso_core::roles::Role::Inbox, kanso_store::Origin::Collab)?;
            let written = self.store.write_shared_text(&SharedText {
                note_id: &loaded.note.note_id,
                title: &title,
                body: &body,
                mark: Some(Engine::mark(&loaded.note)),
                new_in: &inbox,
                expected_rev: rev.as_deref(),
                authors,
            });
            match written {
                Ok(_) => {
                    loaded.base = (title, body);
                    return self.persist(loaded);
                }
                // Someone wrote the note meanwhile: take that in too.
                Err(StoreError::Conflict) => continue,
                // The library won't take it (a lock, a limit): the merge
                // goes on, and the person hears why the note didn't change.
                Err(StoreError::Invalid(message)) => {
                    self.add_problem(&loaded.note.share_id, &message);
                    return self.persist(loaded);
                }
                Err(err) => return Err(err.into()),
            }
        }
        Err(CollabError::Refused(
            "The note kept changing while it was being merged.".into(),
        ))
    }

    /// Takes in library text that differs from `base`: the difference,
    /// rebased onto the CRDT's text, becomes Yrs operations in the outbox.
    pub(crate) fn take_local(&self, loaded: &mut Loaded, local: &(String, String)) -> Result<bool> {
        let base = loaded.base.clone();
        let changed = self.take_edit(loaded, &base, local)?;
        loaded.base = local.clone();
        Ok(changed)
    }

    /// Takes in the open editor's text: the same, from `editor_base`.
    fn take_editor(&self, loaded: &mut Loaded, editor: &(String, String)) -> Result<bool> {
        let base = loaded
            .editor_base
            .clone()
            .unwrap_or_else(|| loaded.base.clone());
        let changed = self.take_edit(loaded, &base, editor)?;
        loaded.editor_base = Some(editor.clone());
        Ok(changed)
    }

    fn take_edit(
        &self,
        loaded: &mut Loaded,
        base: &(String, String),
        local: &(String, String),
    ) -> Result<bool> {
        if local == base || !loaded.role().may_edit() || loaded.doc.text().is_none() {
            return Ok(false);
        }
        // A note's whole state must fit one snapshot: an edit that would
        // grow it past that is kept as a private note instead.
        let growth =
            local.0.len().saturating_sub(base.0.len()) + local.1.len().saturating_sub(base.1.len());
        if growth > 0
            && loaded.doc.encode_state().len() + growth + crate::doc::OP_COST > STATE_LIMIT
        {
            self.keep_text_copy(
                loaded,
                Some(local),
                "This shared note is at its size limit: the edit is kept in a private note.",
            )?;
            return Ok(false);
        }
        let (title_now, body_now) = loaded.text();
        let title = rebase(&base.0, &local.0, &title_now);
        let body = rebase(&base.1, &local.1, &body_now);
        let mut changed = false;
        for (field, text) in [(Field::Title, &title), (Field::Body, &body)] {
            for update in loaded
                .text_doc()
                .set_text_in_parts(field, text, UPDATE_PART)
            {
                self.store
                    .push_outbox(&loaded.note.share_id, "u", &update)?;
                changed = true;
            }
        }
        if changed {
            self.store.add_shared_authors(
                &loaded.note.note_id,
                std::slice::from_ref(&self.account.caller.username),
            )?;
            self.persist(loaded)?;
        }
        Ok(changed)
    }

    /// Seals `plaintext` as this device's next envelope of `kind` under the
    /// note's current epoch.
    pub(crate) fn seal(
        &self,
        loaded: &mut Loaded,
        kind: Kind,
        base_seq: Option<u64>,
        plaintext: &[u8],
    ) -> Result<(u64, Vec<u8>)> {
        let identity = self.identity()?;
        let key = loaded
            .keys
            .get(&loaded.note.epoch)
            .ok_or_else(|| CollabError::Crypto("this device has no key for the note".into()))?
            .clone();
        loaded.note.counter += 1;
        let counter = loaded.note.counter;
        let header = Header {
            kind,
            epoch: loaded.note.epoch,
            author_account: self.account.caller.account_id.clone(),
            author_device: self.account.caller.device_id.clone(),
            counter,
        };
        let sealed = envelope::seal(
            &identity,
            &key,
            &loaded.note.share_id,
            &header,
            base_seq,
            plaintext,
        );
        self.persist(loaded)?;
        Ok((counter, sealed))
    }

    // -------------------------------------------------------------- problems

    /// Adds a problem for the person, once while it stands.
    pub(crate) fn add_problem(&self, share_id: &str, message: &str) {
        let mut problems = lock(&self.problems);
        if !problems
            .iter()
            .any(|p| p.share_id == share_id && p.message == message)
        {
            problems.push(crate::model::Problem {
                share_id: share_id.to_owned(),
                message: message.to_owned(),
                at: now_ms(),
            });
        }
    }

    /// How much of the note's size limit its state uses.
    pub fn size_use(&self, share_id: &str) -> Option<crate::model::SizeUse> {
        let loaded = self.loaded(share_id).ok()?;
        let used = lock(&loaded).doc.encode_state().len();
        Some(crate::model::SizeUse {
            used,
            limit: STATE_LIMIT,
        })
    }

    /// What the person should hear about, oldest first.
    pub fn problems(&self) -> Vec<crate::model::Problem> {
        lock(&self.problems).clone()
    }

    /// Forgets the problems of one note, or of all.
    pub fn dismiss_problems(&self, share_id: Option<&str>) {
        lock(&self.problems).retain(|p| share_id.is_some_and(|id| id != p.share_id));
    }

    // ---------------------------------------------------------------- editor

    /// The editor opened the note: merges now wait for it.
    pub fn attach(&self, share_id: &str) -> Result<()> {
        let loaded = self.loaded(share_id)?;
        lock(&loaded).attached = true;
        Ok(())
    }

    pub fn detach(&self, share_id: &str) {
        if let Ok(loaded) = self.loaded(share_id) {
            let mut loaded = lock(&loaded);
            loaded.attached = false;
            loaded.caret = None;
            loaded.pointer = None;
            loaded.editor_base = None;
            loaded.canvas_base = None;
            if loaded.doc.canvas().is_some() {
                // The library gets what the canvas became while it was open.
                let _ = self.settle_canvas(&mut loaded, true);
            }
        }
    }

    /// The open note's editor, every 250 ms while the note is live, and
    /// whenever the room says edits are waiting: takes in the editor's text,
    /// merges what arrived, writes the result to the library, and says how
    /// the editor's text must change (splices that never move its caret).
    pub fn sync_editor(&self, share_id: &str, title: &str, body: &str) -> Result<EditorMerge> {
        let loaded = self.loaded(share_id)?;
        let mut loaded = lock(&loaded);
        if loaded.doc.text().is_none() {
            return Err(CollabError::Refused(
                "The note is a canvas: it is merged by drawing, not as text.".into(),
            ));
        }
        if loaded.ended || loaded.reloading {
            // Nothing to merge into yet: the editor keeps what it has.
            return Ok(EditorMerge {
                title: Vec::new(),
                body: Vec::new(),
                title_text: title.to_owned(),
                body_text: body.to_owned(),
            });
        }
        let editor = (title.to_owned(), body.to_owned());
        // What other writers (the API, MCP) put in the library first; the
        // editor's own text then, unless that is what the library has.
        let library = self
            .store
            .note(&loaded.note.note_id)?
            .map(|note| (note.title, note.body));
        let from_library = match &library {
            Some(library) => self.take_local(&mut loaded, library)?,
            None => false,
        };
        if !(from_library && library.as_ref() == Some(&editor)) {
            self.take_editor(&mut loaded, &editor)?;
        }
        let delivered = self.merge_delivered(&mut loaded)?;
        loaded.pending_mentions.extend(delivered.mentions);
        let (title_text, body_text) = loaded.text();
        let merge = EditorMerge {
            title: splices(title, &title_text),
            body: splices(body, &body_text),
            title_text,
            body_text,
        };
        loaded.editor_base = Some((merge.title_text.clone(), merge.body_text.clone()));
        if loaded.base != (merge.title_text.clone(), merge.body_text.clone())
            || !delivered.authors.is_empty()
        {
            self.settle(&mut loaded, &delivered.authors)?;
        }
        Ok(merge)
    }

    /// This device's caret in the open note (byte offsets of the body), sent
    /// as presence by the live room.
    pub fn set_caret(&self, share_id: &str, anchor: usize, head: usize) {
        if let Ok(loaded) = self.loaded(share_id) {
            lock(&loaded).caret = Some((anchor, head));
        }
    }

    /// Other people's cursors in the note, placed in its current text.
    pub fn cursors(&self, share_id: &str) -> Vec<RemoteCursor> {
        let Ok(loaded) = self.loaded(share_id) else {
            return Vec::new();
        };
        let mut loaded = lock(&loaded);
        let now = now_ms();
        loaded
            .cursors
            .retain(|_, (_, at)| now - *at < CURSOR_TTL_MS);
        let Some(doc) = loaded.doc.text() else {
            return Vec::new();
        };
        let mut out: Vec<RemoteCursor> = loaded
            .cursors
            .iter()
            .filter_map(|((account, device), (presence, at))| {
                let anchor = doc.resolve(&presence.cursor)?;
                let head = doc.resolve(&presence.head)?;
                Some(RemoteCursor {
                    account: account.clone(),
                    device: device.clone(),
                    name: presence.name.clone(),
                    anchor,
                    head,
                    colour: colour_of(account),
                    seen_at: *at,
                })
            })
            .collect();
        out.sort_by(|a, b| (&a.name, &a.device).cmp(&(&b.name, &b.device)));
        out
    }

    // -------------------------------------------------------------- comments

    /// The note's comments, folded, with where each sits in the text now;
    /// this device's unsent ones included as pending.
    pub fn comments(&self, share_id: &str) -> Result<Vec<CommentView>> {
        let loaded = self.loaded(share_id)?;
        let loaded = lock(&loaded);
        let Some(doc) = loaded.doc.text() else {
            return Ok(Vec::new());
        };
        let me = self.account.caller.account_id.clone();
        let mut events: Vec<Logged> = self
            .store
            .comment_events(share_id)?
            .into_iter()
            .filter_map(|record| {
                Some(Logged {
                    seq: record.seq,
                    author: record.author,
                    author_name: record.author_name,
                    role: Role::parse(&record.role)?,
                    event: serde_json::from_str(&record.json).ok()?,
                })
            })
            .collect();
        let logged = events.len();
        for (i, outgoing) in self
            .store
            .outbox(share_id)?
            .into_iter()
            .filter(|o| o.kind == "c")
            .enumerate()
        {
            if let Ok(event) = serde_json::from_slice::<CommentEvent>(&outgoing.payload) {
                events.push(Logged {
                    seq: u64::MAX / 2 + i as u64,
                    author: me.clone(),
                    author_name: self.account.caller.username.clone(),
                    role: loaded.role(),
                    event,
                });
            }
        }
        let pending: std::collections::HashSet<u64> =
            events[logged..].iter().map(|e| e.seq).collect();
        let mut out: Vec<CommentView> = comments::fold(&events)
            .into_values()
            .map(|comment| CommentView {
                placement: comments::place(doc, &comment),
                pending: pending.contains(&comment.seq),
                mine: comment.author == me,
                comment,
            })
            .collect();
        out.sort_by_key(|view| match &view.placement {
            crate::model::Placement::Anchored(range) => (range.start, view.comment.created_at),
            crate::model::Placement::Orphaned(at) => (*at, view.comment.created_at),
        });
        Ok(out)
    }

    fn comment_event(&self, share_id: &str, event: CommentEvent) -> Result<String> {
        let loaded = self.loaded(share_id)?;
        let loaded = lock(&loaded);
        if !loaded.role().may_comment() {
            return Err(CollabError::Refused(
                "You can view this note, but not comment on it.".into(),
            ));
        }
        let json = serde_json::to_vec(&event).expect("a comment event serializes");
        if json.len() > envelope::MAX_COMMENT - 512 {
            return Err(CollabError::Refused("The comment is too long.".into()));
        }
        self.store.push_outbox(share_id, "c", &json)?;
        Ok(event.id)
    }

    /// Comments on the byte range `range` of the body (the editor syncs its
    /// text first, so the range is the CRDT's). Mentions of members are
    /// notified when the comment reaches the server.
    pub fn add_comment(
        &self,
        share_id: &str,
        range: std::ops::Range<usize>,
        body: &str,
    ) -> Result<String> {
        let body = body.trim();
        if body.is_empty() || body.chars().count() > comments::MAX_BODY_CHARS {
            return Err(CollabError::Refused(
                "A comment needs some words, and not too many.".into(),
            ));
        }
        let (anchor, mentions) = {
            let loaded = self.loaded(share_id)?;
            let loaded = lock(&loaded);
            let Some(doc) = loaded.doc.text() else {
                return Err(CollabError::Refused(
                    "Comments aren't on canvases yet.".into(),
                ));
            };
            let names: Vec<(String, String)> = loaded
                .current
                .members
                .iter()
                .map(|m| (m.account.clone(), m.username.clone()))
                .collect();
            (
                comments::anchor(doc, range),
                comments::mentions(body, &names, &self.account.caller.account_id),
            )
        };
        self.comment_event(
            share_id,
            CommentEvent {
                id: new_id(),
                op: CommentOp::Add,
                anchor: Some(anchor),
                body: Some(body.to_owned()),
                mentions,
                at: now_ms(),
            },
        )
    }

    pub fn edit_comment(&self, share_id: &str, comment_id: &str, body: &str) -> Result<()> {
        self.comment_event(
            share_id,
            CommentEvent {
                id: comment_id.to_owned(),
                op: CommentOp::Edit,
                anchor: None,
                body: Some(body.trim().to_owned()),
                mentions: Vec::new(),
                at: now_ms(),
            },
        )
        .map(|_| ())
    }

    pub fn delete_comment(&self, share_id: &str, comment_id: &str) -> Result<()> {
        self.simple_comment_op(share_id, comment_id, CommentOp::Delete)
    }

    pub fn resolve_comment(&self, share_id: &str, comment_id: &str, resolved: bool) -> Result<()> {
        let op = if resolved {
            CommentOp::Resolve
        } else {
            CommentOp::Reopen
        };
        self.simple_comment_op(share_id, comment_id, op)
    }

    fn simple_comment_op(&self, share_id: &str, comment_id: &str, op: CommentOp) -> Result<()> {
        self.comment_event(
            share_id,
            CommentEvent {
                id: comment_id.to_owned(),
                op,
                anchor: None,
                body: None,
                mentions: Vec::new(),
                at: now_ms(),
            },
        )
        .map(|_| ())
    }

    /// The anchor of the body's byte range, for a caller that keeps its own.
    pub fn anchor(&self, share_id: &str, range: std::ops::Range<usize>) -> Result<Anchor> {
        let loaded = self.loaded(share_id)?;
        let loaded = lock(&loaded);
        match loaded.doc.text() {
            Some(doc) => Ok(comments::anchor(doc, range)),
            None => Err(CollabError::Refused(
                "Comments aren't on canvases yet.".into(),
            )),
        }
    }

    /// The shared note a library note is, if it is one this device has
    /// loaded or can load.
    pub fn share_of(&self, note_id: &str) -> Option<String> {
        self.store
            .shared_note_of(note_id)
            .ok()
            .flatten()
            .filter(|n| n.status == SharedStatus::Active)
            .map(|n| n.share_id)
    }

    /// The note's role for this account.
    pub fn role(&self, share_id: &str) -> Option<Role> {
        self.loaded(share_id).ok().map(|l| lock(&l).role())
    }
}

/// A note's title and body cut to what the library takes, on character
/// boundaries.
pub(crate) fn clamp_text(title: String, body: String) -> (String, String) {
    fn cut(text: String, chars: usize) -> String {
        match text.char_indices().nth(chars) {
            Some((at, _)) => text[..at].to_owned(),
            None => text,
        }
    }
    (
        cut(title, kanso_core::model::MAX_TITLE_LEN),
        cut(body, kanso_core::model::MAX_BODY_LEN),
    )
}

/// Which of the cursor colours an account gets: the same on every device.
pub fn colour_of(account: &str) -> usize {
    let hash = crate::codec::sha256(&[account.as_bytes()]);
    hash[0] as usize % CURSOR_COLOURS
}

/// Rebases a local edit onto concurrent changes: `local` and `current` both
/// came from `base`; the result is `current` with the local edit made where
/// its text now is. With nothing concurrent (`current == base`) it is
/// `local`; with no local edit it is `current`.
pub fn rebase(base: &str, local: &str, current: &str) -> String {
    if local == base {
        return current.to_owned();
    }
    if current == base {
        return local.to_owned();
    }
    let remote = splices(base, current);
    let mut out = current.to_owned();
    // `splices` lists edits from the end of `base`, so each later one still
    // has offsets in `base`, mapped through the remote edits alone.
    for edit in splices(base, local) {
        let start = remap(edit.at, &remote);
        let end = remap(edit.at + edit.delete, &remote).max(start);
        let end = end.min(out.len());
        let start = start.min(end);
        if out.is_char_boundary(start) && out.is_char_boundary(end) {
            out.replace_range(start..end, &edit.insert);
        }
    }
    out
}

#[cfg(test)]
mod canvas_tests;
#[cfg(test)]
mod tests;
