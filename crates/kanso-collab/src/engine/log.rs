//! A shared note's log (docs/ACCOUNTS.md › Collaboration › Updates,
//! snapshots, comments and presence): pulling and merging what members
//! wrote, in log order; pushing what this device wrote; writing snapshots;
//! and mention notifications.

use kanso_core::now_ms;
use kanso_store::{CommentRecord, StoredNotice};

use super::{COMPACT_AFTER_BYTES, COMPACT_AFTER_UPDATES, Engine, Loaded, lock};
use crate::api::{CollabError, NoticeIn, Result};
use crate::codec::unb64;
use crate::envelope::{self, parse_header};
use crate::keys::{open_notice, seal_notice};
use crate::model::{CommentEvent, CommentOp, Header, Kind, NoteSync, Notice, Role};

/// Entries asked for per page.
const PAGE: usize = 500;
/// How much of a comment a mention's notification carries.
const EXCERPT_CHARS: usize = 140;

/// An envelope, opened and its author checked.
pub(crate) struct Opened {
    pub(crate) header: Header,
    pub(crate) plain: zeroize::Zeroizing<Vec<u8>>,
    pub(crate) author_name: String,
    /// The author's role in the epoch they wrote under.
    pub(crate) role: Role,
}

/// Why an envelope didn't open.
pub(crate) enum Unopened {
    /// Written under an epoch newer than this device's keys.
    NeedKey,
    /// Not from a member, forged, or not for this note: skipped.
    Bad,
}

/// What merging delivered entries did.
#[derive(Default)]
pub(crate) struct Merge {
    /// Who changed the text (usernames).
    pub(crate) authors: Vec<String>,
    pub(crate) text: bool,
    pub(crate) comments: bool,
    /// This account's comments the log took, whose mentions are to be told.
    pub(crate) mentions: Vec<CommentEvent>,
}

impl Engine {
    /// Opens an envelope of a loaded note: the author must be in the
    /// owner-signed member list of the epoch it was written under, with that
    /// list's keys and role.
    pub(crate) fn open_entry(
        &self,
        loaded: &Loaded,
        envelope: &[u8],
        base_seq: Option<u64>,
    ) -> std::result::Result<Opened, Unopened> {
        let (header, _) = parse_header(envelope).ok_or(Unopened::Bad)?;
        let Some(key) = loaded.keys.get(&header.epoch) else {
            return Err(if header.epoch > loaded.note.epoch {
                Unopened::NeedKey
            } else {
                Unopened::Bad
            });
        };
        // Only a member of the epoch's owner-signed list writes under it.
        let member = loaded
            .author(header.epoch, &header.author_account)
            .ok_or(Unopened::Bad)?;
        let sign_pub = crate::codec::unb64_array::<32>(&member.sign_pub).ok_or(Unopened::Bad)?;
        let (author_name, role) = (member.username.clone(), member.role);
        let (opened, plain) =
            envelope::open(envelope, &loaded.note.share_id, base_seq, &sign_pub, key)
                .map_err(|_| Unopened::Bad)?;
        Ok(Opened {
            header: opened,
            plain,
            author_name,
            role,
        })
    }

    /// Merges what the log and the room delivered, in log order: the
    /// snapshot first when it is ahead, then every entry that follows
    /// without a gap. Doesn't write the library (see [`Engine::settle`]).
    pub(crate) fn merge_delivered(&self, loaded: &mut Loaded) -> Result<Merge> {
        let mut merge = Merge::default();
        if let Some((base_seq, snapshot)) = loaded.pending_snapshot.take()
            && (base_seq > loaded.note.applied_seq || loaded.note.applied_seq == 0)
        {
            match self.open_entry(loaded, &snapshot, Some(base_seq)) {
                Ok(opened)
                    if opened.header.kind == Kind::Snapshot
                        && opened.role.may_edit()
                        && !past_its_epoch(loaded, opened.header.epoch, base_seq) =>
                {
                    if apply_safely(&loaded.doc, &opened.plain).unwrap_or(false) {
                        merge.text = true;
                        merge.authors.push(opened.author_name);
                    }
                    loaded.note.applied_seq = loaded.note.applied_seq.max(base_seq);
                    loaded.note.since_snapshot = 0;
                    loaded.note.bytes_since_snapshot = 0;
                }
                Err(Unopened::NeedKey) => {
                    loaded.pending_snapshot = Some((base_seq, snapshot));
                    loaded.needs_refresh = true;
                    return Ok(merge);
                }
                _ => {}
            }
        }
        // Entries at or below what's merged: only comments still count
        // (updates there are in the snapshot).
        let below: Vec<u64> = loaded
            .inbox
            .range(..=loaded.note.applied_seq)
            .map(|(seq, _)| *seq)
            .collect();
        for seq in below {
            let envelope = loaded.inbox.remove(&seq).expect("listed");
            if let Ok(opened) = self.open_entry(loaded, &envelope, None)
                && opened.header.kind == Kind::Comment
                && !past_its_epoch(loaded, opened.header.epoch, seq)
            {
                self.take_comment(loaded, seq, opened, &mut merge)?;
            }
        }
        while let Some(envelope) = loaded.inbox.remove(&(loaded.note.applied_seq + 1)) {
            let seq = loaded.note.applied_seq + 1;
            match self.open_entry(loaded, &envelope, None) {
                // Written under an epoch that had ended by then: dropped.
                Ok(opened) if past_its_epoch(loaded, opened.header.epoch, seq) => {}
                Ok(opened) => match opened.header.kind {
                    Kind::Update if opened.role.may_edit() => {
                        // An update that doesn't decode is skipped like a
                        // forged one: it must never stop the log.
                        let changed = apply_safely(&loaded.doc, &opened.plain).unwrap_or(false);
                        if changed {
                            merge.text = true;
                            if !merge.authors.contains(&opened.author_name) {
                                merge.authors.push(opened.author_name.clone());
                            }
                        }
                        self.echoed(loaded, &opened.header)?;
                    }
                    Kind::Comment => self.take_comment(loaded, seq, opened, &mut merge)?,
                    _ => {}
                },
                Err(Unopened::NeedKey) => {
                    loaded.inbox.insert(seq, envelope);
                    loaded.needs_refresh = true;
                    break;
                }
                Err(Unopened::Bad) => {}
            }
            loaded.note.applied_seq = seq;
            loaded.note.since_snapshot += 1;
            loaded.note.bytes_since_snapshot += envelope.len() as u64;
        }
        Ok(merge)
    }

    /// Keeps a comment event the log delivered.
    fn take_comment(
        &self,
        loaded: &mut Loaded,
        seq: u64,
        opened: Opened,
        merge: &mut Merge,
    ) -> Result<()> {
        let Ok(event) = serde_json::from_slice::<CommentEvent>(&opened.plain) else {
            return Ok(());
        };
        self.store.add_comment_event(
            &loaded.note.share_id,
            &CommentRecord {
                seq,
                author: opened.header.author_account.clone(),
                author_name: opened.author_name.clone(),
                role: opened.role.as_str().into(),
                json: String::from_utf8_lossy(&opened.plain).into_owned(),
            },
        )?;
        merge.comments = true;
        if self.echoed(loaded, &opened.header)?
            && event.op == CommentOp::Add
            && !event.mentions.is_empty()
        {
            merge.mentions.push(event);
        }
        Ok(())
    }

    /// An entry of this device the room took: drops it from the outbox.
    /// Whether it was one sent on the room and waiting.
    fn echoed(&self, loaded: &mut Loaded, header: &Header) -> Result<bool> {
        let caller = &self.account.caller;
        if header.author_account != caller.account_id || header.author_device != caller.device_id {
            return Ok(false);
        }
        match loaded.in_flight.remove(&header.counter) {
            Some(id) => {
                self.store.drop_outbox(id)?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    /// Takes a log page into the note's inbox.
    fn deliver(loaded: &mut Loaded, page: crate::api::LogPage) {
        if let Some(snapshot) = page.snapshot
            && let Some(bytes) = unb64(&snapshot.envelope)
        {
            loaded.pending_snapshot = Some((snapshot.base_seq, bytes));
        }
        for entry in page.entries {
            if let Some(bytes) = unb64(&entry.envelope) {
                loaded.inbox.insert(entry.seq, bytes);
            }
        }
    }

    /// The last sequence this device has without a gap, merged or waiting.
    fn known_seq(loaded: &Loaded) -> u64 {
        let mut seq = loaded
            .pending_snapshot
            .as_ref()
            .map_or(loaded.note.applied_seq, |(base, _)| {
                loaded.note.applied_seq.max(*base)
            });
        while loaded.inbox.contains_key(&(seq + 1)) {
            seq += 1;
        }
        seq
    }

    /// Merges what was delivered and, unless the editor has the note open
    /// (it merges through [`Engine::sync_editor`]), writes the library.
    /// Returns whether text and comments changed.
    pub(crate) fn merge_now(&self, share_id: &str) -> Result<(bool, bool)> {
        let loaded = self.loaded(share_id)?;
        let mentions;
        let changed;
        {
            let mut guard = lock(&loaded);
            if guard.attached {
                let waiting = guard.pending_snapshot.is_some()
                    || guard.inbox.contains_key(&(guard.note.applied_seq + 1))
                    || guard
                        .inbox
                        .range(..=guard.note.applied_seq)
                        .next()
                        .is_some();
                // Comments merge without touching the text.
                let merge = self.merge_comments_only(&mut guard)?;
                mentions = merge.mentions;
                changed = (waiting, merge.comments);
            } else {
                // Local edits made since the last merge go in first.
                self.settle(&mut guard, &[])?;
                let merge = self.merge_delivered(&mut guard)?;
                self.settle(&mut guard, &merge.authors)?;
                mentions = merge.mentions;
                changed = (merge.text, merge.comments);
            }
        }
        self.notify_mentions(share_id, &mentions);
        Ok(changed)
    }

    /// While the editor has the note open: keeps comment entries that are
    /// next in the log, leaving text for [`Engine::sync_editor`].
    fn merge_comments_only(&self, loaded: &mut Loaded) -> Result<Merge> {
        let mut merge = Merge::default();
        let below: Vec<u64> = loaded
            .inbox
            .range(..=loaded.note.applied_seq)
            .map(|(seq, _)| *seq)
            .collect();
        for seq in below {
            if let Some(envelope) = loaded.inbox.get(&seq).cloned()
                && let Ok(opened) = self.open_entry(loaded, &envelope, None)
                && opened.header.kind == Kind::Comment
                && !past_its_epoch(loaded, opened.header.epoch, seq)
            {
                loaded.inbox.remove(&seq);
                self.take_comment(loaded, seq, opened, &mut merge)?;
            }
        }
        Ok(merge)
    }

    /// Brings one shared note up to date over HTTP: takes in local edits,
    /// pulls and merges the log, pushes what this device wrote, and writes a
    /// snapshot when one is due.
    pub fn sync_note(&self, share_id: &str) -> Result<NoteSync> {
        let mut result = NoteSync::default();
        let mut refreshed = false;
        loop {
            match self.sync_once(share_id, &mut result) {
                Err(err) if !refreshed && wants_refresh(&err) => {
                    refreshed = true;
                    self.refresh()?;
                    if self.store.shared_note(share_id)?.is_none() {
                        return Ok(result);
                    }
                }
                other => return other.map(|()| result),
            }
        }
    }

    fn sync_once(&self, share_id: &str, result: &mut NoteSync) -> Result<()> {
        let caller = &self.account.caller;
        self.pull(share_id, result)?;
        let pushed_before = result.pushed;
        // The note started again from the log: pull it all.
        let mut restarted = false;
        // Push.
        loop {
            let next = {
                let loaded = self.loaded(share_id)?;
                let mut guard = lock(&loaded);
                let in_flight: std::collections::HashSet<i64> =
                    guard.in_flight.values().copied().collect();
                let Some(outgoing) = self
                    .store
                    .outbox(share_id)?
                    .into_iter()
                    .find(|o| !in_flight.contains(&o.id))
                else {
                    break;
                };
                let kind = if outgoing.kind == "c" {
                    Kind::Comment
                } else {
                    Kind::Update
                };
                if kind == Kind::Update && !guard.role().may_edit()
                    || kind == Kind::Comment && !guard.role().may_comment()
                {
                    // The role changed: what this device wrote can't go.
                    self.store.drop_outbox(outgoing.id)?;
                    if kind == Kind::Update {
                        self.keep_unsent_copy(
                            &guard,
                            "Your role in this shared note changed before your edits were sent: they are kept in a private note.",
                        )?;
                        self.restart_doc(&mut guard)?;
                        restarted = true;
                    }
                    continue;
                }
                let (_, sealed) = self.seal(&mut guard, kind, None, &outgoing.payload)?;
                (outgoing, kind, sealed)
            };
            let (outgoing, kind, sealed) = next;
            match self.api.append(caller, share_id, &sealed) {
                Ok(_) => {
                    self.store.drop_outbox(outgoing.id)?;
                    result.pushed += 1;
                    if kind == Kind::Comment
                        && let Ok(event) = serde_json::from_slice::<CommentEvent>(&outgoing.payload)
                        && event.op == CommentOp::Add
                    {
                        self.notify_mentions(share_id, &[event]);
                    }
                }
                Err(err) if err.code() == Some("compact_first") => {
                    if !self.compact(share_id)? {
                        return Err(err);
                    }
                    result.compacted = true;
                }
                Err(err) if err.code() == Some("too_large") => {
                    // Can never go: dropped, and the note starts again from
                    // the log so it doesn't hold what others never get.
                    self.store.drop_outbox(outgoing.id)?;
                    if kind == Kind::Update {
                        let loaded = self.loaded(share_id)?;
                        let mut guard = lock(&loaded);
                        self.keep_unsent_copy(
                            &guard,
                            "An edit was too large to share: the note as you had it is kept in a private note.",
                        )?;
                        self.restart_doc(&mut guard)?;
                        restarted = true;
                    }
                }
                Err(err) => return Err(err),
            }
        }
        // What was pushed comes back with its sequence (comments are kept
        // from the log, not from the outbox).
        if result.pushed > pushed_before || restarted {
            self.pull(share_id, result)?;
        }
        // Compact.
        let due = {
            let loaded = self.loaded(share_id)?;
            let guard = lock(&loaded);
            guard.role().may_edit()
                && (guard.note.needs_snapshot
                    || guard.note.since_snapshot >= COMPACT_AFTER_UPDATES
                    || guard.note.bytes_since_snapshot >= COMPACT_AFTER_BYTES)
        };
        if due && self.compact(share_id)? {
            result.compacted = true;
        }
        self.flush_mentions(share_id);
        Ok(())
    }

    /// Pulls the log from where this device is, page by page, merging as it
    /// goes.
    fn pull(&self, share_id: &str, result: &mut NoteSync) -> Result<()> {
        let caller = &self.account.caller;
        loop {
            let since = {
                let loaded = self.loaded(share_id)?;
                let guard = lock(&loaded);
                Engine::known_seq(&guard)
            };
            let page = self.api.log(caller, share_id, since, PAGE)?;
            let more = page.more;
            let head = page.head;
            {
                let loaded = self.loaded(share_id)?;
                let mut guard = lock(&loaded);
                result.pulled += page.entries.len();
                Engine::deliver(&mut guard, page);
            }
            self.merge_now(share_id)?;
            {
                let loaded = self.loaded(share_id)?;
                let mut guard = lock(&loaded);
                if guard.needs_refresh {
                    guard.needs_refresh = false;
                    return Err(CollabError::api(
                        409,
                        "epoch_changed",
                        serde_json::json!({}),
                    ));
                }
                if !more || Engine::known_seq(&guard) >= head {
                    break;
                }
            }
        }
        // A note starting again from the log has all of it now: merge it
        // (open in the editor or not) and write the library.
        let loaded = self.loaded(share_id)?;
        let mut guard = lock(&loaded);
        if guard.reloading {
            let merge = self.merge_delivered(&mut guard)?;
            guard.reloading = false;
            self.settle(&mut guard, &merge.authors)?;
        }
        Ok(())
    }

    /// Writes a snapshot of everything merged, if nothing of this device's is
    /// waiting to go and the log's head is where this device is. Whether it
    /// was written.
    pub(crate) fn compact(&self, share_id: &str) -> Result<bool> {
        let (base_seq, sealed) = {
            let loaded = self.loaded(share_id)?;
            let mut guard = lock(&loaded);
            if !guard.role().may_edit()
                || !self.store.outbox(share_id)?.iter().all(|o| o.kind == "c")
            {
                return Ok(false);
            }
            // A canvas first sends what trims its tombstones, then the
            // snapshot is of the trimmed document.
            if let Some(canvas) = guard.doc.canvas()
                && let Some(trim) = canvas.trim_tombstones()
            {
                self.store.push_outbox(share_id, "u", &trim)?;
                guard.canvas_dirty = true;
                return Ok(false);
            }
            let has_updates = self.store.outbox(share_id)?.iter().any(|o| o.kind == "u");
            if has_updates || !guard.inbox.is_empty() || guard.pending_snapshot.is_some() {
                return Ok(false);
            }
            let base_seq = guard.note.applied_seq;
            let state = guard.doc.encode_state();
            let (_, sealed) = self.seal(&mut guard, Kind::Snapshot, Some(base_seq), &state)?;
            (base_seq, sealed)
        };
        match self
            .api
            .snapshot(&self.account.caller, share_id, base_seq, &sealed)
        {
            Ok(_) => {
                let loaded = self.loaded(share_id)?;
                let mut guard = lock(&loaded);
                guard.note.since_snapshot = 0;
                guard.note.bytes_since_snapshot = 0;
                guard.note.needs_snapshot = false;
                self.persist(&mut guard)?;
                Ok(true)
            }
            // Someone wrote meanwhile: the next sync tries again.
            Err(err) if err.code() == Some("head_moved") => Ok(false),
            Err(err) => Err(err),
        }
    }

    /// Before starting a note again from the log: keeps its text as this
    /// library has it (with the edits that can't go) as a private note
    /// beside it, and tells the person.
    pub(crate) fn keep_unsent_copy(&self, loaded: &Loaded, message: &str) -> Result<()> {
        self.keep_text_copy(loaded, None, message)
    }

    /// Keeps `text` (or, without it, the library's) as a private note beside
    /// the shared one, and tells the person why.
    pub(crate) fn keep_text_copy(
        &self,
        loaded: &Loaded,
        text: Option<&(String, String)>,
        message: &str,
    ) -> Result<()> {
        if let Some(note) = self.store.note(&loaded.note.note_id)? {
            let canvas = loaded.doc.canvas();
            let (title, body) = match (text, canvas) {
                (Some((title, body)), _) => (title.clone(), body.clone()),
                // A canvas as this device has it, what it drew that never
                // went included.
                (None, Some(doc)) => (doc.title(), kanso_core::canvas::to_body(&doc.to_canvas())),
                (None, None) => (note.title, note.body),
            };
            let title = format!("{} (not shared)", title.trim());
            // A canvas is never cut short: one too big for a note isn't kept.
            let (title, body) = if canvas.is_some() {
                (super::clamp_text(title, String::new()).0, body)
            } else {
                super::clamp_text(title, body)
            };
            if canvas.is_some() && kanso_core::canvas::check_body(&body).is_err() {
                self.add_problem(&loaded.note.share_id, message);
                return Ok(());
            }
            self.store.put(
                kanso_core::id::DocType::Note,
                serde_json::json!({
                    "_id": kanso_core::id::new_id(kanso_core::id::DocType::Note),
                    "bookId": note.book_id,
                    "title": title,
                    "body": body,
                    "doctype": if canvas.is_some() { "canvas" } else { note.doctype.as_str() },
                }),
                kanso_store::Put::Upsert,
                kanso_store::Origin::Collab,
            )?;
        }
        self.add_problem(&loaded.note.share_id, message);
        Ok(())
    }

    /// Starts the note's CRDT again from the log, dropping what this device
    /// wrote that the server never took (it can't, or won't).
    pub(crate) fn restart_doc(&self, loaded: &mut Loaded) -> Result<()> {
        for outgoing in self.store.outbox(&loaded.note.share_id)? {
            if outgoing.kind == "u" {
                self.store.drop_outbox(outgoing.id)?;
            }
        }
        loaded.doc = crate::shared_doc::SharedDoc::new(loaded.note.kind);
        loaded.canvas_base = None;
        // The library's canvas is the log's, once it has merged.
        loaded.canvas_dirty = true;
        loaded.canvas_full = false;
        loaded.note.applied_seq = 0;
        loaded.note.since_snapshot = 0;
        loaded.note.bytes_since_snapshot = 0;
        loaded.inbox.clear();
        loaded.pending_snapshot = None;
        loaded.in_flight.clear();
        // The library's text isn't a local edit: the log's replaces it.
        if let Some(note) = self.store.note(&loaded.note.note_id)? {
            loaded.base = (note.title, note.body);
        }
        // An open editor shows the library's text: that is its base until
        // the log's text replaces it.
        loaded.editor_base = loaded.attached.then(|| loaded.base.clone());
        loaded.reloading = true;
        self.persist(loaded)
    }

    /// Syncs every shared note, after bringing the account's shares up to
    /// date. Notes that fail are skipped; the first error is returned after.
    pub fn sync_all(&self) -> Result<Vec<String>> {
        self.identity()?;
        let ended = self.refresh()?;
        let mut first_error = None;
        for note in self.store.shared_notes()? {
            if note.status != kanso_store::SharedStatus::Active {
                continue;
            }
            if let Err(err) = self.sync_note(&note.share_id)
                && first_error.is_none()
                && !err.is_offline()
            {
                first_error = Some(err);
            }
        }
        if let Err(err) = self.fetch_notices()
            && first_error.is_none()
        {
            first_error = Some(err);
        }
        match first_error {
            Some(err) => Err(err),
            None => Ok(ended),
        }
    }

    // -------------------------------------------------------------- notices

    /// Tells the mentions the editor's merges left waiting.
    pub(crate) fn flush_mentions(&self, share_id: &str) {
        let events = match self.loaded(share_id) {
            Ok(loaded) => std::mem::take(&mut lock(&loaded).pending_mentions),
            Err(_) => return,
        };
        self.notify_mentions(share_id, &events);
    }

    /// Tells the members a comment mentions, each sealed to them.
    pub(crate) fn notify_mentions(&self, share_id: &str, events: &[CommentEvent]) {
        if events.is_empty() {
            return;
        }
        let Ok(loaded) = self.loaded(share_id) else {
            return;
        };
        let (members, title) = {
            let guard = lock(&loaded);
            (guard.current.members.clone(), guard.doc.title())
        };
        let me = &self.account.caller.account_id;
        for event in events {
            let body = event.body.clone().unwrap_or_default();
            let notice = Notice {
                comment: event.id.clone(),
                title: title.clone(),
                excerpt: body.chars().take(EXCERPT_CHARS).collect(),
                at: event.at,
            };
            let plain = serde_json::to_vec(&notice).expect("a notice serializes");
            for account in &event.mentions {
                let Some(member) = members
                    .iter()
                    .find(|m| &m.account == account && &m.account != me)
                else {
                    continue;
                };
                let Some(box_pub) = crate::codec::unb64_array::<32>(&member.box_pub) else {
                    continue;
                };
                if let Some(sealed) = seal_notice(account, &box_pub, share_id, me, &plain) {
                    // A notification that doesn't go is only a notification.
                    let _ = self.api.notify(
                        &self.account.caller,
                        &NoticeIn {
                            share_id: share_id.to_owned(),
                            recipient: account.clone(),
                            sealed,
                        },
                    );
                }
            }
        }
    }

    /// Takes the account's mention notifications off the server into the
    /// library. A notification only points at a comment: it is kept only
    /// when the note's log has that comment, signed by its sender and
    /// mentioning this account, and what it shows (the note's title, the
    /// comment's words) comes from the library, not from the notification.
    /// Returns the ones new here.
    pub fn fetch_notices(&self) -> Result<Vec<StoredNotice>> {
        let identity = self.identity()?;
        let me = &self.account.caller.account_id;
        let mut synced = std::collections::HashSet::new();
        let mut out = Vec::new();
        for incoming in self.api.notifications(&self.account.caller)? {
            let opened = open_notice(
                &identity,
                me,
                &incoming.share_id,
                &incoming.from,
                &incoming.sealed,
            )
            .and_then(|plain| serde_json::from_slice::<Notice>(&plain).ok());
            if let Some(notice) = opened
                && let Some(shared) = self.store.shared_note(&incoming.share_id)?
                && shared.status == kanso_store::SharedStatus::Active
            {
                if synced.insert(incoming.share_id.clone()) {
                    let _ = self.sync_note(&incoming.share_id);
                }
                let comment = self
                    .store
                    .comment_events(&incoming.share_id)?
                    .into_iter()
                    .filter(|record| record.author == incoming.from)
                    .filter_map(|record| {
                        Some((
                            record.author_name,
                            serde_json::from_str::<CommentEvent>(&record.json).ok()?,
                        ))
                    })
                    .find(|(_, event)| {
                        event.id == notice.comment
                            && event.op == CommentOp::Add
                            && event.mentions.contains(me)
                    });
                if let Some((from_name, event)) = comment {
                    let title = self
                        .store
                        .note(&shared.note_id)?
                        .map(|note| note.title)
                        .unwrap_or_default();
                    let stored = StoredNotice {
                        id: incoming.id.clone(),
                        share_id: incoming.share_id.clone(),
                        comment_id: event.id,
                        from_id: incoming.from.clone(),
                        from_name,
                        title,
                        excerpt: event
                            .body
                            .unwrap_or_default()
                            .chars()
                            .take(EXCERPT_CHARS)
                            .collect(),
                        at: event.at.min(now_ms()),
                        read: false,
                    };
                    self.store.put_notice(&stored)?;
                    out.push(stored);
                }
            }
            self.api
                .delete_notification(&self.account.caller, &incoming.id)?;
        }
        Ok(out)
    }

    pub fn notices(&self) -> Result<Vec<StoredNotice>> {
        Ok(self.store.notices()?)
    }

    pub fn mark_notices_read(&self, share_id: Option<&str>) -> Result<()> {
        Ok(self.store.mark_notices_read(share_id)?)
    }
}

/// Whether log entry `seq`, written under `epoch`, comes after a later
/// epoch began (its list's `from_seq`): a removed or demoted editor's write
/// relayed late.
pub(crate) fn past_its_epoch(loaded: &Loaded, epoch: u32, seq: u64) -> bool {
    loaded
        .lists
        .range(epoch + 1..)
        .find_map(|(_, list)| list.from_seq)
        .is_some_and(|from| seq > from)
}

/// Applies a Yrs update, `None` when it doesn't decode, or Yrs panics on it.
fn apply_safely(doc: &crate::shared_doc::SharedDoc, update: &[u8]) -> Option<bool> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| doc.apply(update)))
        .ok()?
        .ok()
}

/// An answer that means this device's view of the note's epoch or role is
/// behind.
fn wants_refresh(err: &CollabError) -> bool {
    matches!(err.code(), Some("epoch_changed") | Some("forbidden_role"))
}
