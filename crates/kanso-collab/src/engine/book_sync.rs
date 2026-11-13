//! Pulling and pushing a shared notebook (issue #390 › Shared notebooks:
//! data). The manifest says what the notebook is and which notes are in it;
//! each note is its own log. A sync checks the chain, pulls the manifest and
//! the notes, takes in what was written locally (the library's text against
//! what this device last wrote to it), writes the merged text back, and
//! pushes what this device wrote.
//!
//! **The text invariant** of [`super`] holds per note: the library's text
//! differing from `base` (kept in the log's row) is a local edit; it is
//! rebased onto whatever merged since and becomes Yrs operations, so a local
//! edit never undoes a remote one and the library's text is the CRDT's after
//! every merge. A note's properties merge one key at a time
//! ([`CollabDoc::edit_note_in_parts`]).

use std::collections::{BTreeMap, HashMap};

use kanso_core::{DocType, now_ms};
use kanso_store::{BookStream, Origin, Put, SharedStatus, SharedText, StoreError};
use serde_json::json;

use super::book::{BOOK_FIELDS, Ctx, book_mark, book_note_id, stream_key};
use super::log::{Opened, Unopened};
use super::{Engine, STATE_LIMIT, UPDATE_PART, clamp_text, rebase};
use crate::api::{CollabError, MANIFEST_STREAM, Result};
use crate::book_manifest::ManifestDoc;
use crate::book_model::BookSync;
use crate::codec::unb64;
use crate::doc::{CollabDoc, Field};
use crate::envelope::{self, parse_header};
use crate::model::{Kind, Role};

/// Entries asked for per page.
const PAGE: usize = 500;

/// A log's text as the library keeps it: title and note text.
type Text = (String, String);

fn read_base(row: &BookStream) -> Option<Text> {
    serde_json::from_str::<(String, String)>(&row.base).ok()
}

fn write_base(text: &Text) -> String {
    serde_json::to_string(&[&text.0, &text.1]).expect("serializes")
}

impl Engine {
    /// Opens an envelope of a log: its author must be a member of the
    /// epoch's last list with that list's key and a role that may write.
    fn open_book_entry(
        &self,
        ctx: &Ctx,
        stream: &str,
        envelope_bytes: &[u8],
        base_seq: Option<u64>,
    ) -> std::result::Result<Opened, Unopened> {
        let (header, _) = parse_header(envelope_bytes).ok_or(Unopened::Bad)?;
        let Some(key) = ctx.keys.get(&header.epoch) else {
            // No key for the entry's epoch, older or newer: never "forged".
            // The caller refreshes the chain (its history links give the
            // earlier keys) and tries again, so these entries aren't skipped.
            return Err(Unopened::NeedKey);
        };
        let member = ctx
            .chain
            .epochs
            .get(&header.epoch)
            .and_then(|list| list.member(&header.author_account))
            .ok_or(Unopened::Bad)?;
        let sign_pub = crate::codec::unb64_array::<32>(&member.sign_pub).ok_or(Unopened::Bad)?;
        let (author_name, role) = (member.username.clone(), member.role);
        let (opened, plain) = envelope::open(
            envelope_bytes,
            &ctx.row.share_id,
            base_seq,
            &sign_pub,
            &stream_key(key, &ctx.row.share_id, stream),
        )
        .map_err(|_| Unopened::Bad)?;
        Ok(Opened {
            header: opened,
            plain,
            author_name,
            role,
        })
    }

    /// Pulls a log from where this device is, merging each entry that opens
    /// into its document through `apply` (which says whether it changed
    /// anything). Whether anything changed. An entry under an epoch this
    /// device has no key for is `epoch_changed`: the caller refreshes the
    /// chain and tries again.
    fn pull_into(
        &self,
        ctx: &mut Ctx,
        row: &mut BookStream,
        apply: &mut dyn FnMut(&[u8]) -> bool,
        result: &mut BookSync,
    ) -> Result<bool> {
        let caller = &self.account.caller;
        let mut changed = false;
        loop {
            let page = self.api.book_log(
                caller,
                &ctx.row.share_id,
                &row.stream,
                row.applied_seq,
                PAGE,
            )?;
            if let Some(snapshot) = &page.snapshot {
                let bytes = unb64(&snapshot.envelope)
                    .ok_or_else(|| CollabError::Protocol("a snapshot isn't base64url".into()))?;
                match self.open_book_entry(ctx, &row.stream, &bytes, Some(snapshot.base_seq)) {
                    Ok(opened)
                        if opened.header.kind == Kind::Snapshot
                            && opened.role.may_edit()
                            && !ctx.past_its_epoch(opened.header.epoch, snapshot.base_seq) =>
                    {
                        changed |= apply(&opened.plain);
                    }
                    Err(Unopened::NeedKey) => {
                        return Err(CollabError::api(409, "epoch_changed", json!({})));
                    }
                    _ => {}
                }
                row.applied_seq = row.applied_seq.max(snapshot.base_seq);
                ctx.row.head = ctx.row.head.max(row.applied_seq);
            }
            result.pulled += page.entries.len();
            for entry in &page.entries {
                if entry.seq <= row.applied_seq {
                    continue;
                }
                let bytes = unb64(&entry.envelope)
                    .ok_or_else(|| CollabError::Protocol("an entry isn't base64url".into()))?;
                match self.open_book_entry(ctx, &row.stream, &bytes, None) {
                    Ok(opened)
                        if opened.header.kind == Kind::Update
                            && opened.role.may_edit()
                            && !ctx.past_its_epoch(opened.header.epoch, entry.seq) =>
                    {
                        changed |= apply(&opened.plain);
                    }
                    Err(Unopened::NeedKey) => {
                        return Err(CollabError::api(409, "epoch_changed", json!({})));
                    }
                    // Comments, forged and dropped entries: nothing to merge.
                    _ => {}
                }
                row.applied_seq = entry.seq;
                // The head this device trusts grows only with what it has
                // received, never with the server's say-so.
                ctx.row.head = ctx.row.head.max(entry.seq);
            }
            if !page.more {
                break;
            }
        }
        Ok(changed)
    }

    // ---------------------------------------------------------------- sync

    /// Brings a joined notebook up to date over HTTP: the chain, the
    /// manifest, every note, then what this device wrote. Goes on where it
    /// stopped after a crash or an offline spell.
    pub fn sync_book(&self, share_id: &str) -> Result<BookSync> {
        let mut result = BookSync::default();
        let mut refreshed = false;
        loop {
            match self.sync_book_once(share_id, &mut result) {
                Err(err)
                    if !refreshed
                        && (err.code() == Some("epoch_changed")
                            || err.code() == Some("forbidden_role")) =>
                {
                    refreshed = true;
                    let mut ctx = self.book_ctx(share_id)?;
                    self.refresh_chain(&mut ctx)?;
                }
                other => return other.map(|()| result),
            }
        }
    }

    /// Syncs every notebook joined on this device; one's trouble doesn't
    /// stop the others.
    pub fn sync_books(&self) -> Vec<(String, Result<BookSync>)> {
        self.store
            .shared_books()
            .unwrap_or_default()
            .into_iter()
            .filter(|b| b.status == SharedStatus::Active)
            .map(|b| {
                let result = self.sync_book(&b.share_id);
                (b.share_id, result)
            })
            .collect()
    }

    fn sync_book_once(&self, share_id: &str, result: &mut BookSync) -> Result<()> {
        let mut ctx = self.book_ctx(share_id)?;
        self.refresh_chain(&mut ctx)?;
        let me = self.account.caller.account_id.clone();
        let Some(role) = ctx
            .role(&me)
            .filter(|_| ctx.row.status == SharedStatus::Active)
        else {
            result.removed = true;
            return Ok(());
        };
        if ctx.row.uploading && role == Role::Owner {
            // An interrupted share goes on from where it was (and ends with
            // a sync of its own).
            return self.resume_book_share(share_id, &mut |_| {});
        }
        self.sync_streams(&mut ctx, role, result)
    }

    fn sync_streams(&self, ctx: &mut Ctx, role: Role, result: &mut BookSync) -> Result<()> {
        let share_id = ctx.row.share_id.clone();
        // The manifest first: it says which notes there are.
        let mut manifest_row = self.manifest_row(&share_id)?;
        let manifest = ManifestDoc::load(&manifest_row.state)
            .map_err(|e| CollabError::Crypto(e.to_string()))?;
        let mut apply_manifest = |update: &[u8]| manifest.apply(update).unwrap_or(false);
        self.pull_into(ctx, &mut manifest_row, &mut apply_manifest, result)?;
        if role.may_edit() {
            self.take_local_manifest(ctx, &mut manifest_row, &manifest)?;
        }
        manifest_row.state = manifest.encode_state();
        self.store.put_book_stream(&manifest_row)?;
        self.apply_manifest(ctx, &mut manifest_row, &manifest, role, result)?;
        // Then every note's log.
        // A note listed as deleted is still synced where this device has it:
        // an edit made here brings it back.
        let mut listed: Vec<String> = Vec::new();
        for note in manifest.notes() {
            let here = || -> Result<bool> {
                Ok(self
                    .store
                    .book_stream(&share_id, &note.id)?
                    .and_then(|s| s.note_id)
                    .map(|id| self.store.note(&id))
                    .transpose()?
                    .flatten()
                    .is_some())
            };
            if note.deleted_at.is_none() || here()? {
                listed.push(note.id);
            }
        }
        for stream in &listed {
            self.sync_note_stream(ctx, stream, role, result)?;
        }
        if role.may_edit() {
            self.mark_gone_notes(ctx, &manifest)?;
        }
        // An edit to a note someone deleted brings it back.
        let mut revive = Vec::new();
        for note in manifest.notes() {
            if note.deleted_at.is_some()
                && role.may_edit()
                && self
                    .store
                    .book_outbox(&share_id)?
                    .iter()
                    .any(|o| o.stream == note.id)
            {
                revive.push(note.id);
            }
        }
        for id in revive {
            if let Some(update) = manifest.set_deleted(&id, None) {
                self.store
                    .push_book_outbox(&share_id, MANIFEST_STREAM, &update)?;
            }
        }
        manifest_row.state = manifest.encode_state();
        self.store.put_book_stream(&manifest_row)?;
        let pushed = self.push_book(ctx, result)?;
        // What was pushed comes back with its sequence.
        for stream in &pushed {
            if stream == MANIFEST_STREAM {
                let mut row = self.manifest_row(&share_id)?;
                let doc = ManifestDoc::load(&row.state)
                    .map_err(|e| CollabError::Crypto(e.to_string()))?;
                self.pull_into(
                    ctx,
                    &mut row,
                    &mut |u| doc.apply(u).unwrap_or(false),
                    result,
                )?;
                row.state = doc.encode_state();
                self.store.put_book_stream(&row)?;
            } else {
                self.pull_note_only(ctx, stream, result)?;
            }
        }
        self.save_ctx(ctx)
    }

    fn manifest_row(&self, share_id: &str) -> Result<BookStream> {
        if let Some(row) = self.store.book_stream(share_id, MANIFEST_STREAM)? {
            return Ok(row);
        }
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
        Ok(row)
    }

    // ------------------------------------------------------------ manifest

    /// What this device changed about the notebook itself: notes made in it
    /// (they get a log and are listed), notes taken out of it (marked
    /// deleted), and its fields. Goes to the manifest's outbox.
    fn take_local_manifest(
        &self,
        ctx: &mut Ctx,
        row: &mut BookStream,
        manifest: &ManifestDoc,
    ) -> Result<()> {
        let share_id = ctx.row.share_id.clone();
        let me = self.account.caller.account_id.clone();
        let made = self.adopt_local_notes(ctx)?;
        let mut listed = Vec::new();
        for mut stream in made {
            self.upload_snapshot(ctx, &mut stream)?;
            listed.push((stream.stream.clone(), now_ms()));
        }
        if let Some(update) = manifest.add_notes(&listed, &me) {
            self.store
                .push_book_outbox(&share_id, MANIFEST_STREAM, &update)?;
        }
        // The notebook's own fields: what changed here since this device
        // last wrote them, over what the manifest says now.
        let local = self.local_fields(ctx)?;
        let base: BTreeMap<String, String> = serde_json::from_str(&row.base).unwrap_or_default();
        if !row.base.is_empty() && local != base {
            let mut next = manifest.fields();
            for key in local.keys().chain(base.keys()) {
                if local.get(key) != base.get(key) {
                    match local.get(key) {
                        Some(value) => {
                            next.insert(key.clone(), value.clone());
                        }
                        None => {
                            next.remove(key);
                        }
                    }
                }
            }
            if let Some(update) = manifest.set_fields(&next) {
                self.store
                    .push_book_outbox(&share_id, MANIFEST_STREAM, &update)?;
            }
        } else if row.base.is_empty()
            && ctx.row.role == Role::Owner.as_str()
            && manifest.fields().is_empty()
            && let Some(update) = manifest.set_fields(&local)
        {
            self.store
                .push_book_outbox(&share_id, MANIFEST_STREAM, &update)?;
        }
        Ok(())
    }

    /// Notes that are gone from the notebook here (deleted, or moved out),
    /// with nothing new from anyone, are marked deleted in the manifest.
    fn mark_gone_notes(&self, ctx: &Ctx, manifest: &ManifestDoc) -> Result<()> {
        let share_id = ctx.row.share_id.clone();
        for stream in self.store.book_streams(&share_id)? {
            let Some(note_id) = &stream.note_id else {
                continue;
            };
            let gone = match self.store.note(note_id)? {
                None => !stream.base.is_empty(),
                Some(note) => note.book_id != ctx.row.book_id && !stream.base.is_empty(),
            };
            if gone
                && manifest
                    .note(&stream.stream)
                    .is_some_and(|n| n.deleted_at.is_none())
                && let Some(update) = manifest.set_deleted(&stream.stream, Some(now_ms()))
            {
                self.store
                    .push_book_outbox(&share_id, MANIFEST_STREAM, &update)?;
            }
        }
        Ok(())
    }

    /// Takes the manifest into the library: the notebook (made when this
    /// library has none) with its fields, and a log for every note listed.
    fn apply_manifest(
        &self,
        ctx: &mut Ctx,
        row: &mut BookStream,
        manifest: &ManifestDoc,
        role: Role,
        result: &mut BookSync,
    ) -> Result<()> {
        let share_id = ctx.row.share_id.clone();
        let fields = manifest.fields();
        let exists = self
            .store
            .books()?
            .into_iter()
            .any(|b| b.id == ctx.row.book_id);
        if !exists {
            let name = fields
                .get("name")
                .filter(|n| !n.trim().is_empty())
                .cloned()
                .unwrap_or_else(|| "Shared notebook".to_owned());
            self.store.put(
                DocType::Book,
                json!({"_id": ctx.row.book_id, "name": name}),
                Put::Upsert,
                Origin::Collab,
            )?;
        }
        let local = self.local_fields(ctx)?;
        let base: BTreeMap<String, String> = serde_json::from_str(&row.base).unwrap_or_default();
        let mut patch = serde_json::Map::new();
        for (key, value) in &fields {
            // A field changed here and not yet pushed isn't overwritten.
            let edited_here =
                role.may_edit() && local.get(key) != base.get(key) && !row.base.is_empty();
            if local.get(key) == Some(value) || edited_here {
                continue;
            }
            if key == "name" {
                if !value.trim().is_empty() {
                    patch.insert("name".into(), json!(value));
                }
            } else if BOOK_FIELDS.contains(&key.as_str())
                && let Ok(parsed) = serde_json::from_str(value)
            {
                patch.insert(key.clone(), parsed);
            }
        }
        if !patch.is_empty() {
            patch.insert("_id".into(), json!(ctx.row.book_id));
            self.store
                .put(DocType::Book, patch.into(), Put::Merge, Origin::Collab)?;
        }
        row.base = serde_json::to_string(&self.local_fields(ctx)?).expect("serializes");
        self.store.put_book_stream(row)?;
        // A log for each note listed that this device doesn't have yet.
        let existing: HashMap<String, BookStream> = self
            .store
            .book_streams(&share_id)?
            .into_iter()
            .map(|s| (s.stream.clone(), s))
            .collect();
        let marked: HashMap<String, String> = self
            .store
            .notes_marked_for_book(&share_id)?
            .into_iter()
            .map(|(stream, note, _)| (stream, note))
            .collect();
        for note in manifest.notes() {
            if existing.contains_key(&note.id) || note.deleted_at.is_some() {
                continue;
            }
            let note_id = marked
                .get(&note.id)
                .cloned()
                .unwrap_or_else(|| book_note_id(&note.id));
            self.store.put_book_stream(&BookStream {
                share_id: share_id.clone(),
                stream: note.id.clone(),
                note_id: Some(note_id),
                state: CollabDoc::new().encode_state(),
                applied_seq: 0,
                on_server: true,
                base: String::new(),
            })?;
        }
        let _ = result;
        Ok(())
    }

    // --------------------------------------------------------------- notes

    /// Pulls a note's log and settles the library's text with it. Whether
    /// the note was edited here (its edits are in the outbox).
    fn sync_note_stream(
        &self,
        ctx: &mut Ctx,
        stream: &str,
        role: Role,
        result: &mut BookSync,
    ) -> Result<bool> {
        let Some(mut row) = self.store.book_stream(&ctx.row.share_id, stream)? else {
            return Ok(false);
        };
        let doc = CollabDoc::load(&row.state).map_err(|e| CollabError::Crypto(e.to_string()))?;
        let changed = self.pull_into(
            ctx,
            &mut row,
            &mut |update| doc.apply(update).is_ok_and(|m| !m.is_empty()),
            result,
        )?;
        self.settle_note(ctx, &mut row, &doc, changed, role, result)
    }

    /// Pulls a note's log without taking local edits (after a push).
    fn pull_note_only(&self, ctx: &mut Ctx, stream: &str, result: &mut BookSync) -> Result<()> {
        let Some(mut row) = self.store.book_stream(&ctx.row.share_id, stream)? else {
            return Ok(());
        };
        let doc = CollabDoc::load(&row.state).map_err(|e| CollabError::Crypto(e.to_string()))?;
        let changed = self.pull_into(
            ctx,
            &mut row,
            &mut |update| doc.apply(update).is_ok_and(|m| !m.is_empty()),
            result,
        )?;
        let role = ctx
            .role(&self.account.caller.account_id)
            .unwrap_or(Role::View);
        self.settle_note(ctx, &mut row, &doc, changed, role, result)?;
        Ok(())
    }

    /// Takes the library's edits of a note (against its base) into its
    /// document, and writes the document's text to the library.
    fn settle_note(
        &self,
        ctx: &mut Ctx,
        row: &mut BookStream,
        doc: &CollabDoc,
        changed: bool,
        role: Role,
        result: &mut BookSync,
    ) -> Result<bool> {
        let Some(note_id) = row.note_id.clone() else {
            return Ok(false);
        };
        let share_id = ctx.row.share_id.clone();
        let mut edited = false;
        for _ in 0..5 {
            let note = self.store.note(&note_id)?;
            let rev = note.as_ref().and_then(|n| n.rev.clone());
            let local: Option<Text> = note.as_ref().map(|n| (n.title.clone(), n.body.clone()));
            let base = read_base(row);
            match (&local, &base) {
                // Deleted or moved out of the notebook here, and nothing new
                // from anyone: it stays so. A change from someone brings it
                // back (the manifest lists it deleted until then).
                (None, Some(_)) if !changed => {
                    self.persist_note_row(row, doc)?;
                    return Ok(false);
                }
                (Some(_), Some(_))
                    if note.as_ref().is_some_and(|n| n.book_id != ctx.row.book_id) && !changed =>
                {
                    self.persist_note_row(row, doc)?;
                    return Ok(false);
                }
                _ => {}
            }
            if let (Some(local), Some(base)) = (&local, &base)
                && local != base
            {
                if role.may_edit() {
                    match self.take_note_edit(ctx, row, doc, base, local)? {
                        Some(sent) => edited |= sent,
                        // Held back: the library keeps what was typed and
                        // `base` stays, so the next sync takes the whole
                        // edit once the front matter reads.
                        None => {
                            self.persist_note_row(row, doc)?;
                            return Ok(edited);
                        }
                    }
                }
                row.base = write_base(local);
            }
            let (title, text) = clamp_text(doc.text(Field::Title), doc.note_text());
            let written = self.store.write_shared_text(&SharedText {
                note_id: &note_id,
                title: &title,
                body: &text,
                mark: Some(book_mark(&ctx.row, &row.stream)),
                new_in: &ctx.row.book_id,
                expected_rev: rev.as_deref(),
                authors: &[],
            });
            match written {
                Ok(_) => {
                    if local.as_ref() != Some(&(title.clone(), text.clone())) {
                        result.notes_written += 1;
                    }
                    row.base = write_base(&(title, text));
                    self.persist_note_row(row, doc)?;
                    return Ok(edited);
                }
                Err(StoreError::Conflict) => continue,
                Err(StoreError::Invalid(message)) => {
                    self.add_problem(&share_id, &message);
                    self.persist_note_row(row, doc)?;
                    return Ok(edited);
                }
                Err(err) => return Err(err.into()),
            }
        }
        Err(CollabError::Refused(
            "The note kept changing while it was being merged.".into(),
        ))
    }

    fn persist_note_row(&self, row: &mut BookStream, doc: &CollabDoc) -> Result<()> {
        row.state = doc.encode_state();
        self.store.put_book_stream(row)?;
        Ok(())
    }

    /// A local edit of a note's text, `base` to `local`, as Yrs operations in
    /// the notebook's outbox. Whether it sent anything; `None` when the edit
    /// is held back (front matter half typed): nothing is taken, and the
    /// note's text and `base` stay as they are until it reads again.
    fn take_note_edit(
        &self,
        ctx: &mut Ctx,
        row: &mut BookStream,
        doc: &CollabDoc,
        base: &Text,
        local: &Text,
    ) -> Result<Option<bool>> {
        let growth =
            local.0.len().saturating_sub(base.0.len()) + local.1.len().saturating_sub(base.1.len());
        if growth > 0 && doc.encode_state().len() + growth + crate::doc::OP_COST > STATE_LIMIT {
            let inbox = self
                .store
                .role_book(kanso_core::roles::Role::Inbox, Origin::Collab)?;
            self.store.put(
                DocType::Note,
                json!({"bookId": inbox, "title": local.0, "body": local.1}),
                Put::Upsert,
                Origin::Collab,
            )?;
            self.add_problem(
                &ctx.row.share_id,
                "A note in the shared notebook is at its size limit: the edit is kept in a private note.",
            );
            return Ok(Some(false));
        }
        // The text first: held back, nothing at all is taken (the title
        // with it), and the caller leaves the note as it is.
        let Some(text_updates) = doc.try_edit_note_in_parts(&base.1, &local.1, UPDATE_PART, true)
        else {
            return Ok(None);
        };
        let title = rebase(&base.0, &local.0, &doc.text(Field::Title));
        let mut updates = doc.set_text_in_parts(Field::Title, &title, UPDATE_PART);
        updates.extend(text_updates);
        for update in &updates {
            self.store
                .push_book_outbox(&ctx.row.share_id, &row.stream, update)?;
        }
        Ok(Some(!updates.is_empty()))
    }

    // ---------------------------------------------------------------- push

    /// Pushes what this device wrote, oldest first.
    pub(crate) fn push_book(
        &self,
        ctx: &mut Ctx,
        result: &mut BookSync,
    ) -> Result<std::collections::BTreeSet<String>> {
        let caller = &self.account.caller;
        let share_id = ctx.row.share_id.clone();
        let may_edit = ctx
            .role(&caller.account_id)
            .is_some_and(|role| role.may_edit());
        let mut pushed = std::collections::BTreeSet::new();
        for outgoing in self.store.book_outbox(&share_id)? {
            if !may_edit {
                // The role changed: what this device wrote can't go.
                self.store.drop_book_outbox(outgoing.id)?;
                continue;
            }
            let mut compacted = false;
            loop {
                let sealed =
                    self.seal_book(ctx, &outgoing.stream, Kind::Update, None, &outgoing.payload)?;
                match self
                    .api
                    .book_append(caller, &share_id, &outgoing.stream, &sealed)
                {
                    Ok(_) => {
                        self.store.drop_book_outbox(outgoing.id)?;
                        result.pushed += 1;
                        pushed.insert(outgoing.stream.clone());
                        break;
                    }
                    Err(err) if err.code() == Some("compact_first") && !compacted => {
                        compacted = true;
                        self.compact_book_stream(ctx, &outgoing.stream, result)?;
                    }
                    Err(err) if err.code() == Some("too_large") => {
                        self.store.drop_book_outbox(outgoing.id)?;
                        break;
                    }
                    Err(err) => return Err(err),
                }
            }
        }
        Ok(pushed)
    }

    /// Writes a snapshot of everything merged in a log, so the server can
    /// drop the updates before it. The log's pending updates are in the
    /// document already, so they go with it.
    fn compact_book_stream(
        &self,
        ctx: &mut Ctx,
        stream: &str,
        result: &mut BookSync,
    ) -> Result<()> {
        let caller = &self.account.caller;
        let share_id = ctx.row.share_id.clone();
        let Some(mut row) = self.store.book_stream(&share_id, stream)? else {
            return Ok(());
        };
        let state = if stream == MANIFEST_STREAM {
            let doc =
                ManifestDoc::load(&row.state).map_err(|e| CollabError::Crypto(e.to_string()))?;
            self.pull_into(
                ctx,
                &mut row,
                &mut |u| doc.apply(u).unwrap_or(false),
                result,
            )?;
            doc.encode_state()
        } else {
            let doc =
                CollabDoc::load(&row.state).map_err(|e| CollabError::Crypto(e.to_string()))?;
            self.pull_into(
                ctx,
                &mut row,
                &mut |u| doc.apply(u).is_ok_and(|m| !m.is_empty()),
                result,
            )?;
            doc.encode_state()
        };
        row.state = state;
        self.store.put_book_stream(&row)?;
        let sealed = self.seal_book(
            ctx,
            stream,
            Kind::Snapshot,
            Some(row.applied_seq),
            &row.state,
        )?;
        match self
            .api
            .book_snapshot(caller, &share_id, stream, row.applied_seq, &sealed)
        {
            Ok(_) => {
                for outgoing in self.store.book_outbox(&share_id)? {
                    if outgoing.stream == stream {
                        self.store.drop_book_outbox(outgoing.id)?;
                    }
                }
                Ok(())
            }
            // Someone wrote meanwhile: the next sync tries again.
            Err(err) if err.code() == Some("head_moved") => Ok(()),
            Err(err) => Err(err),
        }
    }
}
