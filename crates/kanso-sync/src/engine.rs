//! One sync pass: pull the server's changes, then push local ones; and what
//! a sync reports (docs/ACCOUNTS.md › Sync with encryption, › Problems).
//!
//! [`sync`] is the only engine: the v2 rules over `Sealed` (sealed
//! documents, opaque IDs, the guard, re-merge, repair, quotas and pauses).
//! [`SyncReport`], [`SyncOptions`], [`SyncError`], [`SyncProblem`],
//! [`problem`] and [`report_problem`] are frozen by step 1.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::Range;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use kanso_store::{ApplyOutcome, Attachment, Store, StoreError, SyncDoc, SyncGuard};
use serde_json::Value;

use crate::account::{Session, Usage};
use crate::api::Backend;
use crate::binding::meta_keys;
use crate::remote::{is_file_id, md5_hex};
use crate::sealed::{Entry, FileBytes, Pushed, Sealed};
use crate::wire::{ApiError, QuotaKind, SignOutReason, WireChange};

#[derive(Clone, Debug)]
pub struct SyncOptions {
    /// Documents per `changes` page.
    pub page_size: usize,
    /// Most documents per push.
    pub batch_docs: usize,
    /// Target serialized size of one push; a larger document goes alone.
    pub batch_bytes: usize,
    /// Push rounds per sync. A round resolves the conflicts of the one before.
    pub max_rounds: usize,
    /// Set to stop the sync: the engine checks it between pages, batches and
    /// files and returns [`SyncError::Cancelled`]. Every re-bind and
    /// sign-out cancels and joins the running sync first (fix 2).
    pub cancel: Option<Arc<AtomicBool>>,
    /// Most live files whose bytes this Mac lacks that one sync fetches
    /// again, after the pull (docs/ACCOUNTS.md › Sync with encryption ›
    /// Files). 0 turns it off.
    pub refetch_files: usize,
}

impl Default for SyncOptions {
    fn default() -> Self {
        SyncOptions {
            page_size: 100,
            batch_docs: 100,
            batch_bytes: 1024 * 1024,
            max_rounds: 4,
            cancel: None,
            refetch_files: 8,
        }
    }
}

/// What a sync did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SyncReport {
    /// Remote changes written locally, including conflicts the remote won.
    pub pulled: usize,
    /// Local changes the server accepted.
    pub pushed: usize,
    /// Conflicts the local version won; the remote one is in its revisions.
    pub conflicts_local: usize,
    /// Conflicts the remote version won; the local one is in its revisions.
    pub conflicts_remote: usize,
    pub files_up: usize,
    pub files_down: usize,
    /// Attachments a document refers to whose bytes neither side had.
    pub files_missing: usize,
    /// Local changes still not on the server after this sync: edited during
    /// it, conflicts that kept racing, or listed below.
    pub pending: usize,
    /// Documents that had a conflict this sync, for the UI to flag.
    pub conflicted: Vec<String>,
    /// Documents too large for the server: `fields` JSON over 1,400,000
    /// bytes (`wire::MAX_FIELDS_JSON`).
    pub too_large: Vec<String>,
    /// Remote documents the store could not accept, with the reason.
    pub invalid: Vec<(String, String)>,
    /// Wire IDs of feed documents (format 1) that couldn't be opened and
    /// weren't repaired; the cursor moved past them. `kanso-sync verify
    /// --repair` handles them (security re-check 10).
    pub skipped: Vec<String>,
    /// Local IDs whose server revision was older than this Mac's synced one
    /// (M2): the local version was rebased onto it and pushed again.
    pub stale: Vec<String>,
    /// Local IDs whose undecryptable server copy is being replaced from this
    /// Mac (M3): only when the broken revision is `wire_rev(id, synced_rev)`.
    pub repaired: Vec<String>,
    /// A feed document uses a newer format: the pull stopped before it.
    pub too_new: bool,
    /// The server's copy was restored or rewound (epoch or head changed), and
    /// this Mac re-merged (M1).
    pub remerged: bool,
    /// A push or upload hit this quota (413), now or in an earlier sync
    /// whose entry is still in place: what would grow the account is held
    /// back until usage drops, and deletions still go through.
    pub quota: Option<QuotaKind>,
    /// A 429 on a push or upload: pushing stopped until this time (ms).
    pub paused_until: Option<i64>,
    /// The `limit` of the 429 behind `paused_until`: `writes-account`,
    /// `uploads-account` or `writes-global`. One ending in `-global` is the
    /// server's own daily cap, which gives [`SyncProblem::ServerBusy`].
    pub paused_limit: Option<String>,
    /// After a 429 `uploads-global`: file uploads, and the live file
    /// documents waiting for them, are held until this time (ms). Pulls and
    /// everything else keep syncing, and it is no problem.
    pub uploads_held_until: Option<i64>,
}

impl SyncReport {
    pub fn conflicts(&self) -> usize {
        self.conflicts_local + self.conflicts_remote
    }

    fn note_conflict(&mut self, id: &str) {
        if !self.conflicted.iter().any(|c| c == id) {
            self.conflicted.push(id.to_owned());
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error(transparent)]
    Store(#[from] StoreError),
    /// From a v2 route. A 401 ends the session; a 404 means the server isn't
    /// v2 (`ServerOutdated`).
    #[error(transparent)]
    Api(#[from] ApiError),
    /// The server's key ID differs from this Mac's: nothing was written.
    #[error("the server's account keys don't match this device's")]
    WrongKey,
    /// [`SyncOptions::cancel`] was set, or the library was re-bound while
    /// the sync ran (the store's guard failed).
    #[error("sync cancelled")]
    Cancelled,
    /// A 429 `session-account` (too many requests a minute) on any sync
    /// route: the sync stopped where it was, with nothing after it marked
    /// done and the cursor at the last page applied. The next automatic sync
    /// waits until `until` (ms) ([`SyncProblem::BackOff`]).
    #[error("the server asked this device to slow down until {until}")]
    BackOff { until: i64 },
}

/// The 429 `limit` of the per-account request limit on session routes.
const SESSION_LIMIT: &str = "session-account";

/// The longest wait a 429's `retry_after` can set (review-verify 5): a
/// server's answer, however large, never holds this Mac back more than a
/// day.
const RETRY_AFTER_MAX_S: u64 = 24 * 60 * 60;

/// The furthest ahead a saved time may be (review-verify 5): a day and an
/// hour. One further ahead was saved while the clock ran ahead (or before
/// `retry_after` was bounded), and is dropped when read, so nothing waits
/// on it.
const SAVED_AHEAD_MAX_MS: i64 = 25 * 60 * 60 * 1000;

/// When a 429's wait of `retry_after` seconds ends (ms), at most
/// [`RETRY_AFTER_MAX_S`] from now.
fn retry_at(retry_after: u64) -> i64 {
    let secs = i64::try_from(retry_after.min(RETRY_AFTER_MAX_S)).unwrap_or(i64::MAX);
    now_ms().saturating_add(secs.saturating_mul(1000))
}

/// Whether a saved time `at` (ms) is within [`SAVED_AHEAD_MAX_MS`] of `now`.
fn not_too_far_ahead(at: i64, now: i64) -> bool {
    at <= now.saturating_add(SAVED_AHEAD_MAX_MS)
}

/// A 429 `session-account`: [`SyncError::BackOff`] for `retry_after` seconds.
fn back_off(retry_after: u64) -> SyncError {
    SyncError::BackOff {
        until: retry_at(retry_after),
    }
}

/// The app's sync: one pass of `session`'s account over `b`, with every v2
/// rule (docs/ACCOUNTS.md › Sync with encryption, the order below).
///
/// 1. **Pull.** Pages through the feed after the cursor
///    (`sync.v2.last_seq`), opening each document. The first page
///    is checked before anything is written: a key ID other than this
///    Mac's is [`SyncError::WrongKey`]; an epoch other than the saved one,
///    or a `head_seq` below the cursor, means the server's copy was
///    restored or rewound, and the Mac re-merges (M1): the cursor goes to
///    0, the whole feed is applied through `apply_remote` (so replayed
///    revisions come back `Stale` and are rebased), and the documents the
///    feed lacked are marked unsynced and pushed again with no base.
///    Undecryptable documents never stop the pull; one is repaired only
///    when it is the version this Mac last synced (M3), else skipped. A
///    newer format stops the pull before it. Then up to
///    [`SyncOptions::refetch_files`] live files whose bytes this Mac lacks
///    are fetched again.
/// 2. **Push.** Sends pending documents in batches, sealed, uploading file
///    bytes first; up to [`SyncOptions::max_rounds`] rounds resolve
///    conflicts, lost documents and `behind` answers (which pull first). A
///    413 is remembered per family (documents, uploads) with a level of
///    what it holds back (new documents, then edits, then everything): the
///    batch is retried at once without them, and later syncs send them
///    only once the account's usage dropped, within the room left. A 429
///    stops pushing, except `uploads-global`, which only holds uploads.
///
/// Every store write carries the session's guard
/// ([`crate::binding::guard_pairs`]); if the library was re-bound
/// meanwhile, or [`SyncOptions::cancel`] is set, the sync stops with
/// [`SyncError::Cancelled`]. Nothing is pushed unless the library is bound
/// to the session's account.
pub fn sync(
    store: &Store,
    b: &dyn Backend,
    session: &Session,
    options: &SyncOptions,
) -> Result<SyncReport, SyncError> {
    let pairs = crate::binding::guard_pairs(session);
    let sealed = Sealed::new(b.remote(session), session);
    let mut run = Run {
        store,
        backend: b,
        session,
        sealed: &sealed,
        guard: SyncGuard { expect: &pairs },
        options,
        report: SyncReport::default(),
        epoch: None,
        remerging: false,
        present: HashSet::new(),
        wire_ids: None,
        fetched: HashSet::new(),
        stopped: false,
        holds: [Hold::default(), Hold::default()],
        usage: None,
        uploads_until: None,
    };
    let steps = (|| {
        run.check_cancel()?;
        run.check_bound()?;
        run.pull()?;
        run.refetch()?;
        run.push()
    })();
    match steps {
        Err(SyncError::Api(ApiError::RateLimited { retry_after, limit }))
            if limit == SESSION_LIMIT =>
        {
            return Err(back_off(retry_after));
        }
        Err(err) => return Err(err),
        Ok(()) => {}
    }
    run.report.pending = store.pending_count()?;
    Ok(run.report)
}

/// The value of [`meta_keys::EPOCH`] while a re-merge hasn't finished. It is
/// never a server's epoch (16 hex digits), so an interrupted re-merge runs
/// again from the start at the next sync.
const REMERGING: &str = "remerging";

/// Turns the store's guard failure into [`SyncError::Cancelled`].
fn guarded(err: StoreError) -> SyncError {
    match err {
        StoreError::GuardFailed => SyncError::Cancelled,
        err => SyncError::Store(err),
    }
}

/// How long a file whose bytes a refetch missed waits before it is asked
/// for again (review-verify 4): bytes gone for good, or not matching the
/// document, would otherwise be fetched by every sync.
const REFETCH_EVERY_MS: i64 = 24 * 60 * 60 * 1000;

/// Most files [`meta_keys::REFETCH_MISSED`] remembers. Past it, those due
/// soonest are forgotten, and may be asked for again before their day.
const REFETCH_MISSED_MAX: usize = 512;

/// The files still waiting after a missed refetch, from the
/// [`meta_keys::REFETCH_MISSED`] value: each with the time (ms) it may be
/// asked for again, all after `now` and at most [`SAVED_AHEAD_MAX_MS`]
/// ahead. A damaged value counts as none.
fn missed_files(value: Option<&str>, now: i64) -> BTreeMap<String, i64> {
    let mut missed: BTreeMap<String, i64> = value
        .and_then(|value| serde_json::from_str(value).ok())
        .unwrap_or_default();
    missed.retain(|_, at| *at > now && not_too_far_ahead(*at, now));
    missed
}

/// Keeps at most [`REFETCH_MISSED_MAX`] waiting files: those due latest.
fn bound_missed(missed: &mut BTreeMap<String, i64>) {
    while missed.len() > REFETCH_MISSED_MAX {
        let soonest = missed
            .iter()
            .min_by_key(|(id, at)| (**at, id.as_str()))
            .map(|(id, _)| id.clone());
        match soonest {
            Some(id) => missed.remove(&id),
            None => break,
        };
    }
}

/// The content hash of a local revision (`{generation}-{hash}`).
fn content_hash(rev: &str) -> &str {
    rev.split_once('-').map_or(rev, |(_, hash)| hash)
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as i64)
}

/// An upper bound of a document's size on the wire, sealed, from its
/// `fields` JSON length: the plaintext adds the ID, the revision and 30
/// bytes of framing; sealing adds 41 bytes and base64url a third; the
/// push entry adds its wire ID, revisions and keys.
fn wire_size(doc: &SyncDoc, fields_len: usize) -> usize {
    let plaintext = fields_len + doc.id.len() + doc.rev.len() + 30;
    (plaintext + 41).div_ceil(3) * 4 + 160
}

/// What a remembered 413 holds back (docs/ACCOUNTS.md › Sync with
/// encryption › Quota). Each level holds what the ones before it hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Level {
    /// What the server doesn't have yet: documents with no base (new IDs
    /// and never-synced tombstones), and uploads of new files.
    New,
    /// Also live documents the server has, and their uploads.
    Edits,
    /// The whole family.
    All,
}

impl Level {
    fn as_str(self) -> &'static str {
        match self {
            Level::New => "new",
            Level::Edits => "edits",
            Level::All => "all",
        }
    }
}

/// What a pending document is to a quota, in growing order of what it may
/// add to the account.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Class {
    /// A tombstone of a document the server has: it never grows the account.
    Tombstone,
    /// A live document the server has.
    Edit,
    /// A document the server doesn't have (no base).
    New,
}

impl Class {
    fn of(doc: &SyncDoc) -> Class {
        if doc.base_rev.is_none() {
            Class::New
        } else if doc.deleted {
            Class::Tombstone
        } else {
            Class::Edit
        }
    }

    /// The level a refused request whose worst document is of this class
    /// moves to: it holds that class.
    fn level(self) -> Level {
        match self {
            Class::New => Level::New,
            Class::Edit => Level::Edits,
            Class::Tombstone => Level::All,
        }
    }

    fn held_at(self, level: Level) -> bool {
        match level {
            Level::New => self == Class::New,
            Level::Edits => self != Class::Tombstone,
            Level::All => true,
        }
    }
}

/// A quota family: a 413 on a push holds documents, one on an upload holds
/// uploads (and the file documents waiting for them).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Family {
    Docs = 0,
    Files = 1,
}

impl Family {
    fn key(self) -> &'static str {
        match self {
            Family::Docs => meta_keys::QUOTA_DOCS,
            Family::Files => meta_keys::QUOTA_FILES,
        }
    }
}

/// A remembered 413, as `<kind> <used> <limit> <at_ms> <level>` in
/// [`Family::key`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct QuotaEntry {
    kind: QuotaKind,
    /// The kind's usage before the refused request, as the 413 said.
    used: u64,
    /// The kind's quota, as the 413 said.
    limit: u64,
    at_ms: i64,
    level: Level,
}

/// How long a `server` entry, which `GET /v2/account` can't measure, holds
/// before the next try.
const SERVER_HOLD_MS: i64 = 60 * 60 * 1000;

impl QuotaEntry {
    fn parse(value: &str) -> Option<QuotaEntry> {
        let mut parts = value.split(' ');
        let kind = parts.next()?;
        let kind = QuotaKind::ALL.into_iter().find(|k| k.as_str() == kind)?;
        let used = parts.next()?.parse().ok()?;
        let limit = parts.next()?.parse().ok()?;
        let at_ms = parts.next()?.parse().ok()?;
        let level = match parts.next()? {
            "new" => Level::New,
            "edits" => Level::Edits,
            "all" => Level::All,
            _ => return None,
        };
        parts.next().is_none().then_some(QuotaEntry {
            kind,
            used,
            limit,
            at_ms,
            level,
        })
    }

    fn encode(&self) -> String {
        format!(
            "{} {} {} {} {}",
            self.kind.as_str(),
            self.used,
            self.limit,
            self.at_ms,
            self.level.as_str()
        )
    }

    /// The room left in `family` when usage dropped since the 413 (or, for
    /// a `server` entry, an hour went by); `None` while it hasn't.
    fn room(&self, family: Family, usage: &Usage, now: i64) -> Option<Room> {
        let quota = &usage.quota;
        let dropped = match self.kind {
            QuotaKind::Docs => usage.docs < self.used || quota.docs > self.limit,
            QuotaKind::DocBytes => usage.doc_bytes < self.used || quota.doc_bytes > self.limit,
            QuotaKind::Files => usage.files < self.used || quota.files > self.limit,
            QuotaKind::FileBytes => usage.file_bytes < self.used || quota.file_bytes > self.limit,
            QuotaKind::Server => now >= self.at_ms.saturating_add(SERVER_HOLD_MS),
        };
        dropped.then(|| match family {
            Family::Docs => Room {
                count: quota.docs.saturating_sub(usage.docs),
                bytes: quota.doc_bytes.saturating_sub(usage.doc_bytes),
            },
            Family::Files => Room {
                count: quota.files.saturating_sub(usage.files),
                bytes: quota.file_bytes.saturating_sub(usage.file_bytes),
            },
        })
    }
}

/// What held documents may still add once usage dropped: new live
/// documents (or new files) and bytes.
#[derive(Clone, Copy, Debug)]
struct Room {
    count: u64,
    bytes: u64,
}

/// What the server charges a document row on top of its sealed text.
const DOC_ROW_COST: u64 = 512;
/// What the server charges a file on top of its sealed bytes, plus what
/// sealing adds.
const FILE_ROW_COST: u64 = 1024 + 64;

/// One family's quota state in a sync.
#[derive(Debug, Default)]
struct Hold {
    entry: Option<QuotaEntry>,
    /// The room left, once worked out this sync; `None` inside holds per
    /// the entry's level.
    room: Option<Option<Room>>,
    /// Something of the family was held back this sync.
    held_back: bool,
    /// A request of the family got a 413 this sync.
    refused: bool,
    /// Documents whose cost was taken from the room this sync.
    charged: HashSet<String>,
}

/// One run of [`sync`].
struct Run<'r> {
    store: &'r Store,
    backend: &'r dyn Backend,
    session: &'r Session,
    sealed: &'r Sealed<'r>,
    guard: SyncGuard<'r>,
    options: &'r SyncOptions,
    report: SyncReport,
    /// The epoch of this sync's first page.
    epoch: Option<String>,
    /// A re-merge is running: `present` collects the local IDs the feed
    /// carries.
    remerging: bool,
    present: HashSet<String>,
    /// Wire ID → (local ID, `synced_rev`) of every local document, built
    /// when the first undecryptable document of a pull shows up.
    wire_ids: Option<HashMap<String, (String, Option<String>)>>,
    /// File IDs whose bytes this sync already asked the server for.
    fetched: HashSet<String>,
    /// Pushing stopped for this sync (a 429).
    stopped: bool,
    /// The quota entries of [`Family::Docs`] and [`Family::Files`].
    holds: [Hold; 2],
    /// `GET /v2/account`, read at most once a sync (`None` inside when it
    /// failed).
    usage: Option<Option<Usage>>,
    /// Uploads are held until then (a 429 `uploads-global`).
    uploads_until: Option<i64>,
}

impl Run<'_> {
    fn check_cancel(&self) -> Result<(), SyncError> {
        match &self.options.cancel {
            Some(flag) if flag.load(std::sync::atomic::Ordering::SeqCst) => {
                Err(SyncError::Cancelled)
            }
            _ => Ok(()),
        }
    }

    /// The library must still be bound to the session's account before
    /// anything of it is sent there.
    fn check_bound(&self) -> Result<(), SyncError> {
        for (key, value) in self.guard.expect {
            if self.store.meta(key)?.as_deref() != Some(*value) {
                return Err(SyncError::Cancelled);
            }
        }
        Ok(())
    }

    fn set_meta(&self, key: &str, value: &str) -> Result<(), SyncError> {
        self.store
            .set_meta_if(key, value, &self.guard)
            .map_err(guarded)
    }

    fn cursor(&self) -> Result<i64, SyncError> {
        Ok(self
            .store
            .meta(meta_keys::LAST_SEQ)?
            .and_then(|v| v.parse::<i64>().ok())
            .filter(|v| *v >= 0)
            .unwrap_or(0))
    }

    /// Pages through the feed from the cursor (see [`sync`]). Runs again
    /// before a push round after a `behind` answer.
    fn pull(&mut self) -> Result<(), SyncError> {
        let page_size = self
            .options
            .page_size
            .clamp(1, crate::wire::MAX_CHANGES_LIMIT);
        let mut since = self.cursor()?;
        self.wire_ids = None;
        loop {
            self.check_cancel()?;
            let page = self.sealed.changes(since, page_size)?;
            if page.key_id != self.sealed.key_id() {
                return Err(SyncError::WrongKey);
            }
            match &self.epoch {
                None => {
                    self.epoch = Some(page.epoch.clone());
                    let saved = self.store.meta(meta_keys::EPOCH)?;
                    let restored =
                        saved.as_deref().is_some_and(|e| e != page.epoch) || page.head_seq < since;
                    if restored {
                        self.set_meta(meta_keys::LAST_SEQ, "0")?;
                        self.set_meta(meta_keys::EPOCH, REMERGING)?;
                        self.remerging = true;
                        self.present.clear();
                        self.report.remerged = true;
                        if since != 0 {
                            since = 0;
                            continue;
                        }
                    } else if saved.is_none() {
                        // The first sync after binding: adopt the epoch.
                        self.set_meta(meta_keys::EPOCH, &page.epoch)?;
                    }
                }
                Some(epoch) if *epoch != page.epoch => {
                    return Err(SyncError::Api(ApiError::Protocol(
                        "the epoch changed during the sync".into(),
                    )));
                }
                Some(_) => {}
            }
            let mut last = since;
            let mut too_new = false;
            for entry in page.entries {
                let seq = entry.seq();
                match entry {
                    Entry::TooNew { .. } => {
                        too_new = true;
                        break;
                    }
                    Entry::Opened { doc, .. } => {
                        if self.remerging {
                            self.present.insert(doc.id.clone());
                        }
                        self.apply(&doc)?;
                    }
                    Entry::Undecryptable { wire, .. } => self.undecryptable(&wire)?,
                }
                last = seq;
            }
            if last > since {
                self.set_meta(meta_keys::LAST_SEQ, &last.to_string())?;
                since = last;
            }
            if too_new {
                // Stop before it, keeping the cursor; pushing continues.
                self.report.too_new = true;
                return Ok(());
            }
            if !page.more {
                break;
            }
        }
        if self.remerging {
            self.store
                .mark_missing_unsynced(&self.present, Some(&self.guard))
                .map_err(guarded)?;
            let epoch = self.epoch.clone().unwrap_or_default();
            self.set_meta(meta_keys::EPOCH, &epoch)?;
            self.remerging = false;
            self.present.clear();
        }
        Ok(())
    }

    /// Applies one opened server document, fetching its file bytes when the
    /// store lacks them.
    fn apply(&mut self, doc: &SyncDoc) -> Result<(), SyncError> {
        let attachment = if !doc.deleted && is_file_id(&doc.id) {
            self.fetch_blob(doc)?
        } else {
            None
        };
        // During a re-merge the Mac can't tell whether a server version
        // descends from the one it synced (the server lost history), so a
        // remote version that replaces different live content is flagged.
        let replaced = if self.remerging {
            self.store
                .get(&doc.id)?
                .map(|d| d.rev)
                .filter(|rev| content_hash(rev) != content_hash(&doc.rev))
        } else {
            None
        };
        let outcome = match self.store.apply_remote(doc, attachment, Some(&self.guard)) {
            Ok(outcome) => outcome,
            Err(StoreError::Invalid(reason)) => {
                self.report.invalid.push((doc.id.clone(), reason));
                return Ok(());
            }
            Err(err) => return Err(guarded(err)),
        };
        let report = &mut self.report;
        match outcome {
            ApplyOutcome::Applied => {
                report.pulled += 1;
                if replaced.is_some() {
                    // The local version is in history (`apply_remote`
                    // snapshots what it replaces).
                    report.conflicts_remote += 1;
                    report.note_conflict(&doc.id);
                }
            }
            ApplyOutcome::Unchanged => {}
            ApplyOutcome::RemoteWon => {
                report.pulled += 1;
                report.conflicts_remote += 1;
                report.note_conflict(&doc.id);
            }
            ApplyOutcome::LocalWon { .. } => {
                report.conflicts_local += 1;
                report.note_conflict(&doc.id);
            }
            ApplyOutcome::Stale { conflict, .. } => {
                if !report.stale.contains(&doc.id) {
                    report.stale.push(doc.id.clone());
                }
                if conflict {
                    // A live version below the floor that this Mac never
                    // held: kept in history and flagged, never silent.
                    report.conflicts_local += 1;
                    report.note_conflict(&doc.id);
                }
            }
        }
        Ok(())
    }

    /// The opened bytes of a live file document, when they match its
    /// `md5digest`; otherwise they count as missing.
    fn fetch_blob(&mut self, doc: &SyncDoc) -> Result<Option<Attachment>, SyncError> {
        let md5 = doc
            .fields
            .get("md5digest")
            .and_then(Value::as_str)
            .map(str::to_ascii_lowercase);
        if let Some(md5) = &md5
            && self.store.has_blob(&doc.id, md5)?
        {
            return Ok(None);
        }
        self.check_cancel()?;
        self.fetched.insert(doc.id.clone());
        match self.sealed.get_file(&doc.id)? {
            FileBytes::Opened(bytes) if md5.as_deref() == Some(md5_hex(&bytes).as_str()) => {
                self.report.files_down += 1;
                Ok(Some(Attachment::new(bytes)))
            }
            FileBytes::Opened(_) | FileBytes::Missing | FileBytes::Unreadable => {
                self.report.files_missing += 1;
                Ok(None)
            }
        }
    }

    /// Fetches again the bytes of up to [`SyncOptions::refetch_files`] live
    /// files this Mac lacks (review-sync 5), going on in ID order from where
    /// the last sync stopped and skipping files this sync already asked for.
    /// Bytes are kept only when they open, match the document's `md5digest`
    /// and the document still has the revision they were fetched for
    /// (`Store::fill_blob`, guarded).
    ///
    /// It never stops the sync, nor the push after it (review-verify 4): a
    /// file the server doesn't have, whose bytes don't match, or whose
    /// request fails counts in `files_missing` and waits a day
    /// ([`meta_keys::REFETCH_MISSED`]); a failed request also ends this
    /// sync's refetching. Only a 429 `session-account` stops the sync, as
    /// on every route ([`SyncError::BackOff`]).
    fn refetch(&mut self) -> Result<(), SyncError> {
        let limit = self.options.refetch_files;
        if limit == 0 {
            return Ok(());
        }
        let now = now_ms();
        let saved = self.store.meta(meta_keys::REFETCH_MISSED)?;
        let mut missed = missed_files(saved.as_deref(), now);
        let after = self
            .store
            .meta(meta_keys::REFETCH_AFTER)?
            .unwrap_or_default();
        // Enough rows that skipping the ones this sync fetched, and the
        // ones waiting, still leaves `limit`, from where the last sync
        // stopped, then from the start.
        let window = limit + self.fetched.len() + missed.len();
        let mut todo = self.store.files_without_bytes(&after, window)?;
        if todo.len() < window && !after.is_empty() {
            todo.extend(
                self.store
                    .files_without_bytes("", window)?
                    .into_iter()
                    .filter(|(id, ..)| *id <= after),
            );
        }
        todo.retain(|(id, ..)| !self.fetched.contains(id) && !missed.contains_key(id));
        todo.truncate(limit);
        let mut last = None;
        // A 429 `session-account`: the sync stops, once this is saved.
        let mut slow_down = None;
        for (id, rev, md5) in todo {
            self.check_cancel()?;
            self.fetched.insert(id.clone());
            last = Some(id.clone());
            let mut failed = false;
            let found = match self.sealed.get_file(&id) {
                Ok(FileBytes::Opened(bytes)) if md5_hex(&bytes) == md5 => {
                    if self
                        .store
                        .fill_blob(&id, &rev, Attachment::new(bytes), Some(&self.guard))
                        .map_err(guarded)?
                    {
                        self.report.files_down += 1;
                    }
                    true
                }
                Ok(FileBytes::Opened(_) | FileBytes::Missing | FileBytes::Unreadable) => false,
                Err(ApiError::RateLimited { retry_after, limit }) if limit == SESSION_LIMIT => {
                    slow_down = Some(back_off(retry_after));
                    break;
                }
                Err(_) => {
                    // Offline, or the server failed: the rest would most
                    // likely fail too, and each can take minutes.
                    failed = true;
                    false
                }
            };
            if !found {
                self.report.files_missing += 1;
                missed.insert(id, now.saturating_add(REFETCH_EVERY_MS));
            }
            if failed {
                break;
            }
        }
        bound_missed(&mut missed);
        if let Some(last) = last {
            self.set_meta(meta_keys::REFETCH_AFTER, &last)?;
        }
        let value = (!missed.is_empty())
            .then(|| serde_json::to_string(&missed).expect("a map of strings to numbers"));
        if value != saved {
            match &value {
                Some(value) => self.set_meta(meta_keys::REFETCH_MISSED, value)?,
                None => self
                    .store
                    .remove_meta_if(meta_keys::REFETCH_MISSED, &self.guard)
                    .map_err(guarded)?,
            }
        }
        slow_down.map_or(Ok(()), Err)
    }

    /// A feed document that didn't open (M3, security re-check 10).
    fn undecryptable(&mut self, wire: &WireChange) -> Result<(), SyncError> {
        if self.wire_ids.is_none() {
            let map = self
                .store
                .sync_docs()?
                .into_iter()
                .map(|d| (self.sealed.wire_id(&d.id), (d.id, d.base_rev)))
                .collect();
            self.wire_ids = Some(map);
        }
        let Some((local_id, synced)) = self.wire_ids.as_ref().and_then(|m| m.get(&wire.id)) else {
            self.report.skipped.push(wire.id.clone());
            return Ok(());
        };
        let (local_id, synced) = (local_id.clone(), synced.clone());
        if self.remerging {
            // The server has it, broken: it isn't missing.
            self.present.insert(local_id.clone());
        }
        let ours = synced.map(|s| self.sealed.wire_rev(&local_id, &s));
        if ours.as_deref() == Some(wire.rev.as_str())
            && self
                .store
                .bump_revision(&local_id, Some(&self.guard))
                .map_err(guarded)?
                .is_some()
        {
            // Pushed one generation higher over the broken revision, which
            // is `wire_rev(id, synced_rev)`, the base the push sends.
            if !self.report.repaired.contains(&local_id) {
                self.report.repaired.push(local_id);
            }
        } else {
            self.report.skipped.push(wire.id.clone());
        }
        Ok(())
    }

    /// A 429 on a push or upload (other than `uploads-global`): pushing
    /// stops for this sync; the report keeps the latest time and its limit.
    fn pause(&mut self, retry_after: u64, limit: String) {
        let until = retry_at(retry_after);
        if self.report.paused_until.is_none_or(|u| until >= u) {
            self.report.paused_until = Some(until);
            self.report.paused_limit = Some(limit);
        }
        self.stopped = true;
    }

    /// A 429 `uploads-global` on an upload: uploads wait until then, in a
    /// guarded meta write; everything else goes on.
    fn hold_uploads(&mut self, retry_after: u64) -> Result<(), SyncError> {
        let until = retry_at(retry_after);
        let until = self.uploads_until.map_or(until, |u| u.max(until));
        self.set_meta(meta_keys::UPLOADS_UNTIL, &until.to_string())?;
        self.uploads_until = Some(until);
        self.report.uploads_held_until = Some(until);
        Ok(())
    }

    /// Reads the quota entries and the upload hold this push starts with,
    /// removing an upload hold whose time has come. An entry or hold saved
    /// more than [`SAVED_AHEAD_MAX_MS`] ahead of now goes too
    /// (review-verify 5): it came from a clock that ran ahead, and would
    /// hold until then, which only a re-bind cleared. A dropped quota entry
    /// costs at most one more refused request.
    fn load_holds(&mut self) -> Result<(), SyncError> {
        let now = now_ms();
        for family in [Family::Docs, Family::Files] {
            let Some(value) = self.store.meta(family.key())? else {
                continue;
            };
            let entry = QuotaEntry::parse(&value);
            match entry {
                Some(entry) if not_too_far_ahead(entry.at_ms, now) => {
                    self.holds[family as usize].entry = Some(entry);
                }
                Some(_) => self
                    .store
                    .remove_meta_if(family.key(), &self.guard)
                    .map_err(guarded)?,
                None => {}
            }
        }
        if let Some(value) = self.store.meta(meta_keys::UPLOADS_UNTIL)? {
            match value.parse::<i64>() {
                Ok(until) if until > now && not_too_far_ahead(until, now) => {
                    self.uploads_until = Some(until);
                    self.report.uploads_held_until = Some(until);
                }
                _ => self
                    .store
                    .remove_meta_if(meta_keys::UPLOADS_UNTIL, &self.guard)
                    .map_err(guarded)?,
            }
        }
        Ok(())
    }

    /// The room left in `family` once usage dropped, from one
    /// `GET /v2/account` a sync; `None` holds per the entry's level. A
    /// failed read holds too: the entry stays.
    fn room(&mut self, family: Family) -> Result<Option<Room>, SyncError> {
        let i = family as usize;
        if let Some(room) = self.holds[i].room {
            return Ok(room);
        }
        let Some(entry) = self.holds[i].entry else {
            return Ok(None);
        };
        if self.usage.is_none() {
            self.check_cancel()?;
            self.usage = Some(crate::account::usage(self.backend, self.session).ok());
        }
        let room = self
            .usage
            .flatten()
            .and_then(|usage| entry.room(family, &usage, now_ms()));
        self.holds[i].room = Some(room);
        Ok(room)
    }

    /// Whether `family`'s quota entry holds `doc` back this sync. Once usage
    /// dropped, what fits in the room goes, and its cost (`count` new items
    /// and `bytes`) is taken from the room once a sync.
    fn quota_holds(
        &mut self,
        family: Family,
        doc: &SyncDoc,
        class: Class,
        count: u64,
        bytes: u64,
    ) -> Result<bool, SyncError> {
        let i = family as usize;
        let Some(entry) = self.holds[i].entry else {
            return Ok(false);
        };
        if !class.held_at(entry.level) || self.holds[i].charged.contains(&doc.id) {
            return Ok(false);
        }
        if let Some(room) = self.room(family)? {
            // A tombstone of a document the server has never grows it.
            let (count, bytes) = if class == Class::Tombstone {
                (0, 0)
            } else {
                (count, bytes)
            };
            if room.count >= count && room.bytes >= bytes {
                self.holds[i].room = Some(Some(Room {
                    count: room.count - count,
                    bytes: room.bytes - bytes,
                }));
                self.holds[i].charged.insert(doc.id.clone());
                return Ok(false);
            }
        }
        self.note_held(doc);
        Ok(true)
    }

    /// A pending file can outlive the document hold that kept it from the
    /// push. Once that hold's room opens, its own quota may be the next
    /// refusal. When this sync already knows the account's usage, record
    /// that hold before spending an upload, so an unchanged follow-up sync
    /// does not send a request the server will refuse. A later sync can
    /// use the entry's room normally.
    fn precheck_upload_quota(
        &mut self,
        doc: &SyncDoc,
        class: Class,
        count: u64,
        bytes: u64,
    ) -> Result<bool, SyncError> {
        let i = Family::Files as usize;
        if self.holds[i].entry.is_some() {
            return Ok(false);
        }
        let Some(Some(usage)) = self.usage else {
            return Ok(false);
        };
        let (kind, used, limit) =
            if count > 0 && usage.files.saturating_add(count) > usage.quota.files {
                (QuotaKind::Files, usage.files, usage.quota.files)
            } else if usage.file_bytes.saturating_add(bytes) > usage.quota.file_bytes {
                (
                    QuotaKind::FileBytes,
                    usage.file_bytes,
                    usage.quota.file_bytes,
                )
            } else {
                return Ok(false);
            };
        self.holds[i].entry = Some(QuotaEntry {
            kind,
            used,
            limit,
            at_ms: now_ms(),
            level: class.level(),
        });
        let held = self.quota_holds(Family::Files, doc, class, count, bytes)?;
        if held {
            self.report.quota = Some(kind);
        }
        Ok(held)
    }

    /// `doc` stays pending this sync for a quota or an upload hold. Neither
    /// family's entry can go while something it may concern waits: a held
    /// document, or a live file whose upload was never tried.
    fn note_held(&mut self, doc: &SyncDoc) {
        self.holds[Family::Docs as usize].held_back = true;
        if !doc.deleted && is_file_id(&doc.id) {
            self.holds[Family::Files as usize].held_back = true;
        }
    }

    /// A 413 on a request of `family` whose worst document is `worst`:
    /// remembers it (guarded) at the level that holds that class, and stops
    /// taking from the room for the rest of this sync. Returns the level.
    fn refused(
        &mut self,
        family: Family,
        kind: QuotaKind,
        used: u64,
        limit: u64,
        worst: Class,
    ) -> Result<Level, SyncError> {
        let i = family as usize;
        let level = self.holds[i]
            .entry
            .map_or(worst.level(), |e| e.level.max(worst.level()));
        let entry = QuotaEntry {
            kind,
            used,
            limit,
            at_ms: now_ms(),
            level,
        };
        self.set_meta(family.key(), &entry.encode())?;
        let hold = &mut self.holds[i];
        hold.entry = Some(entry);
        hold.room = Some(None);
        hold.refused = true;
        hold.held_back = true;
        self.report.quota = Some(kind);
        Ok(level)
    }

    /// After the push: a family's entry goes once a sync that ran to the
    /// end held nothing of it back and got no 413 for it; one that stays
    /// keeps `report.quota` set.
    fn settle_holds(&mut self) -> Result<(), SyncError> {
        for family in [Family::Docs, Family::Files] {
            let hold = &self.holds[family as usize];
            let Some(entry) = hold.entry else { continue };
            let unchecked = family == Family::Files && self.uploads_until.is_some();
            if !self.stopped && !hold.held_back && !hold.refused && !unchecked {
                self.store
                    .remove_meta_if(family.key(), &self.guard)
                    .map_err(guarded)?;
                self.holds[family as usize].entry = None;
            } else if self.report.quota.is_none() {
                self.report.quota = Some(entry.kind);
            }
        }
        Ok(())
    }

    /// Sends pending documents (see [`sync`]).
    fn push(&mut self) -> Result<(), SyncError> {
        self.load_holds()?;
        // Documents this sync can't push: their server copy is too new or
        // unreadable.
        let mut held: HashSet<String> = HashSet::new();
        let max_docs = self.options.batch_docs.clamp(1, crate::wire::MAX_PUSH_DOCS);
        let max_bytes = self
            .options
            .batch_bytes
            .clamp(1, crate::wire::MAX_PUSH_BODY - 1024);
        for _ in 0..self.options.max_rounds.max(1) {
            self.check_cancel()?;
            let mut docs = Vec::new();
            let mut sizes = Vec::new();
            self.report.too_large.clear();
            for doc in self.store.pending_push(usize::MAX)? {
                if held.contains(&doc.id) {
                    continue;
                }
                let fields = serde_json::to_string(&doc.fields).map_or(usize::MAX, |s| s.len());
                if !doc.deleted && fields > crate::wire::MAX_FIELDS_JSON {
                    self.report.too_large.push(doc.id);
                    continue;
                }
                let size = wire_size(&doc, fields);
                let class = Class::of(&doc);
                // Every live document is charged one potential slot: an edit
                // based on a revision the server has since tombstoned
                // revives it, so it can grow `docs` just like a new one.
                let count = u64::from(class != Class::Tombstone);
                let bytes = size as u64 + DOC_ROW_COST;
                if self.quota_holds(Family::Docs, &doc, class, count, bytes)? {
                    continue;
                }
                sizes.push(size);
                docs.push(doc);
            }
            if docs.is_empty() {
                break;
            }
            let mut again = false;
            let mut need_pull = false;
            for range in batches(&sizes, max_docs, max_bytes) {
                self.check_cancel()?;
                let batch = self.upload_files(docs[range].to_vec())?;
                if self.stopped {
                    break;
                }
                if batch.is_empty() {
                    continue;
                }
                let Some((batch, results)) = self.push_batch(batch)? else {
                    if self.stopped {
                        break;
                    }
                    continue;
                };
                for (doc, result) in batch.iter().zip(results) {
                    match result {
                        Pushed::Ok => {
                            self.store
                                .mark_synced(&doc.id, &doc.rev, Some(&self.guard))
                                .map_err(guarded)?;
                            self.report.pushed += 1;
                        }
                        Pushed::Conflict(current) => {
                            again = true;
                            self.apply(&current)?;
                        }
                        Pushed::Lost => {
                            // The server lost it (M1): push it again with no base.
                            self.store
                                .mark_unsynced(&doc.id, Some(&self.guard))
                                .map_err(guarded)?;
                            again = true;
                        }
                        Pushed::Behind => {
                            need_pull = true;
                            again = true;
                        }
                        Pushed::Unreadable {
                            newer_format: true, ..
                        } => {
                            self.report.too_new = true;
                            held.insert(doc.id.clone());
                        }
                        Pushed::Unreadable {
                            wire_rev,
                            newer_format: false,
                        } => {
                            let ours = doc
                                .base_rev
                                .as_deref()
                                .map(|s| self.sealed.wire_rev(&doc.id, s));
                            if ours.as_deref() == Some(wire_rev.as_str())
                                && self
                                    .store
                                    .bump_revision(&doc.id, Some(&self.guard))
                                    .map_err(guarded)?
                                    .is_some()
                            {
                                if !self.report.repaired.contains(&doc.id) {
                                    self.report.repaired.push(doc.id.clone());
                                }
                                again = true;
                            } else {
                                let wire_id = self.sealed.wire_id(&doc.id);
                                if !self.report.skipped.contains(&wire_id) {
                                    self.report.skipped.push(wire_id);
                                }
                                held.insert(doc.id.clone());
                            }
                        }
                    }
                }
            }
            if self.stopped || !again {
                break;
            }
            if need_pull {
                self.pull()?;
            }
        }
        self.settle_holds()
    }

    /// Whether `doc`, picked for this round, still goes once a 413 earlier
    /// in this sync raised `family`'s level: what the level holds stays,
    /// even if it was taken from the room.
    fn still_goes(&mut self, family: Family, doc: &SyncDoc) -> bool {
        let hold = &self.holds[family as usize];
        match hold.entry {
            Some(entry) if hold.refused && Class::of(doc).held_at(entry.level) => {
                self.note_held(doc);
                false
            }
            _ => true,
        }
    }

    /// Pushes one batch, handling a pause and a quota: after a 413 the
    /// batch is retried at once without what the new level holds, until
    /// nothing is left. `None` when nothing was pushed.
    #[allow(clippy::type_complexity)]
    fn push_batch(
        &mut self,
        mut batch: Vec<SyncDoc>,
    ) -> Result<Option<(Vec<SyncDoc>, Vec<Pushed>)>, SyncError> {
        loop {
            if batch.is_empty() {
                return Ok(None);
            }
            self.check_cancel()?;
            // Right before the request: an upload just before it writes
            // nothing locally, so no guarded write would have noticed a
            // re-bind. The guarded `mark_sent` checks the binding again in
            // its transaction, and records what is in flight so that, if the
            // answer is lost, its echo is recognised as ours.
            self.check_bound()?;
            let sent: Vec<(&str, &str)> = batch
                .iter()
                .map(|d| (d.id.as_str(), d.rev.as_str()))
                .collect();
            self.store
                .mark_sent(&sent, Some(&self.guard))
                .map_err(guarded)?;
            match self.sealed.push(&batch) {
                Ok(results) => return Ok(Some((batch, results))),
                Err(ApiError::QuotaExceeded { quota, used, limit }) => {
                    let worst = batch.iter().map(Class::of).max().unwrap_or(Class::New);
                    let level = self.refused(Family::Docs, quota, used, limit, worst)?;
                    let (out, kept): (Vec<_>, Vec<_>) =
                        batch.into_iter().partition(|d| Class::of(d).held_at(level));
                    for doc in &out {
                        self.note_held(doc);
                    }
                    batch = kept;
                }
                Err(ApiError::RateLimited { retry_after, limit }) if limit == SESSION_LIMIT => {
                    return Err(back_off(retry_after));
                }
                Err(ApiError::RateLimited { retry_after, limit }) => {
                    self.pause(retry_after, limit);
                    return Ok(None);
                }
                Err(err) => return Err(err.into()),
            }
        }
    }

    /// Uploads the bytes of the live files in `batch`, so they are on the
    /// server before any device can see the documents. Documents a 413
    /// earlier in this sync now holds leave the batch first. A file whose
    /// upload is held (a files quota entry, or a 429 `uploads-global`) or
    /// hits a quota leaves the batch and stays pending; any other 429 stops
    /// pushing.
    fn upload_files(&mut self, batch: Vec<SyncDoc>) -> Result<Vec<SyncDoc>, SyncError> {
        let mut kept = Vec::with_capacity(batch.len());
        for doc in batch {
            if !self.still_goes(Family::Docs, &doc) {
                continue;
            }
            if doc.deleted || !is_file_id(&doc.id) {
                kept.push(doc);
                continue;
            }
            let Some(blob) = self.store.stored_attachment(&doc.id)? else {
                self.report.files_missing += 1;
                kept.push(doc);
                continue;
            };
            if self.uploads_until.is_some() {
                self.note_held(&doc);
                continue;
            }
            let class = if doc.base_rev.is_none() {
                Class::New
            } else {
                Class::Edit
            };
            let bytes = blob.data.len() as u64 + FILE_ROW_COST;
            let count = u64::from(class != Class::Tombstone);
            // A document hold can make room for this upload before its own
            // family has seen a refusal. Use the usage already read for the
            // document room to avoid spending a write on a known refusal.
            if self.precheck_upload_quota(&doc, class, count, bytes)? {
                continue;
            }
            // As in `push`: a live file edit can revive a server tombstone.
            if self.quota_holds(Family::Files, &doc, class, count, bytes)?
                || !self.still_goes(Family::Files, &doc)
            {
                continue;
            }
            self.check_cancel()?;
            self.check_bound()?;
            match self.sealed.put_file(&doc.id, &blob.data) {
                Ok(()) => {
                    self.report.files_up += 1;
                    kept.push(doc);
                }
                Err(ApiError::QuotaExceeded { quota, used, limit }) => {
                    self.refused(Family::Files, quota, used, limit, class)?;
                    self.note_held(&doc);
                }
                Err(ApiError::RateLimited { retry_after, limit }) if limit == SESSION_LIMIT => {
                    return Err(back_off(retry_after));
                }
                Err(ApiError::RateLimited { retry_after, limit }) if limit == "uploads-global" => {
                    self.hold_uploads(retry_after)?;
                    self.note_held(&doc);
                }
                Err(ApiError::RateLimited { retry_after, limit }) => {
                    self.pause(retry_after, limit);
                    return Ok(Vec::new());
                }
                Err(err) => return Err(err.into()),
            }
        }
        Ok(kept)
    }
}

/// What the sidebar and Settings › Account show for a sync that failed or
/// finished with problems (docs/ACCOUNTS.md › Problems).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SyncProblem {
    /// 401: "Signed out". Stops.
    SignedOut(SignOutReason),
    /// 403: "Account turned off". Stops; Sync now retries.
    AccountDisabled,
    /// A 429 on a push or upload: "Sync paused" until `until` (ms).
    Paused { until: i64 },
    /// A 429 `writes-global`: the server's own daily cap. "Server busy":
    /// "The server has taken all the changes it can for today. Sync resumes
    /// at *time*." Waits until `until` (ms).
    ServerBusy { until: i64 },
    /// A 429 `session-account` ([`SyncError::BackOff`]): too many requests
    /// in a minute. A short wait, not a pause: "Syncing shortly". Waits
    /// until `until` (ms), about `retry_after` seconds.
    BackOff { until: i64 },
    /// A 413 quota (never `Server`): "Account full". Keeps running.
    QuotaExceeded(QuotaKind),
    /// A 413 on a server-wide cap: "Server full". Keeps running.
    ServerFull,
    /// Feed documents skipped unrepaired: "Some items skipped". Keeps running.
    Skipped { count: usize },
    /// A newer format in the feed: "Update Kanso". Stops; Sync now retries.
    UpdateKanso,
    /// The key ID differs: "Keys don't match". Stops.
    WrongKey,
    /// A 404 on a v2 route: "Server needs an update". Stops; Sync now retries.
    ServerOutdated,
    /// No answer: "Offline". Keeps running.
    Offline,
    /// Anything else: "Couldn't sync". Keeps running.
    Other,
}

/// What the sync schedule does after a problem (the table's last column).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Schedule {
    KeepsRunning,
    /// Waits until this time (ms), then runs again.
    WaitsUntil(i64),
    /// Stops; "Sync now" tries again.
    StopsUntilSyncNow,
    /// Stops until the user signs in again (or out and in).
    Stops,
}

impl SyncProblem {
    /// Whether the schedule keeps running after this problem.
    pub fn schedule(&self) -> Schedule {
        match self {
            SyncProblem::SignedOut(_) | SyncProblem::WrongKey => Schedule::Stops,
            SyncProblem::AccountDisabled
            | SyncProblem::UpdateKanso
            | SyncProblem::ServerOutdated => Schedule::StopsUntilSyncNow,
            SyncProblem::Paused { until }
            | SyncProblem::ServerBusy { until }
            | SyncProblem::BackOff { until } => Schedule::WaitsUntil(*until),
            SyncProblem::QuotaExceeded(_)
            | SyncProblem::ServerFull
            | SyncProblem::Skipped { .. }
            | SyncProblem::Offline
            | SyncProblem::Other => Schedule::KeepsRunning,
        }
    }
}

/// The problem a failed sync shows; `None` for [`SyncError::Cancelled`].
/// Pure; implemented in step 1.
pub fn problem(err: &SyncError) -> Option<SyncProblem> {
    Some(match err {
        SyncError::Cancelled => return None,
        SyncError::WrongKey => SyncProblem::WrongKey,
        SyncError::BackOff { until } => SyncProblem::BackOff { until: *until },
        SyncError::Api(api) => match api {
            ApiError::Unauthorized { reason } => SyncProblem::SignedOut(*reason),
            ApiError::AccountDisabled => SyncProblem::AccountDisabled,
            ApiError::NotFound => SyncProblem::ServerOutdated,
            ApiError::QuotaExceeded {
                quota: QuotaKind::Server,
                ..
            } => SyncProblem::ServerFull,
            ApiError::QuotaExceeded { quota, .. } => SyncProblem::QuotaExceeded(*quota),
            ApiError::Transport(_) => SyncProblem::Offline,
            _ => SyncProblem::Other,
        },
        SyncError::Store(_) => SyncProblem::Other,
    })
}

/// The problem a finished sync shows, in this order of precedence:
/// `UpdateKanso`, `Paused` or `ServerBusy` (from `paused_until`:
/// `ServerBusy` when `paused_limit` ends with `-global`), `ServerFull`,
/// `QuotaExceeded`, `Skipped`. Rebased stale documents, repairs and an
/// upload hold (`uploads_held_until`) aren't problems. Pure.
pub fn report_problem(report: &SyncReport) -> Option<SyncProblem> {
    if report.too_new {
        return Some(SyncProblem::UpdateKanso);
    }
    if let Some(until) = report.paused_until {
        let global = report
            .paused_limit
            .as_deref()
            .is_some_and(|limit| limit.ends_with("-global"));
        return Some(if global {
            SyncProblem::ServerBusy { until }
        } else {
            SyncProblem::Paused { until }
        });
    }
    match report.quota {
        Some(QuotaKind::Server) => return Some(SyncProblem::ServerFull),
        Some(kind) => return Some(SyncProblem::QuotaExceeded(kind)),
        None => {}
    }
    if !report.skipped.is_empty() {
        return Some(SyncProblem::Skipped {
            count: report.skipped.len(),
        });
    }
    None
}

/// Splits items of the given sizes into consecutive batches of at most
/// `max_docs` items and `max_bytes` bytes, filling each batch greedily. An
/// item larger than `max_bytes` goes alone.
pub fn batches(sizes: &[usize], max_docs: usize, max_bytes: usize) -> Vec<Range<usize>> {
    let max_docs = max_docs.max(1);
    let mut out = Vec::new();
    let mut start = 0;
    let mut bytes = 0usize;
    for (i, &size) in sizes.iter().enumerate() {
        let len = i - start;
        if len > 0 && (len >= max_docs || bytes.saturating_add(size) > max_bytes) {
            out.push(start..i);
            start = i;
            bytes = 0;
        }
        bytes = bytes.saturating_add(size);
    }
    if start < sizes.len() {
        out.push(start..sizes.len());
    }
    out
}

#[cfg(test)]
mod problem_tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn errors_map_to_problems() {
        assert_eq!(problem(&SyncError::Cancelled), None);
        assert_eq!(problem(&SyncError::WrongKey), Some(SyncProblem::WrongKey));
        assert_eq!(
            problem(&SyncError::Api(ApiError::Unauthorized {
                reason: SignOutReason::Evicted
            })),
            Some(SyncProblem::SignedOut(SignOutReason::Evicted))
        );
        assert_eq!(
            problem(&SyncError::Api(ApiError::AccountDisabled)),
            Some(SyncProblem::AccountDisabled)
        );
        assert_eq!(
            problem(&SyncError::Api(ApiError::NotFound)),
            Some(SyncProblem::ServerOutdated)
        );
        assert_eq!(
            problem(&SyncError::Api(ApiError::Transport("dns".into()))),
            Some(SyncProblem::Offline)
        );
        assert_eq!(
            problem(&SyncError::Api(ApiError::Internal)),
            Some(SyncProblem::Other)
        );
        assert_eq!(
            problem(&SyncError::Store(StoreError::GuardFailed)),
            Some(SyncProblem::Other)
        );
    }

    #[test]
    fn schedules() {
        assert_eq!(
            SyncProblem::SignedOut(SignOutReason::Expired).schedule(),
            Schedule::Stops
        );
        assert_eq!(SyncProblem::WrongKey.schedule(), Schedule::Stops);
        assert_eq!(
            SyncProblem::UpdateKanso.schedule(),
            Schedule::StopsUntilSyncNow
        );
        assert_eq!(
            SyncProblem::Paused { until: 7 }.schedule(),
            Schedule::WaitsUntil(7)
        );
        assert_eq!(
            SyncProblem::ServerBusy { until: 9 }.schedule(),
            Schedule::WaitsUntil(9)
        );
        assert_eq!(
            SyncProblem::BackOff { until: 11 }.schedule(),
            Schedule::WaitsUntil(11)
        );
        assert_eq!(
            problem(&SyncError::BackOff { until: 11 }),
            Some(SyncProblem::BackOff { until: 11 })
        );
        assert_eq!(SyncProblem::Offline.schedule(), Schedule::KeepsRunning);
        assert_eq!(report_problem(&SyncReport::default()), None);
    }

    #[test]
    fn a_global_limit_is_server_busy() {
        let report = |limit: &str| SyncReport {
            paused_until: Some(5),
            paused_limit: Some(limit.to_owned()),
            ..SyncReport::default()
        };
        assert_eq!(
            report_problem(&report("writes-global")),
            Some(SyncProblem::ServerBusy { until: 5 })
        );
        for limit in ["writes-account", "uploads-account"] {
            assert_eq!(
                report_problem(&report(limit)),
                Some(SyncProblem::Paused { until: 5 })
            );
        }
        let held = SyncReport {
            uploads_held_until: Some(5),
            ..SyncReport::default()
        };
        assert_eq!(
            report_problem(&held),
            None,
            "an upload hold isn't a problem"
        );
    }

    #[test]
    fn quota_entries_round_trip() {
        for kind in QuotaKind::ALL {
            for level in [Level::New, Level::Edits, Level::All] {
                let entry = QuotaEntry {
                    kind,
                    used: 10,
                    limit: 12,
                    at_ms: 1_700_000_000_000,
                    level,
                };
                assert_eq!(QuotaEntry::parse(&entry.encode()), Some(entry));
            }
        }
        for bad in [
            "",
            "docs 1 2 3",
            "docs 1 2 3 most",
            "disk 1 2 3 new",
            "docs 1 2 3 new x",
        ] {
            assert_eq!(QuotaEntry::parse(bad), None, "{bad:?}");
        }
    }

    fn gen_report() -> impl Strategy<Value = SyncReport> {
        (
            any::<bool>(),
            prop::option::of(0i64..1_000),
            prop::option::of(prop::sample::select(vec![
                "writes-account",
                "uploads-account",
                "writes-global",
            ])),
            prop::option::of(prop::sample::select(QuotaKind::ALL.to_vec())),
            prop::option::of(0i64..1_000),
            0..3usize,
            0..3usize,
            0..3usize,
        )
            .prop_map(
                |(
                    too_new,
                    paused_until,
                    limit,
                    quota,
                    uploads_held_until,
                    skipped,
                    stale,
                    repaired,
                )| {
                    SyncReport {
                        too_new,
                        paused_until,
                        paused_limit: paused_until.and(limit.map(str::to_owned)),
                        quota,
                        uploads_held_until,
                        skipped: (0..skipped).map(|i| format!("note:s{i}")).collect(),
                        stale: (0..stale).map(|i| format!("note:t{i}")).collect(),
                        repaired: (0..repaired).map(|i| format!("note:r{i}")).collect(),
                        ..SyncReport::default()
                    }
                },
            )
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Model-based: the reported problem is the first that applies, in
        /// the order UpdateKanso, Paused or ServerBusy, ServerFull,
        /// QuotaExceeded, Skipped; stale and repaired documents and an
        /// upload hold alone are never a problem.
        #[test]
        fn prop_report_problem_follows_the_precedence(report in gen_report()) {
            let mut model = Vec::new();
            if report.too_new { model.push(SyncProblem::UpdateKanso); }
            if let Some(until) = report.paused_until {
                model.push(if report.paused_limit.as_deref() == Some("writes-global") {
                    SyncProblem::ServerBusy { until }
                } else {
                    SyncProblem::Paused { until }
                });
            }
            if report.quota == Some(QuotaKind::Server) { model.push(SyncProblem::ServerFull); }
            if let Some(kind) = report.quota.filter(|k| *k != QuotaKind::Server) { model.push(SyncProblem::QuotaExceeded(kind)); }
            if !report.skipped.is_empty() { model.push(SyncProblem::Skipped { count: report.skipped.len() }); }
            prop_assert_eq!(report_problem(&report), model.first().cloned());
            if let Some(SyncProblem::QuotaExceeded(kind)) = report_problem(&report) {
                prop_assert_ne!(kind, QuotaKind::Server);
            }
        }
    }
}

#[cfg(test)]
mod refetch_tests {
    use super::*;
    use proptest::prelude::*;

    const NOW: i64 = 1_800_000_000_000;

    /// Due waits, waits too far ahead and a damaged value are dropped.
    #[test]
    fn saved_waits_are_checked_on_load() {
        let saved = serde_json::json!({
            "file:due": NOW,
            "file:waiting": NOW + 1,
            "file:tomorrow": NOW + REFETCH_EVERY_MS,
            "file:far": NOW + SAVED_AHEAD_MAX_MS + 1,
        })
        .to_string();
        let missed = missed_files(Some(&saved), NOW);
        assert_eq!(
            missed.keys().map(String::as_str).collect::<Vec<_>>(),
            ["file:tomorrow", "file:waiting"]
        );
        assert!(missed_files(Some("not json"), NOW).is_empty());
        assert!(missed_files(None, NOW).is_empty());
    }

    /// A 429's wait is the server's, up to a day (review-verify 5).
    #[test]
    fn retry_after_is_bounded_by_a_day() {
        let before = now_ms();
        let at = retry_at(60);
        assert!(at >= before + 60_000 && at <= now_ms() + 60_000);
        assert!(retry_at(u64::MAX) <= now_ms() + 24 * 60 * 60 * 1000);
        assert!(not_too_far_ahead(NOW + SAVED_AHEAD_MAX_MS, NOW));
        assert!(!not_too_far_ahead(NOW + SAVED_AHEAD_MAX_MS + 1, NOW));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Invariant: a 429's wait ends no sooner than now, never more than
        /// a day ahead, and is the server's own when that is under a day;
        /// and a time it gives is never dropped as too far ahead.
        #[test]
        fn prop_retry_after_waits_at_most_a_day(retry_after in prop_oneof![0u64..200_000, any::<u64>()]) {
            let before = now_ms();
            let at = retry_at(retry_after);
            let after = now_ms();
            prop_assert!(at >= before);
            prop_assert!(at <= after + RETRY_AFTER_MAX_S as i64 * 1000);
            if retry_after <= RETRY_AFTER_MAX_S {
                let wait = retry_after as i64 * 1000;
                prop_assert!(at >= before + wait && at <= after + wait);
            }
            prop_assert!(not_too_far_ahead(at, after));
        }

        /// Invariant: whatever was saved, the files still waiting are due
        /// after now and at most a day and an hour ahead, there are at
        /// most `REFETCH_MISSED_MAX` of them, and bounding keeps those due
        /// latest.
        #[test]
        fn prop_waits_stay_bounded(
            saved in prop::collection::btree_map("file:[a-z0-9]{1,6}", -2 * REFETCH_EVERY_MS..3 * REFETCH_EVERY_MS, 0..700),
        ) {
            let value = serde_json::to_string(
                &saved.iter().map(|(id, at)| (id.clone(), NOW + at)).collect::<BTreeMap<_, _>>(),
            )
            .unwrap();
            let mut missed = missed_files(Some(&value), NOW);
            let loaded = missed.clone();
            bound_missed(&mut missed);
            prop_assert!(missed.len() <= REFETCH_MISSED_MAX);
            prop_assert_eq!(missed.len(), loaded.len().min(REFETCH_MISSED_MAX));
            for at in missed.values() {
                prop_assert!(*at > NOW && *at <= NOW + SAVED_AHEAD_MAX_MS);
            }
            let dropped = loaded.iter().filter(|(id, _)| !missed.contains_key(*id));
            let kept_soonest = missed.values().min().copied().unwrap_or(i64::MAX);
            for (_, at) in dropped {
                prop_assert!(*at <= kept_soonest);
            }
        }
    }
}
