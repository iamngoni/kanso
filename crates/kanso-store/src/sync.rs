//! Sync bookkeeping (docs/ACCOUNTS.md › Sync with encryption › What a Mac
//! remembers per document). Each document keeps one fact per question:
//!
//! - `synced_rev`: the revision this account's server holds, as far as this
//!   Mac knows; the next push's `base_rev`. A document whose current revision
//!   differs has local changes to push.
//! - `synced_gen`: the replay floor, the highest generation this Mac has seen
//!   on this account for the document. Every write that learns a revision
//!   from the server raises it; a server-reported loss and a re-merge clear
//!   `synced_rev` but never lower it. Only binding resets it.
//! - `sent_rev`: the revision in a push whose answer this Mac hasn't
//!   processed. Its echo is our own push, never a conflict. Every write that
//!   sets or clears `synced_rev` clears it.
//!
//! A tombstone with `synced_rev = rev` and `synced_gen = 0` was left by
//! [`Store::reset_sync`] and was never on this account: it is never pushed,
//! never protected by the floor, and never "missing" in a re-merge.

use kanso_core::DocType;
use kanso_core::model::UNSEALED_FIELD;
use kanso_core::roles::Role;
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::doc::Attachment;
use crate::validate::{Refs, normalize};
use crate::write::{Synced, load_row, store_row};
use crate::{Origin, Result, Store, StoreError};

/// A document as exchanged with the sync server.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SyncDoc {
    pub id: String,
    pub rev: String,
    #[serde(default)]
    pub deleted: bool,
    /// The document's fields; empty for deletions.
    #[serde(default)]
    pub fields: Map<String, Value>,
    /// The server revision this change was made on top of; `None` for a
    /// document the server has never seen.
    #[serde(default)]
    pub base_rev: Option<String>,
}

/// A condition every sync write checks inside its own write transaction:
/// every `(meta key, value)` pair in `expect` must hold. The engine passes the
/// library's binding, `[("sync.v2.origin", origin), ("sync.v2.account_id",
/// account_id)]`, so a sync that started under one account can never write
/// after the library was re-bound to another account or server
/// (docs/ACCOUNTS.md › Sync with encryption › The guard; fix 2 / M6; security
/// re-check 3). A mismatch fails with [`StoreError::GuardFailed`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyncGuard<'a> {
    pub expect: &'a [(&'a str, &'a str)],
}

/// What [`Store::apply_remote`] did with a server document
/// (docs/ACCOUNTS.md › Library binding › kanso-store APIs).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApplyOutcome {
    /// The remote version replaced the local one.
    Applied,
    /// The local copy already had this revision, or its pending changes are
    /// based on it.
    Unchanged,
    /// Both sides changed; the remote version won and the local one was kept
    /// as a snapshot. Also a live remote version this Mac never held that
    /// falls at or below the replay floor while the local side is a
    /// tombstone: an edit always beats a deletion (review-verify 2).
    RemoteWon,
    /// Both sides changed; the local version won and will be pushed with
    /// `base_rev` = the remote revision. The remote one was kept as a snapshot.
    ///
    /// `new_rev` is the local version's revision after the conflict. Under M2
    /// (docs/ACCOUNTS.md › Revision generations only grow) it is a new
    /// revision with generation `max(local, remote) + 1` and the same content
    /// hash. (Only the rollback test's stand-in for the old app, behind the
    /// `legacy-v1-rules` feature, keeps the local revision unchanged.)
    LocalWon { new_rev: String },
    /// A remote revision at or below the replay floor (M2, security re-check
    /// 4): its generation is at most the document's `synced_gen`, and it
    /// isn't the revision the Mac holds or last synced. It never reverts the
    /// Mac. Instead the local version is rebased onto it like a local win:
    /// it gets `new_rev` (generation `max(local, remote) + 1`, same content
    /// hash), `synced_rev` becomes the remote revision, and the local version
    /// is pushed again. Never produced by the old app's rules.
    ///
    /// `conflict`: the remote version was live, differed from the local
    /// content, and its content hash was neither the local one nor in the
    /// document's history. It was kept as a snapshot, and the engine flags
    /// the document, so no edit is discarded silently: after a restore,
    /// another Mac's newer offline edit can fall below this Mac's floor. A
    /// replayed version the Mac already holds adds nothing. When the local
    /// side is a tombstone, such a version isn't stale: it wins
    /// ([`ApplyOutcome::RemoteWon`]), so `conflict` is only ever set over
    /// live local content.
    Stale { new_rev: String, conflict: bool },
}

/// One document's sync facts (docs/ACCOUNTS.md › Sync with encryption ›
/// What a Mac remembers per document). Tests and `kanso-sync verify` read
/// them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncState {
    pub id: String,
    pub rev: String,
    pub deleted: bool,
    /// The revision the account's server holds, as far as this Mac knows.
    pub synced_rev: Option<String>,
    /// The replay floor.
    pub synced_gen: u64,
    /// The revision in a push whose answer this Mac hasn't processed.
    pub sent_rev: Option<String>,
}

impl SyncState {
    /// The server lacks this Mac's version.
    pub fn pending(&self) -> bool {
        self.synced_rev.as_deref() != Some(self.rev.as_str())
    }

    /// A tombstone `reset_sync` left behind: never on this account, so never
    /// pushed, never protected by the floor and never missing.
    pub fn never_on_account(&self) -> bool {
        self.deleted && self.synced_gen == 0 && !self.pending()
    }
}

/// SQL for [`SyncState::never_on_account`] over a `docs` row.
const NEVER_ON_ACCOUNT: &str = "(deleted != 0 AND synced_gen = 0 AND synced_rev = rev)";

/// A generation as SQLite stores it.
fn sql_gen(generation: u64) -> i64 {
    i64::try_from(generation).unwrap_or(i64::MAX)
}

/// Records, inside a write, that the account's server holds `rev` of `id`:
/// `synced_rev` becomes it and the floor rises to its generation.
fn learned(tx: &rusqlite::Transaction, id: &str, rev: &str) -> Result<()> {
    let generation = parse_rev(rev)?.generation;
    tx.execute(
        "UPDATE docs SET synced_rev = ?2, synced_gen = max(synced_gen, ?3), sent_rev = NULL WHERE id = ?1",
        params![id, rev, sql_gen(generation)],
    )?;
    Ok(())
}

/// Whether `local` already holds the content of `rev`: it is the current
/// content, or a snapshot in the document's history.
fn holds_content(
    tx: &rusqlite::Transaction,
    id: &str,
    local: &crate::write::Row,
    rev: &kanso_core::Rev,
) -> Result<bool> {
    if parse_rev(&local.rev)?.hash == rev.hash {
        return Ok(true);
    }
    Ok(tx
        .query_row(
            "SELECT 1 FROM revisions WHERE doc_id = ?1 AND substr(rev, instr(rev, '-') + 1) = ?2 LIMIT 1",
            params![id, rev.hash],
            |_| Ok(()),
        )
        .map(|_| true)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(false),
            e => Err(e),
        })?)
}

/// Fails with [`StoreError::GuardFailed`] unless `guard` (when given) holds.
/// Runs inside the caller's write transaction, so it can't race a re-bind.
fn check_guard(tx: &rusqlite::Transaction, guard: Option<&SyncGuard>) -> Result<()> {
    for (key, value) in guard.map_or(&[][..], |g| g.expect) {
        if crate::schema::get_meta(tx, key)?.as_deref() != Some(*value) {
            return Err(StoreError::GuardFailed);
        }
    }
    Ok(())
}

/// A document's fields as private sync sends them: a shared note without
/// its text, which its CRDT carries (docs/ACCOUNTS.md › Collaboration).
fn private_fields(json: &str) -> Map<String, Value> {
    let mut fields = match serde_json::from_str(json) {
        Ok(Value::Object(map)) => map,
        _ => Map::new(),
    };
    crate::collab::private_part(&mut fields);
    fields
}

fn updated_at(fields: &Map<String, Value>) -> i64 {
    fields.get("updatedAt").and_then(Value::as_i64).unwrap_or(0)
}

impl Store {
    /// Local changes the server doesn't have yet, oldest first, each with
    /// `base_rev` = its `synced_rev`. A document re-created over a tombstone
    /// that was never on this account goes with no base.
    pub fn pending_push(&self, limit: usize) -> Result<Vec<SyncDoc>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, rev, deleted, json, CASE WHEN synced_gen = 0 THEN NULL ELSE synced_rev END
                 FROM docs WHERE synced_rev IS NULL OR synced_rev != rev ORDER BY seq LIMIT ?",
            )?;
            let rows = stmt.query_map([limit as i64], |row| {
                let json: String = row.get(3)?;
                Ok(SyncDoc {
                    id: row.get(0)?,
                    rev: row.get(1)?,
                    deleted: row.get(2)?,
                    fields: private_fields(&json),
                    base_rev: row.get(4)?,
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// Every document, tombstones included, in `seq` order, with `base_rev`
    /// set to its `synced_rev`. `kanso-sync verify` and `estimate` read the
    /// whole library with it, and the engine maps wire IDs back to local
    /// ones when it meets an undecryptable server copy (M3).
    pub fn sync_docs(&self) -> Result<Vec<SyncDoc>> {
        self.read(|conn| {
            let mut stmt =
                conn.prepare("SELECT id, rev, deleted, json, synced_rev FROM docs ORDER BY seq")?;
            let rows = stmt.query_map([], |row| {
                let json: String = row.get(3)?;
                Ok(SyncDoc {
                    id: row.get(0)?,
                    rev: row.get(1)?,
                    deleted: row.get(2)?,
                    fields: private_fields(&json),
                    base_rev: row.get(4)?,
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// Every document's sync facts, in `seq` order.
    pub fn sync_states(&self) -> Result<Vec<SyncState>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, rev, deleted, synced_rev, synced_gen, sent_rev FROM docs ORDER BY seq",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(SyncState {
                    id: row.get(0)?,
                    rev: row.get(1)?,
                    deleted: row.get(2)?,
                    synced_rev: row.get(3)?,
                    synced_gen: u64::try_from(row.get::<_, i64>(4)?).unwrap_or(0),
                    sent_rev: row.get(5)?,
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// The number of documents with changes that never reached any server:
    /// the length of [`Store::pending_push`]. `binding::local_library` reports
    /// it as `unsynced`.
    pub fn pending_count(&self) -> Result<usize> {
        self.read(|conn| {
            let count: i64 = conn.query_row(
                "SELECT count(*) FROM docs WHERE synced_rev IS NULL OR synced_rev != rev",
                [],
                |row| row.get(0),
            )?;
            Ok(count as usize)
        })
    }

    /// Records that the server accepted revision `rev` of `id`. Call it right
    /// after the push that sent `rev` succeeded. With a `guard`, nothing is
    /// written unless it holds ([`StoreError::GuardFailed`]).
    ///
    /// If the document was edited locally while the push was in flight, the
    /// edit stays pending, now based on `rev`: local writes only ever build
    /// on the current revision, so the newer one descends from what the
    /// server now has and its next push is accepted.
    pub fn mark_synced(&self, id: &str, rev: &str, guard: Option<&SyncGuard>) -> Result<()> {
        self.write(|tx, _| {
            check_guard(tx, guard)?;
            learned(tx, id, rev)
        })
    }

    /// Records, right before a push request, which revision of each document
    /// it carries (`sent_rev`), in one transaction. If the answer is lost
    /// and the document is edited again, the server's copy coming back
    /// through the feed is recognised as our own push, never a conflict
    /// (docs/ACCOUNTS.md › POST /v2/push › Retrying). With a `guard`,
    /// nothing is written unless it holds.
    pub fn mark_sent(&self, docs: &[(&str, &str)], guard: Option<&SyncGuard>) -> Result<()> {
        self.write(|tx, _| {
            check_guard(tx, guard)?;
            for (id, rev) in docs {
                tx.execute(
                    "UPDATE docs SET sent_rev = ?2 WHERE id = ?1",
                    params![id, rev],
                )?;
            }
            Ok(())
        })
    }

    /// Records that the server says it has no `id`: its `synced_rev` becomes
    /// NULL, so it is pending and its next push carries `base_rev = null`.
    /// The replay floor stays, so a server that claims a loss and then
    /// answers with an old version gets [`ApplyOutcome::Stale`] (security
    /// review, M2 bypass). The engine calls it for a document the server lost
    /// (`Lost`, M1). A repair of an undecryptable server copy (M3) instead
    /// pushes a new generation on the broken revision (docs/ACCOUNTS.md ›
    /// Sync with encryption). With a `guard`, nothing is written unless it
    /// holds.
    pub fn mark_unsynced(&self, id: &str, guard: Option<&SyncGuard>) -> Result<()> {
        self.write(|tx, _| {
            check_guard(tx, guard)?;
            tx.execute(
                "UPDATE docs SET synced_rev = NULL, sent_rev = NULL WHERE id = ?",
                [id],
            )?;
            Ok(())
        })
    }

    /// After a re-merge's full pull (M1, security re-check 4): every document
    /// that has a `synced_rev` (tombstones included) but whose ID isn't in
    /// `present`, the local IDs the server's whole feed carried, gets
    /// `synced_rev = NULL`, so it is pushed again with `base_rev = null`. The
    /// replay floor stays. A tombstone that was never on this account isn't
    /// missing and stays as it is. Returns those IDs, sorted. Nothing is
    /// written unless `guard` holds.
    pub fn mark_missing_unsynced(
        &self,
        present: &std::collections::HashSet<String>,
        guard: Option<&SyncGuard>,
    ) -> Result<Vec<String>> {
        self.write(|tx, _| {
            check_guard(tx, guard)?;
            let synced: Vec<String> = tx
                .prepare(&format!(
                    "SELECT id FROM docs WHERE synced_rev IS NOT NULL AND NOT {NEVER_ON_ACCOUNT} ORDER BY id"
                ))?
                .query_map([], |row| row.get(0))?
                .collect::<rusqlite::Result<_>>()?;
            let missing: Vec<String> = synced
                .into_iter()
                .filter(|id| !present.contains(id))
                .collect();
            for id in &missing {
                tx.execute("UPDATE docs SET synced_rev = NULL, sent_rev = NULL WHERE id = ?", [id])?;
            }
            Ok(missing)
        })
    }

    /// Writes a `meta` entry only while `guard` holds, checked in the same
    /// transaction. The engine saves the cursor with it after each page.
    pub fn set_meta_if(&self, key: &str, value: &str, guard: &SyncGuard) -> Result<()> {
        self.write(|tx, _| {
            check_guard(tx, Some(guard))?;
            crate::schema::set_meta(tx, key, value)
        })
    }

    /// Removes a `meta` entry only while `guard` holds, checked in the same
    /// transaction. The engine clears a quota entry or upload hold with it.
    pub fn remove_meta_if(&self, key: &str, guard: &SyncGuard) -> Result<()> {
        self.write(|tx, _| {
            check_guard(tx, Some(guard))?;
            write_meta(tx, &[(key, None)])
        })
    }

    /// Prepares the library for a different sync binding, in one transaction
    /// (docs/ACCOUNTS.md › Library binding): live documents get
    /// `synced_rev = NULL`; tombstones get `synced_rev = rev`, so they are
    /// never pushed to an account that never had them; every replay floor
    /// goes to 0, so a tombstone this account never had doesn't count as a
    /// version it had (the account's live copy wins over it); then each meta
    /// entry is written (`Some`) or removed (`None`). Used by bind fresh,
    /// merge (add) and unbind: binding to a different account. A re-merge
    /// with the same account (M1) doesn't use it; it keeps `synced_rev` and
    /// the floor (security re-check 4).
    pub fn reset_sync(&self, meta: &[(&str, Option<&str>)]) -> Result<()> {
        self.write(|tx, _| {
            tx.execute(
                "UPDATE docs SET synced_rev = NULL, synced_gen = 0, sent_rev = NULL WHERE deleted = 0",
                [],
            )?;
            tx.execute(
                "UPDATE docs SET synced_rev = rev, synced_gen = 0, sent_rev = NULL WHERE deleted != 0",
                [],
            )?;
            write_meta(tx, meta)
        })
    }

    /// Empties the library in one transaction (docs/ACCOUNTS.md › Library
    /// binding › Replace, fix 3): removes every document, note-tag row, blob,
    /// revision, FTS row, archive row and derived link, heading and task row, then writes (`Some`) or removes
    /// (`None`) the meta entries. Broadcasts a deletion per document
    /// (`Origin::Sync`). Returns the number of documents removed. A crash
    /// leaves either the old library or an empty one bound fresh.
    ///
    /// Change sequences keep growing afterwards: the highest one handed out
    /// so far is kept as a floor, so API change feeds never go back.
    pub fn replace_library(&self, meta: &[(&str, Option<&str>)]) -> Result<usize> {
        self.write(|tx, changes| {
            let floor = crate::schema::last_seq(tx)?;
            let removed: Vec<(String, String)> = tx
                .prepare("SELECT id, rev FROM docs ORDER BY seq")?
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
                .collect::<rusqlite::Result<_>>()?;
            for table in [
                "note_tags",
                "notes_fts",
                "blobs",
                "revisions",
                "archive",
                "docs",
            ]
            .into_iter()
            .chain(crate::derived::TABLES)
            .chain(crate::embed::TABLES)
            .chain(crate::ocr::TABLES)
            {
                tx.execute(&format!("DELETE FROM {table}"), [])?;
            }
            crate::schema::set_meta(tx, crate::schema::SEQ_FLOOR_KEY, &floor.to_string())?;
            write_meta(tx, meta)?;
            changes.extend(removed.iter().map(|(id, rev)| crate::Change {
                id: id.clone(),
                rev: rev.clone(),
                seq: floor,
                deleted: true,
                origin: Origin::Sync,
            }));
            Ok(removed.len())
        })
    }

    /// Writes (`Some`) or removes (`None`) several `meta` entries in one
    /// transaction (device identity, docs/ACCOUNTS.md › Device identity).
    pub fn set_metas(&self, meta: &[(&str, Option<&str>)]) -> Result<()> {
        self.write(|tx, _| write_meta(tx, meta))
    }

    /// Gives `id` a new revision one generation higher with the same content
    /// hash, keeping its `synced_rev`, so it is pushed again on top of the
    /// server's current revision. The engine's automatic repair of an
    /// undecryptable server copy (M3, security re-check 10) and `kanso-sync
    /// verify --repair` use it. Returns the new revision, or `None` when
    /// there is no such document. Nothing is written unless `guard` holds.
    pub fn bump_revision(&self, id: &str, guard: Option<&SyncGuard>) -> Result<Option<String>> {
        self.write(|tx, changes| {
            check_guard(tx, guard)?;
            let Some(local) = load_row(tx, id)? else {
                return Ok(None);
            };
            let rev = parse_rev(&local.rev)?;
            let new_rev = rebased(&rev, rev.generation);
            set_rev(
                tx,
                id,
                &local,
                &new_rev,
                local.synced_rev.as_deref(),
                changes,
            )?;
            Ok(Some(new_rev))
        })
    }

    /// Live file documents whose bytes this Mac lacks (no blob matches their
    /// `md5digest`), as `(id, rev, md5digest)` in ID order, starting after
    /// `after`. Documents without an `md5digest` can't be checked and are
    /// left out. The engine fetches a few of them again at each sync
    /// (docs/ACCOUNTS.md › Sync with encryption › Files).
    pub fn files_without_bytes(
        &self,
        after: &str,
        limit: usize,
    ) -> Result<Vec<(String, String, String)>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT d.id, d.rev, lower(json_extract(d.json, '$.md5digest')) FROM docs d
                 WHERE d.type = 'file' AND d.deleted = 0 AND d.id > ?1
                   AND json_type(d.json, '$.md5digest') = 'text'
                   AND NOT EXISTS (SELECT 1 FROM blobs b WHERE b.file_id = d.id
                                   AND b.md5 = lower(json_extract(d.json, '$.md5digest')))
                 ORDER BY d.id LIMIT ?2",
            )?;
            let rows = stmt.query_map(params![after, limit as i64], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// Stores bytes fetched again for file `id` (docs/ACCOUNTS.md › Sync
    /// with encryption › Files), only if `id` is still live at `rev` and the
    /// bytes match its `md5digest`, checked in the same transaction as the
    /// `guard`. Returns whether they were stored. The document's revision
    /// doesn't change, so nothing is broadcast.
    pub fn fill_blob(
        &self,
        id: &str,
        rev: &str,
        blob: Attachment,
        guard: Option<&SyncGuard>,
    ) -> Result<bool> {
        self.write(|tx, _| {
            check_guard(tx, guard)?;
            let Some(local) = load_row(tx, id)? else {
                return Ok(false);
            };
            let md5 = local
                .fields
                .get("md5digest")
                .and_then(Value::as_str)
                .map(str::to_ascii_lowercase);
            if local.deleted
                || local.doc_type != DocType::File
                || local.rev != rev
                || md5.as_deref() != Some(blob.md5_hex().as_str())
            {
                return Ok(false);
            }
            tx.execute(
                "INSERT INTO blobs (file_id, md5, data) VALUES (?1, ?2, ?3)
                 ON CONFLICT (file_id) DO UPDATE SET md5 = excluded.md5, data = excluded.data",
                params![id, blob.md5_hex(), blob.data],
            )?;
            Ok(true)
        })
    }

    /// The MD5 (hex) of the bytes the store holds for attachment `id`, if any.
    pub fn blob_md5(&self, id: &str) -> Result<Option<String>> {
        self.read(|conn| {
            Ok(conn
                .query_row("SELECT md5 FROM blobs WHERE file_id = ?", [id], |row| {
                    row.get::<_, String>(0)
                })
                .map(Some)
                .or_else(|e| match e {
                    rusqlite::Error::QueryReturnedNoRows => Ok(None),
                    e => Err(e),
                })?)
        })
    }

    /// Whether the store has the bytes of attachment `id` with this MD5 (hex).
    pub fn has_blob(&self, id: &str, md5_hex: &str) -> Result<bool> {
        self.read(|conn| {
            Ok(conn
                .query_row(
                    "SELECT 1 FROM blobs WHERE file_id = ? AND md5 = ?",
                    [id, md5_hex],
                    |_| Ok(()),
                )
                .map(|_| true)
                .or_else(|e| match e {
                    rusqlite::Error::QueryReturnedNoRows => Ok(false),
                    e => Err(e),
                })?)
        })
    }

    /// Applies a document received from the server. When both sides changed
    /// it, the newer `updatedAt` wins (an edit always beats a deletion) and the
    /// losing version is kept in the document's revision history. With a
    /// `guard`, nothing is written unless it holds ([`StoreError::GuardFailed`]).
    ///
    /// The M2 rules hold (docs/ACCOUNTS.md › Revision generations only grow):
    /// a local win gets a new revision, generation `max(local, remote) + 1`
    /// with the same content hash, pushed on top of the remote one; and a
    /// remote revision at or below the replay floor `synced_gen` (and not
    /// `synced_rev` itself) never reverts the Mac: the local version is
    /// rebased onto it the same way ([`ApplyOutcome::Stale`]). The one
    /// exception is a live version this Mac never held over a local
    /// tombstone: an edit beats a deletion, so it wins
    /// ([`ApplyOutcome::RemoteWon`]). Every outcome that takes a revision
    /// from the server raises the floor.
    pub fn apply_remote(
        &self,
        remote: &SyncDoc,
        attachment: Option<Attachment>,
        guard: Option<&SyncGuard>,
    ) -> Result<ApplyOutcome> {
        self.apply(remote, attachment, guard, Rules::V2)
    }

    /// [`Store::apply_remote`] under the old app's rules (before accounts):
    /// a local win keeps its revision and nothing is ever `Stale`. Test
    /// support only, behind the `legacy-v1-rules` feature, which no shipped
    /// build enables: the rollback test (docs/ACCOUNTS.md › Rollback)
    /// replays the old app's sync with it, so a rolled-back library is
    /// checked against the rules the old app really runs.
    #[cfg(feature = "legacy-v1-rules")]
    pub fn apply_remote_legacy(
        &self,
        remote: &SyncDoc,
        attachment: Option<Attachment>,
    ) -> Result<ApplyOutcome> {
        self.apply(remote, attachment, None, Rules::Legacy)
    }

    fn apply(
        &self,
        remote: &SyncDoc,
        attachment: Option<Attachment>,
        guard: Option<&SyncGuard>,
        rules: Rules,
    ) -> Result<ApplyOutcome> {
        let doc_type = DocType::of_id(&remote.id)
            .ok_or_else(|| StoreError::Invalid(format!("Invalid document ID '{}'.", remote.id)))?;
        let remote_rev = parse_rev(&remote.rev)?;
        self.write(|tx, changes| {
            check_guard(tx, guard)?;
            let local = load_row(tx, &remote.id)?;
            let mut fields = if remote.deleted {
                Map::new()
            } else {
                let created = remote.fields.get("createdAt").and_then(Value::as_i64).unwrap_or(0);
                let mut incoming = remote.fields.clone();
                // A shared note's text comes from its CRDT, never from private sync.
                crate::collab::keep_shared_text(
                    &mut incoming,
                    local.as_ref().filter(|l| !l.deleted).map(|l| &l.fields),
                );
                normalize(doc_type, &remote.id, incoming, None, created, Refs::Skip)?
            };
            if doc_type == DocType::File
                && fields.contains_key(kanso_core::model::FILE_LOCK_FIELD)
            {
                fields.remove(kanso_core::model::OCR_FIELD);
            }
            let requested_unseal = fields
                .remove(UNSEALED_FIELD)
                .and_then(|marker| marker.get("seal").and_then(Value::as_str).map(str::to_owned));
            let store_blob = |tx: &rusqlite::Transaction| -> Result<()> {
                if let Some(blob) = &attachment {
                    tx.execute(
                        "INSERT INTO blobs (file_id, md5, data) VALUES (?1, ?2, ?3)
                         ON CONFLICT (file_id) DO UPDATE SET md5 = excluded.md5, data = excluded.data",
                        params![remote.id, blob.md5_hex(), blob.data],
                    )?;
                }
                Ok(())
            };
            let Some(local) = local else {
                store_blob(tx)?;
                store_row(tx, &remote.id, doc_type, &remote.rev, remote.deleted, &fields, None, Synced::Set(&remote.rev), Origin::Sync, changes)?;
                learned(tx, &remote.id, &remote.rev)?;
                return Ok(ApplyOutcome::Applied);
            };
            let clear_stale_blob = |tx: &rusqlite::Transaction| -> Result<()> {
                // Remove an old blob only when this file document is accepted
                // and its bytes no longer match the document. A replay that
                // loses below must leave the current bytes intact; a plain
                // unseal that arrives before its bytes must not keep the old
                // sealed container behind a now-plain document.
                if !remote.deleted
                    && let Some(expected) = fields.get("md5digest").and_then(Value::as_str)
                {
                    let stored = tx
                        .query_row(
                            "SELECT data FROM blobs WHERE file_id = ?1",
                            [&remote.id],
                            |row| row.get::<_, Vec<u8>>(0),
                        )
                        .optional()?;
                    if stored.is_some_and(|data| {
                        !Attachment::new(data)
                            .md5_hex()
                            .eq_ignore_ascii_case(expected)
                    }) {
                        tx.execute("DELETE FROM blobs WHERE file_id = ?1", [&remote.id])?;
                    }
                }
                Ok(())
            };
            // An old plaintext document must never remove a local seal. A
            // matching `kansoUnsealed.seal` is the explicit transition
            // token; otherwise rebase the local sealed version on the
            // incoming revision and leave its blob untouched.
            if !remote.deleted
                && doc_type == DocType::File
                && !local.deleted
                && local.fields.contains_key(kanso_core::model::FILE_LOCK_FIELD)
                && !fields.contains_key(kanso_core::model::FILE_LOCK_FIELD)
            {
                let local_seal = local.fields
                    .get(kanso_core::model::FILE_LOCK_FIELD)
                    .and_then(|lock| lock.get("seal"))
                    .and_then(Value::as_str);
                if local_seal != requested_unseal.as_deref() {
                    let local_rev = parse_rev(&local.rev)?;
                    let new_rev = rebased(
                        &local_rev,
                        local_rev.generation.max(remote_rev.generation),
                    );
                    set_rev(tx, &remote.id, &local, &new_rev, Some(&remote.rev), changes)?;
                    learned(tx, &remote.id, &remote.rev)?;
                    return Ok(ApplyOutcome::LocalWon { new_rev });
                }
            }
            if local.rev == remote.rev {
                // Same revision, same content: bytes that arrive with it are
                // this document's (e.g. fetched after an earlier miss).
                if !remote.deleted {
                    clear_stale_blob(tx)?;
                    store_blob(tx)?;
                }
                learned(tx, &remote.id, &remote.rev)?;
                return Ok(ApplyOutcome::Unchanged);
            }
            // The server's version is the one the local edits are based on
            // (typically our own push coming back through the change feed):
            // nothing new on the remote side, so it is not a conflict.
            if local.synced_rev.as_deref() == Some(remote.rev.as_str()) {
                return Ok(ApplyOutcome::Unchanged);
            }
            // Our own push, whose answer was lost, coming back after a
            // newer local edit: the server has it, and the edit stays
            // pending on top of it. Never a conflict, whatever `updatedAt`
            // says (an API merge keeps it).
            if local.sent_rev.as_deref() == Some(remote.rev.as_str()) {
                learned(tx, &remote.id, &remote.rev)?;
                return Ok(ApplyOutcome::Unchanged);
            }
            let local_rev = parse_rev(&local.rev)?;
            // M2: a revision at or below the floor (the highest generation
            // this Mac has seen on this account) is a replay or a rewound
            // server. Never revert; rebase and push. The floor survives a
            // reported loss and a re-merge, so neither can open the way for
            // an old version. A tombstone `reset_sync` left has floor 0: the
            // account's versions aren't replays of it.
            if !rules.legacy() && remote_rev.generation <= local.synced_gen {
                // A live version this Mac has never held is kept and
                // flagged: it may be another Mac's edit made after a restore
                // (generations from different Macs stop comparing then).
                let differs = local.deleted || local.fields != fields;
                let conflict = !remote.deleted
                    && differs
                    && !holds_content(tx, &remote.id, &local, &remote_rev)?;
                // An edit always beats a deletion, below the floor too
                // (review-verify 2): a live version this Mac never held replaces
                // a local tombstone, bytes and all, and is flagged. Pushing
                // the tombstone would delete another Mac's edit, and the
                // file's bytes, everywhere. A replay of content this Mac
                // held stays stale below.
                if conflict && local.deleted {
                    clear_stale_blob(tx)?;
                    store_blob(tx)?;
                    store_row(tx, &remote.id, doc_type, &remote.rev, false, &fields, Some(&local), Synced::Set(&remote.rev), Origin::Sync, changes)?;
                    learned(tx, &remote.id, &remote.rev)?;
                    return Ok(ApplyOutcome::RemoteWon);
                }
                if conflict {
                    keep_remote(tx, &remote.id, &remote.rev, &fields)?;
                }
                let new_rev = rebased(&local_rev, local_rev.generation.max(remote_rev.generation));
                set_rev(tx, &remote.id, &local, &new_rev, Some(&remote.rev), changes)?;
                learned(tx, &remote.id, &remote.rev)?;
                return Ok(ApplyOutcome::Stale { new_rev, conflict });
            }
            // Two first versions of the same day's note, made apart (two
            // devices offline): merged rather than one winning.
            if !rules.legacy() && is_separate_daily_creation(&remote.id, &local, remote) {
                return merge_daily(tx, remote, &remote_rev, &local, &local_rev, &fields, changes);
            }
            // Two Journals (or Inboxes, or Templates) made apart: the same
            // notebook, so the account's copy is taken without a conflict,
            // unless this one already names a daily template it lacks.
            if !rules.legacy() && is_separate_role_book_creation(&remote.id, &local, remote, &fields) {
                store_row(tx, &remote.id, doc_type, &remote.rev, false, &fields, Some(&local), Synced::Set(&remote.rev), Origin::Sync, changes)?;
                learned(tx, &remote.id, &remote.rev)?;
                return Ok(ApplyOutcome::Applied);
            }
            let local_dirty = local.synced_rev.as_deref() != Some(local.rev.as_str());
            let remote_wins = !local_dirty
                || match (local.deleted, remote.deleted) {
                    (true, false) => true,
                    (false, true) => false,
                    (true, true) => true,
                    (false, false) => updated_at(&remote.fields) >= updated_at(&local.fields),
                };
            if remote_wins {
                clear_stale_blob(tx)?;
                store_blob(tx)?;
                store_row(tx, &remote.id, doc_type, &remote.rev, remote.deleted, &fields, Some(&local), Synced::Set(&remote.rev), Origin::Sync, changes)?;
                learned(tx, &remote.id, &remote.rev)?;
                return Ok(if local_dirty { ApplyOutcome::RemoteWon } else { ApplyOutcome::Applied });
            }
            // Keep the local version; base it on the server's revision so the
            // next push is accepted, and keep the remote version as history.
            if !remote.deleted {
                keep_remote(tx, &remote.id, &remote.rev, &fields)?;
            }
            if rules.legacy() {
                learned(tx, &remote.id, &remote.rev)?;
                return Ok(ApplyOutcome::LocalWon { new_rev: local.rev });
            }
            let new_rev = rebased(&local_rev, local_rev.generation.max(remote_rev.generation));
            set_rev(tx, &remote.id, &local, &new_rev, Some(&remote.rev), changes)?;
            learned(tx, &remote.id, &remote.rev)?;
            Ok(ApplyOutcome::LocalWon { new_rev })
        })
    }
}

/// Whether `local` and `remote` are a role notebook (`book:journal`, …)
/// made apart: both live, the local one never reached this account, and
/// taking the remote one loses nothing the local one was given.
fn is_separate_role_book_creation(
    id: &str,
    local: &crate::write::Row,
    remote: &SyncDoc,
    remote_fields: &Map<String, Value>,
) -> bool {
    // What only the local one was given: a daily template, or a link it
    // was published at.
    let given = [
        crate::roles::DAILY_TEMPLATE_FIELD,
        kanso_core::model::PUBLISH_FIELD,
    ]
    .iter()
    .any(|field| {
        local.fields.get(*field).is_some_and(|v| !v.is_null())
            && remote_fields.get(*field) != local.fields.get(*field)
    });
    [Role::Journal, Role::Inbox, Role::Templates]
        .iter()
        .any(|role| role.book_id() == id)
        && local.doc_type == DocType::Book
        && !local.deleted
        && !remote.deleted
        && local.synced_rev.is_none()
        && local.synced_gen == 0
        && !given
}

/// Whether `local` and `remote` are two versions of a daily note made apart:
/// both live, and the local one never reached this account.
fn is_separate_daily_creation(id: &str, local: &crate::write::Row, remote: &SyncDoc) -> bool {
    id.starts_with(kanso_core::roles::DAILY_PREFIX)
        && local.doc_type == DocType::Note
        && !local.deleted
        && !remote.deleted
        && local.synced_rev.is_none()
        && local.synced_gen == 0
}

/// Merges two separately made versions of a daily note
/// (`kanso_core::roles::merge_daily_bodies`, the older one's lines first,
/// tags joined). When the remote version already has everything, it is
/// applied like any newer version. Otherwise the merge becomes the local
/// version, one generation above both, based on the remote one so it is
/// pushed, and the remote version is kept in history: an
/// [`ApplyOutcome::LocalWon`].
fn merge_daily(
    tx: &rusqlite::Transaction,
    remote: &SyncDoc,
    remote_rev: &kanso_core::Rev,
    local: &crate::write::Row,
    local_rev: &kanso_core::Rev,
    remote_fields: &Map<String, Value>,
    changes: &mut Vec<crate::Change>,
) -> Result<ApplyOutcome> {
    let body = |fields: &Map<String, Value>| {
        fields
            .get("body")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned()
    };
    let created =
        |fields: &Map<String, Value>| fields.get("createdAt").and_then(Value::as_i64).unwrap_or(0);
    let tags = |fields: &Map<String, Value>| -> Vec<String> {
        fields
            .get("tags")
            .and_then(Value::as_array)
            .map(|tags| {
                tags.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    };
    let (remote_body, local_body) = (body(remote_fields), body(&local.fields));
    let merged_body = if created(&local.fields) < created(remote_fields) {
        kanso_core::roles::merge_daily_bodies(&local_body, &remote_body)
    } else {
        kanso_core::roles::merge_daily_bodies(&remote_body, &local_body)
    };
    let mut merged_tags = tags(remote_fields);
    for tag in tags(&local.fields) {
        if !merged_tags.contains(&tag) {
            merged_tags.push(tag);
        }
    }
    let doc_type = DocType::Note;
    if merged_body == remote_body && merged_tags == tags(remote_fields) {
        store_row(
            tx,
            &remote.id,
            doc_type,
            &remote.rev,
            false,
            remote_fields,
            Some(local),
            Synced::Set(&remote.rev),
            Origin::Sync,
            changes,
        )?;
        learned(tx, &remote.id, &remote.rev)?;
        return Ok(ApplyOutcome::Applied);
    }
    let mut fields = remote_fields.clone();
    fields.insert("body".into(), merged_body.into());
    fields.insert("tags".into(), merged_tags.into());
    let updated = updated_at(&local.fields).max(updated_at(remote_fields));
    fields.insert("updatedAt".into(), updated.into());
    fields.insert("timestamp".into(), updated.into());
    let first_created = created(&local.fields).min(created(remote_fields));
    let fields = normalize(
        doc_type,
        &remote.id,
        fields,
        None,
        first_created,
        Refs::Skip,
    )?;
    let above = kanso_core::Rev {
        generation: local_rev.generation.max(remote_rev.generation),
        hash: String::new(),
    };
    let new_rev = kanso_core::Rev::next(Some(&above), &Value::Object(fields.clone())).to_string();
    keep_remote(tx, &remote.id, &remote.rev, remote_fields)?;
    store_row(
        tx,
        &remote.id,
        doc_type,
        &new_rev,
        false,
        &fields,
        Some(local),
        Synced::Set(&remote.rev),
        Origin::Sync,
        changes,
    )?;
    learned(tx, &remote.id, &remote.rev)?;
    Ok(ApplyOutcome::LocalWon { new_rev })
}

/// Which conflict rules [`Store::apply_remote`] follows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Rules {
    /// Accounts (M2): local wins and replays get a higher generation.
    V2,
    /// The old app's rules, for the rollback test only.
    #[cfg(feature = "legacy-v1-rules")]
    Legacy,
}

impl Rules {
    /// Whether these are the old app's rules (never, without the feature).
    fn legacy(self) -> bool {
        #[cfg(feature = "legacy-v1-rules")]
        return self == Rules::Legacy;
        #[cfg(not(feature = "legacy-v1-rules"))]
        return false;
    }
}

fn parse_rev(rev: &str) -> Result<kanso_core::Rev> {
    rev.parse::<kanso_core::Rev>()
        .map_err(|e| StoreError::Invalid(e.to_string()))
}

/// `rev`'s content hash under generation `above + 1`.
fn rebased(rev: &kanso_core::Rev, above: u64) -> String {
    kanso_core::Rev {
        generation: above.saturating_add(1),
        hash: rev.hash.clone(),
    }
    .to_string()
}

/// Keeps a remote version that didn't win as a snapshot in history.
fn keep_remote(
    tx: &rusqlite::Transaction,
    id: &str,
    rev: &str,
    fields: &Map<String, Value>,
) -> Result<()> {
    tx.execute(
        "INSERT INTO revisions (doc_id, rev, json, saved_at, origin) VALUES (?1, ?2, ?3, ?4, 'sync')",
        params![
            id,
            rev,
            serde_json::to_string(fields)?,
            crate::history::version_time(tx, id)?
        ],
    )?;
    Ok(())
}

/// Gives a row a new revision (same content) and `synced_rev`, under a new
/// change sequence, and reports the change.
fn set_rev(
    tx: &rusqlite::Transaction,
    id: &str,
    local: &crate::write::Row,
    new_rev: &str,
    synced_rev: Option<&str>,
    changes: &mut Vec<crate::Change>,
) -> Result<()> {
    let seq = crate::schema::next_seq(tx)?;
    tx.execute(
        "UPDATE docs SET rev = ?2, synced_rev = ?3, seq = ?4 WHERE id = ?1",
        params![id, new_rev, synced_rev, seq],
    )?;
    changes.push(crate::Change {
        id: id.to_owned(),
        rev: new_rev.to_owned(),
        seq,
        deleted: local.deleted,
        origin: Origin::Sync,
    });
    Ok(())
}

/// Writes (`Some`) or removes (`None`) meta entries inside a transaction.
fn write_meta(tx: &rusqlite::Transaction, meta: &[(&str, Option<&str>)]) -> Result<()> {
    for (key, value) in meta {
        match value {
            Some(value) => crate::schema::set_meta(tx, key, value)?,
            None => {
                tx.execute("DELETE FROM meta WHERE key = ?", [key])?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use kanso_core::DocType;
    use kanso_core::model::{FILE_LOCK_FIELD, FileLock};
    use serde_json::json;

    use super::*;
    use crate::Put;

    fn store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        (dir, store)
    }

    fn pending(store: &Store, id: &str) -> Option<SyncDoc> {
        store
            .pending_push(usize::MAX)
            .unwrap()
            .into_iter()
            .find(|d| d.id == id)
    }

    fn create_note(store: &Store, id: &str) -> crate::WriteResult {
        store
            .put(
                DocType::Note,
                json!({"_id": id, "bookId": "trash", "body": "v1", "updatedAt": 100}),
                Put::Upsert,
                Origin::Api,
            )
            .unwrap()
    }

    fn merge_body(store: &Store, id: &str, body: &str) -> crate::WriteResult {
        store
            .put(
                DocType::Note,
                json!({"_id": id, "body": body}),
                Put::Merge,
                Origin::Api,
            )
            .unwrap()
    }

    fn create_file(store: &Store, id: &str, bytes: &[u8]) -> crate::WriteResult {
        use base64::Engine as _;
        store
            .put(
                DocType::File,
                json!({"_id": id, "name": "photo.png", "contentType": "image/png",
                    "_attachments": {"index": {"content_type": "image/png",
                        "data": base64::engine::general_purpose::STANDARD.encode(bytes)}}}),
                Put::Upsert,
                Origin::Api,
            )
            .unwrap()
    }

    fn seal_file(store: &Store, id: &str, seal: &str, bytes: &[u8]) -> crate::WriteResult {
        let mut file = store.file_meta(id).unwrap().unwrap();
        FileLock::store(Some(&FileLock::sealed(seal.to_owned())), &mut file.extra);
        store
            .put_sealed_file(file, Attachment::new(bytes.to_vec()))
            .unwrap()
    }

    fn remote_file(id: &str, generation: u64, bytes: &[u8], lock: Option<&str>) -> SyncDoc {
        let blob = Attachment::new(bytes.to_vec());
        let mut fields = json!({"name": "photo.png", "contentType": "image/png",
            "contentLength": bytes.len(), "md5digest": blob.md5_hex(), "createdAt": 1,
            "updatedAt": 9_999_999_999_999i64})
        .as_object()
        .unwrap()
        .clone();
        if let Some(seal) = lock {
            fields.insert(
                FILE_LOCK_FIELD.into(),
                serde_json::to_value(FileLock::sealed(seal.to_owned())).unwrap(),
            );
        }
        SyncDoc {
            id: id.into(),
            rev: format!(
                "{generation}-{}",
                char::from(b'a' + (generation % 26) as u8)
                    .to_string()
                    .repeat(32)
            ),
            deleted: false,
            fields,
            base_rev: None,
        }
    }

    fn plain_doc_with_unseal(mut doc: SyncDoc, seal: &str) -> SyncDoc {
        doc.fields
            .insert(UNSEALED_FIELD.into(), json!({"seal": seal}));
        doc
    }

    /// Pushing R1, editing to R2 and then pulling R1 back through the change
    /// feed must keep R2, even when both share `updatedAt` (merges keep it).
    #[test]
    fn own_push_echo_is_not_a_conflict() {
        let (_dir, store) = store();
        let note = create_note(&store, "note:echo1");
        let pushed = pending(&store, &note.id).unwrap();
        store.mark_synced(&note.id, &pushed.rev, None).unwrap();

        let edited = merge_body(&store, &note.id, "v2");
        assert_eq!(store.note(&note.id).unwrap().unwrap().updated_at, 100);

        let echo = SyncDoc {
            base_rev: None,
            ..pushed.clone()
        };
        assert_eq!(
            store.apply_remote(&echo, None, None).unwrap(),
            ApplyOutcome::Unchanged
        );
        assert_eq!(store.note(&note.id).unwrap().unwrap().body, "v2");
        let still = pending(&store, &note.id).unwrap();
        assert_eq!(still.rev, edited.rev);
        assert_eq!(still.base_rev.as_deref(), Some(pushed.rev.as_str()));
        assert!(
            store
                .revisions(&note.id)
                .unwrap()
                .iter()
                .all(|r| r.origin != "sync"),
            "an echo must not be recorded as a conflicting version"
        );
    }

    /// An edit made while a push was in flight stays pending, based on the
    /// revision the server accepted.
    #[test]
    fn mark_synced_rebases_an_edit_made_during_the_push() {
        let (_dir, store) = store();
        let note = create_note(&store, "note:race1");
        let pushed = pending(&store, &note.id).unwrap();
        assert_eq!(pushed.base_rev, None);

        let edited = merge_body(&store, &note.id, "v2");
        store.mark_synced(&note.id, &pushed.rev, None).unwrap();

        let next = pending(&store, &note.id).unwrap();
        assert_eq!(next.rev, edited.rev);
        assert_eq!(next.base_rev.as_deref(), Some(pushed.rev.as_str()));

        store.mark_synced(&note.id, &edited.rev, None).unwrap();
        assert!(pending(&store, &note.id).is_none());
    }

    /// A file first applied without its bytes (the server had none yet)
    /// takes them when the same revision arrives again with them.
    #[test]
    fn same_revision_with_bytes_fills_a_missing_blob() {
        let (_dir, store) = store();
        let bytes = b"\x89PNG fake".to_vec();
        let blob = Attachment::new(bytes.clone());
        let doc = SyncDoc {
            id: "file:img1".into(),
            rev: "1-aa".into(),
            deleted: false,
            fields: json!({"name": "a.png", "contentType": "image/png", "md5digest": blob.md5_hex(), "createdAt": 1})
                .as_object()
                .unwrap()
                .clone(),
            base_rev: None,
        };
        assert_eq!(
            store.apply_remote(&doc, None, None).unwrap(),
            ApplyOutcome::Applied
        );
        assert!(store.attachment(&doc.id).unwrap().is_none());
        assert!(!store.has_blob(&doc.id, &blob.md5_hex()).unwrap());

        assert_eq!(
            store.apply_remote(&doc, Some(blob.clone()), None).unwrap(),
            ApplyOutcome::Unchanged
        );
        let (content_type, stored) = store.attachment(&doc.id).unwrap().unwrap();
        assert_eq!(content_type, "image/png");
        assert_eq!(stored.data, bytes);
        assert!(store.has_blob(&doc.id, &blob.md5_hex()).unwrap());
        assert!(store.pending_push(10).unwrap().is_empty());
    }

    #[test]
    fn replayed_plaintext_file_cannot_remove_a_local_seal() {
        let (_dir, store) = store();
        let id = "file:seal-replay";
        create_file(&store, id, b"old plaintext");
        let sealed = b"sealed container";
        seal_file(&store, id, "AAAAAAAAAAAAAAAAAAAAAA", sealed);

        let replay = remote_file(id, 1, b"old plaintext", None);
        assert!(matches!(
            store.apply_remote(&replay, None, None).unwrap(),
            ApplyOutcome::LocalWon { .. }
        ));
        assert!(store.file_meta(id).unwrap().unwrap().is_sealed());
        assert_eq!(store.stored_attachment(id).unwrap().unwrap().data, sealed);
    }

    #[test]
    fn a_remote_sealed_file_never_keeps_ocr_text() {
        let (_dir, store) = store();
        let id = "file:seal-ocr";
        create_file(&store, id, b"old");
        let sealed_bytes = b"fresh sealed container";
        let mut remote = remote_file(id, 3, sealed_bytes, Some("AAAAAAAAAAAAAAAAAAAAAA"));
        remote.fields.insert(
            kanso_core::model::OCR_FIELD.into(),
            json!({"text": "secret words", "md5": "whatever", "engine": "test"}),
        );
        assert!(matches!(
            store
                .apply_remote(&remote, Some(Attachment::new(sealed_bytes.to_vec())), None)
                .unwrap(),
            ApplyOutcome::Applied | ApplyOutcome::RemoteWon
        ));
        let meta = store.file_meta(id).unwrap().unwrap();
        assert!(!meta.extra.contains_key(kanso_core::model::OCR_FIELD));
        assert_eq!(store.image_text(id).unwrap(), None);
    }

    #[test]
    fn a_matching_unseal_without_bytes_drops_the_old_container() {
        let (_dir, store) = store();
        let id = "file:seal-awaiting-bytes";
        create_file(&store, id, b"old");
        let sealed = seal_file(&store, id, "AAAAAAAAAAAAAAAAAAAAAA", b"container");
        store.mark_synced(id, &sealed.rev, None).unwrap();

        let plain = b"deliberately unsealed";
        let remote =
            plain_doc_with_unseal(remote_file(id, 3, plain, None), "AAAAAAAAAAAAAAAAAAAAAA");
        assert_eq!(
            store.apply_remote(&remote, None, None).unwrap(),
            ApplyOutcome::Applied
        );
        assert!(!store.file_meta(id).unwrap().unwrap().is_sealed());
        assert_eq!(store.stored_attachment(id).unwrap(), None);

        assert_eq!(
            store
                .apply_remote(&remote, Some(Attachment::new(plain.to_vec())), None)
                .unwrap(),
            ApplyOutcome::Unchanged
        );
        assert_eq!(store.stored_attachment(id).unwrap().unwrap().data, plain);
    }

    #[test]
    fn matching_unseal_marker_allows_a_remote_unseal() {
        let (_dir, store) = store();
        let id = "file:seal-accept";
        create_file(&store, id, b"old");
        let sealed = seal_file(&store, id, "AAAAAAAAAAAAAAAAAAAAAA", b"container");
        store.mark_synced(id, &sealed.rev, None).unwrap();

        let plain = b"deliberately unsealed";
        let remote =
            plain_doc_with_unseal(remote_file(id, 3, plain, None), "AAAAAAAAAAAAAAAAAAAAAA");
        assert_eq!(
            store
                .apply_remote(&remote, Some(Attachment::new(plain.to_vec())), None)
                .unwrap(),
            ApplyOutcome::Applied
        );
        let meta = store.file_meta(id).unwrap().unwrap();
        assert!(!meta.is_sealed());
        assert!(!meta.extra.contains_key(UNSEALED_FIELD));
        assert_eq!(store.stored_attachment(id).unwrap().unwrap().data, plain);
    }

    #[test]
    fn mismatched_unseal_marker_cannot_remove_a_local_seal() {
        let (_dir, store) = store();
        let id = "file:seal-refuse";
        create_file(&store, id, b"old");
        let sealed_bytes = b"container";
        seal_file(&store, id, "AAAAAAAAAAAAAAAAAAAAAA", sealed_bytes);

        let remote = plain_doc_with_unseal(
            remote_file(id, 3, b"plaintext", None),
            "BBBBBBBBBBBBBBBBBBBBBB",
        );
        assert!(matches!(
            store
                .apply_remote(&remote, Some(Attachment::new(b"plaintext".to_vec())), None)
                .unwrap(),
            ApplyOutcome::LocalWon { .. }
        ));
        assert!(store.file_meta(id).unwrap().unwrap().is_sealed());
        assert_eq!(
            store.stored_attachment(id).unwrap().unwrap().data,
            sealed_bytes
        );
    }

    #[test]
    fn a_new_sealed_document_drops_stale_blob_until_its_bytes_arrive() {
        let (_dir, store) = store();
        let id = "file:seal-replace";
        let initial = create_file(&store, id, b"stale plaintext");
        store.mark_synced(id, &initial.rev, None).unwrap();

        let sealed_bytes = b"fresh sealed container";
        let remote = remote_file(id, 3, sealed_bytes, Some("AAAAAAAAAAAAAAAAAAAAAA"));
        assert_eq!(
            store.apply_remote(&remote, None, None).unwrap(),
            ApplyOutcome::Applied
        );
        assert!(store.file_meta(id).unwrap().unwrap().is_sealed());
        assert!(store.stored_attachment(id).unwrap().is_none());

        assert_eq!(
            store
                .apply_remote(&remote, Some(Attachment::new(sealed_bytes.to_vec())), None)
                .unwrap(),
            ApplyOutcome::Unchanged
        );
        assert_eq!(
            store.stored_attachment(id).unwrap().unwrap().data,
            sealed_bytes
        );
    }

    #[test]
    fn a_rejected_old_sealed_document_keeps_the_current_blob() {
        let (_dir, store) = store();
        let id = "file:seal-old";
        create_file(&store, id, b"plain");
        let current = b"current sealed container";
        let current_doc = seal_file(&store, id, "AAAAAAAAAAAAAAAAAAAAAA", current);
        store.mark_synced(id, &current_doc.rev, None).unwrap();

        let replay = remote_file(
            id,
            1,
            b"old sealed container",
            Some("BBBBBBBBBBBBBBBBBBBBBB"),
        );
        assert!(matches!(
            store.apply_remote(&replay, None, None).unwrap(),
            ApplyOutcome::Stale { .. }
        ));
        assert_eq!(store.stored_attachment(id).unwrap().unwrap().data, current);
    }

    fn guard_meta(store: &Store, origin: &str, account: &str) {
        store.set_meta("sync.v2.origin", origin).unwrap();
        store.set_meta("sync.v2.account_id", account).unwrap();
    }

    const BOUND: [(&str, &str); 2] = [
        ("sync.v2.origin", "https://a.example"),
        ("sync.v2.account_id", "a_1"),
    ];

    /// A guarded write lands only while every guarded meta entry matches,
    /// and a refused one writes nothing.
    #[test]
    fn guarded_writes_need_the_whole_binding() {
        let (_dir, store) = store();
        let note = create_note(&store, "note:guard1");
        let rev = pending(&store, &note.id).unwrap().rev;
        let guard = SyncGuard { expect: &BOUND };

        assert!(matches!(
            store.mark_synced(&note.id, &rev, Some(&guard)),
            Err(StoreError::GuardFailed)
        ));
        guard_meta(&store, "https://b.example", "a_1");
        assert!(matches!(
            store.mark_synced(&note.id, &rev, Some(&guard)),
            Err(StoreError::GuardFailed)
        ));
        assert!(matches!(
            store.set_meta_if("sync.v2.last_seq", "9", &guard),
            Err(StoreError::GuardFailed)
        ));
        assert_eq!(store.meta("sync.v2.last_seq").unwrap(), None);
        assert_eq!(store.pending_count().unwrap(), 1);

        guard_meta(&store, "https://a.example", "a_1");
        store.mark_synced(&note.id, &rev, Some(&guard)).unwrap();
        store.set_meta_if("sync.v2.last_seq", "9", &guard).unwrap();
        assert_eq!(
            store.meta("sync.v2.last_seq").unwrap().as_deref(),
            Some("9")
        );
        assert_eq!(store.pending_count().unwrap(), 0);

        guard_meta(&store, "https://a.example", "a_2");
        let remote = SyncDoc {
            rev: "9-ff".into(),
            ..pending_doc(&note.id)
        };
        assert!(matches!(
            store.apply_remote(&remote, None, Some(&guard)),
            Err(StoreError::GuardFailed)
        ));
        assert_eq!(
            store.get_api(&note.id, None, false).unwrap()["_rev"],
            rev.as_str()
        );
    }

    fn pending_doc(id: &str) -> SyncDoc {
        SyncDoc {
            id: id.into(),
            rev: "1-aa".into(),
            deleted: false,
            fields: json!({"bookId": "trash", "body": "remote", "updatedAt": 1})
                .as_object()
                .unwrap()
                .clone(),
            base_rev: None,
        }
    }

    /// A document the server lost goes back to pending with no base.
    #[test]
    fn mark_unsynced_makes_a_document_pending_again() {
        let (_dir, store) = store();
        let note = create_note(&store, "note:lost1");
        let rev = pending(&store, &note.id).unwrap().rev;
        store.mark_synced(&note.id, &rev, None).unwrap();
        assert!(pending(&store, &note.id).is_none());
        store.mark_unsynced(&note.id, None).unwrap();
        let again = pending(&store, &note.id).unwrap();
        assert_eq!((again.rev, again.base_rev), (rev, None));
    }

    fn state(store: &Store, id: &str) -> SyncState {
        store
            .sync_states()
            .unwrap()
            .into_iter()
            .find(|s| s.id == id)
            .unwrap()
    }

    fn live(id: &str, rev: &str, body: &str) -> SyncDoc {
        SyncDoc {
            id: id.into(),
            rev: rev.into(),
            deleted: false,
            fields: json!({"bookId": "trash", "body": body, "updatedAt": 9_000})
                .as_object()
                .unwrap()
                .clone(),
            base_rev: None,
        }
    }

    /// Security review, M2 bypass: a server that answers `Lost`, or leaves a
    /// document out of a re-merge, and then offers an old version can't
    /// bring a deleted note back. The floor survives both.
    #[test]
    fn lost_and_missing_keep_the_floor() {
        for missing in [false, true] {
            let (_dir, store) = store();
            let id = "note:floor1";
            let old = format!("3-{}", "a".repeat(32));
            store
                .apply_remote(&live(id, &old, "months old"), None, None)
                .unwrap();
            store
                .apply_remote(
                    &live(id, &format!("5-{}", "b".repeat(32)), "current"),
                    None,
                    None,
                )
                .unwrap();
            store.delete(id, Origin::Api).unwrap();
            let tombstone = pending(&store, id).unwrap().rev;
            store.mark_synced(id, &tombstone, None).unwrap();
            assert_eq!(state(&store, id).synced_gen, 6);
            if missing {
                let gone = store
                    .mark_missing_unsynced(&std::collections::HashSet::new(), None)
                    .unwrap();
                assert_eq!(gone, vec![id.to_owned()]);
            } else {
                store.mark_unsynced(id, None).unwrap();
            }
            let after = state(&store, id);
            assert_eq!((after.synced_rev, after.synced_gen), (None, 6));
            let outcome = store
                .apply_remote(&live(id, &old, "months old"), None, None)
                .unwrap();
            assert!(matches!(outcome, ApplyOutcome::Stale { .. }), "{outcome:?}");
            assert!(
                store.get(id).unwrap().is_none(),
                "the deleted note came back"
            );
            assert_eq!(state(&store, id).synced_gen, 6);
        }
    }

    /// Review-sync 1: a tombstone `reset_sync` leaves was never on the new
    /// account. It isn't pending, the account's live copy beats it at any
    /// generation, a re-merge doesn't count it missing, and a note
    /// re-created over it is pushed with no base.
    #[test]
    fn a_reset_tombstone_was_never_on_the_account() {
        for remote_gen in 1..=4u64 {
            let (_dir, store) = store();
            let id = "note:reset1";
            create_note(&store, id);
            let first = pending(&store, id).unwrap().rev;
            store.mark_synced(id, &first, None).unwrap();
            store.delete(id, Origin::Api).unwrap();
            store.reset_sync(&[]).unwrap();
            let reset = state(&store, id);
            assert!(reset.never_on_account() && !reset.pending());
            assert!(pending(&store, id).is_none());
            assert!(
                store
                    .mark_missing_unsynced(&std::collections::HashSet::new(), None)
                    .unwrap()
                    .is_empty()
            );
            let theirs = live(
                id,
                &format!("{remote_gen}-{}", "c".repeat(32)),
                "the account's",
            );
            assert_eq!(
                store.apply_remote(&theirs, None, None).unwrap(),
                ApplyOutcome::Applied,
                "generation {remote_gen}"
            );
            assert_eq!(store.note(id).unwrap().unwrap().body, "the account's");
        }

        let (_dir, store) = store();
        create_note(&store, "note:reset2");
        store.delete("note:reset2", Origin::Api).unwrap();
        store.reset_sync(&[]).unwrap();
        create_note(&store, "note:reset2");
        let again = pending(&store, "note:reset2").unwrap();
        assert_eq!(again.base_rev, None, "the account never had the tombstone");
    }

    /// Review-sync 3: a push whose answer was lost, then an API merge that
    /// keeps `updatedAt`. The echo of the push is ours: the server has it,
    /// the newer merge stays pending on top of it, and nothing conflicts.
    #[test]
    fn the_echo_of_a_sent_revision_is_ours() {
        let (_dir, store) = store();
        let note = create_note(&store, "note:sent1");
        let first = pending(&store, &note.id).unwrap();
        store.mark_synced(&note.id, &first.rev, None).unwrap();
        let sent = merge_body(&store, &note.id, "v2");
        store.mark_sent(&[(&sent.id, &sent.rev)], None).unwrap();
        assert_eq!(
            state(&store, &note.id).sent_rev.as_deref(),
            Some(sent.rev.as_str())
        );
        let newer = merge_body(&store, &note.id, "v3");
        let echo = SyncDoc {
            rev: sent.rev.clone(),
            fields: store
                .revisions(&note.id)
                .unwrap()
                .into_iter()
                .find(|r| r.rev == sent.rev)
                .unwrap()
                .fields,
            ..first.clone()
        };
        assert_eq!(
            store.apply_remote(&echo, None, None).unwrap(),
            ApplyOutcome::Unchanged
        );
        assert_eq!(store.note(&note.id).unwrap().unwrap().body, "v3");
        let next = pending(&store, &note.id).unwrap();
        assert_eq!(
            (next.rev, next.base_rev),
            (newer.rev, Some(sent.rev.clone()))
        );
        let after = state(&store, &note.id);
        assert_eq!((after.sent_rev, after.synced_gen), (None, 2));
        assert!(
            store
                .revisions(&note.id)
                .unwrap()
                .iter()
                .all(|r| r.origin != "sync")
        );
    }

    /// Review-sync 5: files without their bytes are listed, and fetched
    /// bytes land only for the revision they were fetched for and only when
    /// they match its `md5digest`.
    #[test]
    fn missing_bytes_are_listed_and_filled_only_for_their_revision() {
        let (_dir, store) = store();
        let bytes = b"\x89PNG later".to_vec();
        let blob = Attachment::new(bytes.clone());
        let doc = |id: &str, rev: &str, md5: &str| {
            SyncDoc {
            id: id.into(),
            rev: rev.into(),
            deleted: false,
            fields: json!({"name": "a.png", "contentType": "image/png", "md5digest": md5, "createdAt": 1})
                .as_object()
                .unwrap()
                .clone(),
            base_rev: None,
        }
        };
        let md5 = blob.md5_hex().to_uppercase();
        for id in ["file:b", "file:a", "file:c"] {
            store
                .apply_remote(&doc(id, "1-aa", &md5), None, None)
                .unwrap();
        }
        store
            .apply_remote(&doc("file:d", "1-aa", &md5), Some(blob.clone()), None)
            .unwrap();
        let ids = |after: &str, limit: usize| -> Vec<String> {
            store
                .files_without_bytes(after, limit)
                .unwrap()
                .into_iter()
                .map(|(id, _, _)| id)
                .collect()
        };
        assert_eq!(ids("", usize::MAX), ["file:a", "file:b", "file:c"]);
        assert_eq!(ids("file:a", 1), ["file:b"]);

        let wrong = Attachment::new(b"other".to_vec());
        assert!(!store.fill_blob("file:a", "1-aa", wrong, None).unwrap());
        assert!(
            !store
                .fill_blob("file:a", "2-bb", blob.clone(), None)
                .unwrap()
        );
        assert!(
            store
                .fill_blob("file:a", "1-aa", blob.clone(), None)
                .unwrap()
        );
        assert_eq!(store.attachment("file:a").unwrap().unwrap().1.data, bytes);
        assert_eq!(ids("", usize::MAX), ["file:b", "file:c"]);
        let guard_pairs = [("sync.v2.account_id", "a_1")];
        let guard = SyncGuard {
            expect: &guard_pairs,
        };
        assert!(matches!(
            store.fill_blob("file:b", "1-aa", blob, Some(&guard)),
            Err(StoreError::GuardFailed)
        ));
    }

    /// Migration 2 starts each floor at the generation of `synced_rev`.
    #[test]
    fn the_migration_fills_the_floor_from_synced_rev() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kanso.db");
        {
            let conn = rusqlite::Connection::open(&path).unwrap();
            conn.execute_batch(&format!(
                "{}; PRAGMA user_version = 1;",
                crate::schema::MIGRATIONS[0]
            ))
            .unwrap();
            conn.execute(
                "INSERT INTO docs (id, type, rev, seq, deleted, json, synced_rev) VALUES
                 ('note:m1', 'note', '7-aa', 1, 0, '{}', '12-bb'),
                 ('note:m2', 'note', '2-aa', 2, 0, '{}', NULL)",
                [],
            )
            .unwrap();
        }
        let store = Store::open(&path).unwrap();
        assert_eq!(state(&store, "note:m1").synced_gen, 12);
        assert_eq!(state(&store, "note:m2").synced_gen, 0);
    }

    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]

        /// Model-based: `pending_count` equals the length of `pending_push`
        /// (the oracle), and `mark_missing_unsynced` turns exactly the synced
        /// documents missing from `present` (tombstones included) back into
        /// pending ones.
        #[test]
        fn prop_missing_documents_become_pending(
            docs in prop::collection::vec((any::<bool>(), any::<bool>(), any::<bool>()), 1..8),
        ) {
            let (_dir, store) = store();
            let mut present = std::collections::HashSet::new();
            let mut expected_missing = Vec::new();
            for (i, (synced, deleted, in_feed)) in docs.iter().enumerate() {
                let id = format!("note:m{i}");
                create_note(&store, &id);
                if *deleted {
                    store.delete(&id, Origin::Api).unwrap();
                }
                if *synced {
                    let rev = pending(&store, &id).unwrap().rev;
                    store.mark_synced(&id, &rev, None).unwrap();
                    if !*in_feed {
                        expected_missing.push(id.clone());
                    }
                }
                if *in_feed {
                    present.insert(id);
                }
                prop_assert_eq!(store.pending_count().unwrap(), store.pending_push(usize::MAX).unwrap().len());
            }
            let before: std::collections::BTreeSet<String> =
                store.pending_push(usize::MAX).unwrap().into_iter().map(|d| d.id).collect();
            expected_missing.sort();
            let missing = store.mark_missing_unsynced(&present, None).unwrap();
            prop_assert_eq!(&missing, &expected_missing);
            let after: std::collections::BTreeSet<String> =
                store.pending_push(usize::MAX).unwrap().into_iter().map(|d| d.id).collect();
            let mut expected = before;
            expected.extend(expected_missing);
            prop_assert_eq!(after, expected);
            prop_assert!(store.pending_push(usize::MAX).unwrap().iter().filter(|d| missing.contains(&d.id)).all(|d| d.base_rev.is_none()));
        }
    }
}
