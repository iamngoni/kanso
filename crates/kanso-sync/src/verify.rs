//! `kanso-sync verify` and `estimate` (docs/ACCOUNTS.md › Command line).
//! The types were frozen by step 1; the legacy manifest hash must match the
//! admin script's byte for byte (security re-check 6).

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use kanso_store::{Store, StoreError, SyncDoc, SyncGuard};
use serde::{Deserialize, Serialize};

use crate::account::Session;
use crate::api::Backend;
use crate::binding::guard_pairs;
use crate::crypto::{SEAL_OVERHEAD, sealed_doc_len};
use crate::engine::SyncError;
use crate::remote::{is_file_id, md5_hex};
use crate::sealed::{Entry, FileBytes, Pushed, Sealed};
use crate::wire::{DOC_ROW_COST, FILE_ROW_COST, MAX_CHANGES_LIMIT};

/// What `verify` found, comparing the whole feed (opened) with the library.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifyReport {
    /// Local documents with `synced_rev == rev` found at that revision with
    /// equal fields, and live files whose bytes match.
    pub matched: usize,
    pub missing: usize,
    /// On the server but not in the library.
    pub extra: usize,
    pub different: usize,
    pub undecryptable: usize,
    /// Local changes not yet synced.
    pub pending: usize,
    /// With `--repair`: undecryptable server copies replaced from this Mac.
    pub repaired: usize,
    /// With `--legacy`: the manifest check.
    pub legacy: Option<LegacyReport>,
}

impl VerifyReport {
    /// Exit 0: nothing missing, different, undecryptable or pending, and the
    /// legacy check (if any) passed.
    pub fn ok(&self) -> bool {
        self.missing == 0
            && self.different == 0
            && self.undecryptable == 0
            && self.pending == 0
            && self.legacy.as_ref().is_none_or(|l| l.ok)
    }
}

/// The `--legacy` result, written as `R.json`: `{"manifest_sha256",
/// "account_id", "ok", "rows", "failed"}`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegacyReport {
    /// [`manifest_sha256`] of the manifest's rows.
    pub manifest_sha256: String,
    pub account_id: String,
    pub ok: bool,
    /// Rows checked.
    pub rows: usize,
    /// IDs of rows that failed, sorted.
    pub failed: Vec<String>,
}

/// One row of `admin legacy-export`'s `manifest.json`: `[id, rev, deleted,
/// md5digest]`, `md5digest` a lowercase hex string for files and null
/// otherwise.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    from = "(String, String, bool, Option<String>)",
    into = "(String, String, bool, Option<String>)"
)]
pub struct LegacyRow {
    pub id: String,
    pub rev: String,
    pub deleted: bool,
    pub md5digest: Option<String>,
}

impl From<(String, String, bool, Option<String>)> for LegacyRow {
    fn from((id, rev, deleted, md5digest): (String, String, bool, Option<String>)) -> Self {
        LegacyRow {
            id,
            rev,
            deleted,
            md5digest,
        }
    }
}

impl From<LegacyRow> for (String, String, bool, Option<String>) {
    fn from(row: LegacyRow) -> Self {
        (row.id, row.rev, row.deleted, row.md5digest)
    }
}

/// `manifest.json`: `{"exported_at", "rows": [[id, rev, deleted, md5digest]]}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegacyManifest {
    pub exported_at: i64,
    pub rows: Vec<LegacyRow>,
}

/// The manifest hash `verify --legacy` writes and `purge-legacy` recomputes
/// from a fresh export (security re-check 6). Rows only, never
/// `exported_at`: the rows sorted by `id` (bytewise), each written as
/// `id TAB rev TAB ("1" | "0") TAB (md5digest | "") LF` in UTF-8, and the
/// SHA-256 of those bytes as 64 lowercase hex digits. Pure; implemented in
/// step 1.
pub fn manifest_sha256(rows: &[LegacyRow]) -> String {
    use sha2::Digest as _;
    let mut sorted: Vec<&LegacyRow> = rows.iter().collect();
    sorted.sort_by(|a, b| a.id.as_bytes().cmp(b.id.as_bytes()));
    let mut canonical = String::new();
    for row in sorted {
        canonical.push_str(&row.id);
        canonical.push('\t');
        canonical.push_str(&row.rev);
        canonical.push('\t');
        canonical.push(if row.deleted { '1' } else { '0' });
        canonical.push('\t');
        canonical.push_str(row.md5digest.as_deref().unwrap_or(""));
        canonical.push('\n');
    }
    sha2::Sha256::digest(canonical.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The charged bytes a library needs on a v2 server (`kanso-sync estimate`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Estimate {
    /// Live documents.
    pub docs: u64,
    /// `length(sealed) + 512` over every document, tombstones included.
    pub doc_bytes: u64,
    /// Live files with bytes.
    pub files: u64,
    /// `size + 41 + 1024` over those files.
    pub file_bytes: u64,
}

/// Pulls the whole feed into memory without applying it, opens every
/// document and live file, and compares them with the library
/// (docs/ACCOUNTS.md › Command line): every local document whose
/// `synced_rev == rev` must be on the server at that revision with equal
/// fields, and every such live file's bytes must open to the local ones.
///
/// With `repair`, each undecryptable server copy of a document this Mac has
/// is replaced from the Mac first: the local version gets a new revision one
/// generation higher (`Store::bump_revision`) and is pushed over the broken
/// revision. A newer format is never replaced. With `legacy`, each row of
/// the `admin legacy-export` manifest is checked too (H4).
///
/// A key ID other than this Mac's is [`SyncError::WrongKey`]; a library not
/// bound to the session's account is [`SyncError::Cancelled`].
pub fn verify(
    store: &Store,
    b: &dyn Backend,
    session: &Session,
    legacy: Option<&Path>,
    repair: bool,
) -> Result<VerifyReport, SyncError> {
    let pairs = guard_pairs(session);
    let guard = SyncGuard { expect: &pairs };
    for (key, value) in &pairs {
        if store.meta(key)?.as_deref() != Some(*value) {
            return Err(SyncError::Cancelled);
        }
    }
    let manifest = legacy.map(read_manifest).transpose()?;
    let sealed = Sealed::new(b.remote(session), session);

    // The whole feed, opened, in memory.
    let mut server: HashMap<String, SyncDoc> = HashMap::new();
    // Wire ID → (wire rev, newer format) of rows that didn't open.
    let mut broken: BTreeMap<String, (String, bool)> = BTreeMap::new();
    let mut since = 0;
    loop {
        let page = sealed.changes(since, MAX_CHANGES_LIMIT)?;
        if page.key_id != sealed.key_id() {
            return Err(SyncError::WrongKey);
        }
        for entry in page.entries {
            match entry {
                Entry::Opened { doc, .. } => {
                    server.insert(doc.id.clone(), doc);
                }
                Entry::Undecryptable { wire, .. } => {
                    broken.insert(wire.id, (wire.rev, false));
                }
                Entry::TooNew { wire, .. } => {
                    broken.insert(wire.id, (wire.rev, true));
                }
            }
        }
        since = page.last_seq;
        if !page.more {
            break;
        }
    }
    let mut local: BTreeMap<String, SyncDoc> = store
        .sync_docs()?
        .into_iter()
        .map(|d| (d.id.clone(), d))
        .collect();
    let by_wire: HashMap<String, String> = local
        .keys()
        .map(|id| (sealed.wire_id(id), id.clone()))
        .collect();
    // Tombstones binding left behind were never on this account: neither
    // pending nor missing.
    let never_on_account: BTreeSet<String> = store
        .sync_states()?
        .into_iter()
        .filter(|s| s.never_on_account())
        .map(|s| s.id)
        .collect();

    let mut report = VerifyReport::default();
    if repair {
        for (wire_id, (wire_rev, newer)) in &broken.clone() {
            let Some(id) = by_wire.get(wire_id) else {
                continue;
            };
            if *newer {
                continue;
            }
            if let Some(doc) = repair_one(store, &sealed, &guard, &local[id], wire_rev)? {
                broken.remove(wire_id);
                server.insert(id.clone(), doc.clone());
                local.insert(id.clone(), doc);
                report.repaired += 1;
            }
        }
    }

    let mut matched = BTreeSet::new();
    for (id, doc) in &local {
        if never_on_account.contains(id) {
            continue;
        }
        if doc.base_rev.as_deref() != Some(doc.rev.as_str()) {
            report.pending += 1;
            continue;
        }
        let Some(theirs) = server.get(id) else {
            if broken.contains_key(&sealed.wire_id(id)) {
                // Counted below as undecryptable.
                continue;
            }
            report.missing += 1;
            continue;
        };
        let same = theirs.rev == doc.rev
            && theirs.deleted == doc.deleted
            && (doc.deleted || theirs.fields == doc.fields);
        if !same {
            report.different += 1;
            continue;
        }
        if !doc.deleted && is_file_id(id) {
            let Some(blob) = store.stored_attachment(id)? else {
                report.missing += 1;
                continue;
            };
            match sealed.get_file(id)? {
                FileBytes::Opened(bytes) if bytes == blob.data => {}
                FileBytes::Opened(_) => {
                    report.different += 1;
                    continue;
                }
                FileBytes::Missing => {
                    report.missing += 1;
                    continue;
                }
                FileBytes::Unreadable => {
                    report.undecryptable += 1;
                    continue;
                }
            }
        }
        report.matched += 1;
        matched.insert(id.clone());
    }
    report.extra = server.keys().filter(|id| !local.contains_key(*id)).count();
    report.undecryptable += broken.len();

    if let Some(manifest) = manifest {
        report.legacy = Some(check_legacy(store, session, &manifest, &local, &matched)?);
    }
    Ok(report)
}

/// Replaces one undecryptable server copy from this Mac: a new revision one
/// generation higher, pushed over the broken one. Returns the pushed
/// version (now synced) when the server took it.
fn repair_one(
    store: &Store,
    sealed: &Sealed,
    guard: &SyncGuard,
    doc: &SyncDoc,
    broken_rev: &str,
) -> Result<Option<SyncDoc>, SyncError> {
    let Some(rev) = store.bump_revision(&doc.id, Some(guard)).map_err(guarded)? else {
        return Ok(None);
    };
    let bumped = SyncDoc {
        rev: rev.clone(),
        ..doc.clone()
    };
    if !bumped.deleted
        && is_file_id(&bumped.id)
        && let Some(blob) = store.stored_attachment(&bumped.id)?
    {
        sealed.put_file(&bumped.id, &blob.data)?;
    }
    sealed.set_repair_base(&bumped.id, broken_rev);
    match sealed.push(std::slice::from_ref(&bumped))?.first() {
        Some(Pushed::Ok) => {
            store
                .mark_synced(&bumped.id, &rev, Some(guard))
                .map_err(guarded)?;
            Ok(Some(SyncDoc {
                base_rev: Some(rev),
                ..bumped
            }))
        }
        _ => Ok(None),
    }
}

fn guarded(err: StoreError) -> SyncError {
    match err {
        StoreError::GuardFailed => SyncError::Cancelled,
        err => SyncError::Store(err),
    }
}

fn read_manifest(path: &Path) -> Result<LegacyManifest, SyncError> {
    let bytes = std::fs::read(path).map_err(StoreError::from)?;
    Ok(serde_json::from_slice(&bytes).map_err(StoreError::from)?)
}

/// The generation of a local revision, or `None` when it isn't one.
fn generation(rev: &str) -> Option<u64> {
    rev.parse::<kanso_core::Rev>().ok().map(|r| r.generation)
}

/// The `--legacy` check (H4): each live legacy document is in the library
/// at that revision or a higher generation and passed the checks above;
/// each live legacy file also has a local blob with the manifest's
/// `md5digest`. Files imported from a PouchDB library may have no `md5digest`; for
/// those the blob must exist and its bytes must hash to what the store
/// recorded for them.
fn check_legacy(
    store: &Store,
    session: &Session,
    manifest: &LegacyManifest,
    local: &BTreeMap<String, SyncDoc>,
    matched: &BTreeSet<String>,
) -> Result<LegacyReport, SyncError> {
    let mut failed = Vec::new();
    for row in &manifest.rows {
        if row.deleted {
            continue;
        }
        let ok = match local.get(&row.id) {
            Some(doc) if !doc.deleted => {
                let revision = doc.rev == row.rev
                    || matches!(
                        (generation(&doc.rev), generation(&row.rev)),
                        (Some(mine), Some(theirs)) if mine > theirs
                    );
                let bytes = if is_file_id(&row.id) {
                    match &row.md5digest {
                        Some(md5) => store.has_blob(&row.id, &md5.to_ascii_lowercase())?,
                        None => match (store.blob_md5(&row.id)?, store.stored_attachment(&row.id)?)
                        {
                            (Some(recorded), Some(blob)) => md5_hex(&blob.data) == recorded,
                            _ => false,
                        },
                    }
                } else {
                    true
                };
                revision && bytes && matched.contains(&row.id)
            }
            _ => false,
        };
        if !ok {
            failed.push(row.id.clone());
        }
    }
    failed.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    failed.dedup();
    Ok(LegacyReport {
        manifest_sha256: manifest_sha256(&manifest.rows),
        account_id: session.account_id.clone(),
        ok: failed.is_empty(),
        rows: manifest.rows.len(),
        failed,
    })
}

/// The charged bytes the library needs on a v2 server: every document
/// sealed as the engine would (exact lengths: canonical JSON, 41 bytes of
/// sealing, base64url) plus 512 bytes a row, and every live file's bytes
/// plus 41 bytes of sealing and 1,024 a row.
pub fn estimate(store: &Store) -> Result<Estimate, StoreError> {
    let mut estimate = Estimate::default();
    for doc in store.sync_docs()? {
        estimate.doc_bytes += sealed_doc_len(&doc) as u64 + DOC_ROW_COST;
        if doc.deleted {
            continue;
        }
        estimate.docs += 1;
        if is_file_id(&doc.id)
            && let Some(blob) = store.stored_attachment(&doc.id)?
        {
            estimate.files += 1;
            estimate.file_bytes += blob.data.len() as u64 + SEAL_OVERHEAD as u64 + FILE_ROW_COST;
        }
    }
    Ok(estimate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn row(id: &str, rev: &str, deleted: bool, md5: Option<&str>) -> LegacyRow {
        LegacyRow {
            id: id.into(),
            rev: rev.into(),
            deleted,
            md5digest: md5.map(Into::into),
        }
    }

    /// The pinned vector the admin script must reproduce (also in
    /// docs/ACCOUNTS.md › Contract notes).
    #[test]
    fn manifest_hash_vector() {
        let rows = [
            row("note:b", "2-bb", false, None),
            row(
                "file:a",
                "1-aa",
                false,
                Some("d41d8cd98f00b204e9800998ecf8427e"),
            ),
            row("tag:c", "3-cc", true, None),
        ];
        let canonical = "file:a\t1-aa\t0\td41d8cd98f00b204e9800998ecf8427e\nnote:b\t2-bb\t0\t\ntag:c\t3-cc\t1\t\n";
        use sha2::Digest as _;
        let expected: String = sha2::Sha256::digest(canonical.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(manifest_sha256(&rows), expected);
        assert_eq!(
            expected,
            "a0dbdfeedc42d0d264500b4faad3718aa823528946c653f34e62b2be42575b92"
        );
        assert_eq!(
            manifest_sha256(&[]),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn manifest_rows_use_the_array_shape() {
        let manifest: LegacyManifest = serde_json::from_str(
            r#"{"exported_at": 5, "rows": [["file:a", "1-aa", false, "d41d8cd98f00b204e9800998ecf8427e"], ["note:b", "2-bb", true, null]]}"#,
        )
        .unwrap();
        assert_eq!(manifest.rows[1], row("note:b", "2-bb", true, None));
        let back = serde_json::to_value(&manifest).unwrap();
        assert_eq!(
            back["rows"][0],
            serde_json::json!(["file:a", "1-aa", false, "d41d8cd98f00b204e9800998ecf8427e"])
        );
    }

    #[test]
    fn verify_report_ok() {
        assert!(VerifyReport::default().ok());
        assert!(
            !VerifyReport {
                pending: 1,
                ..VerifyReport::default()
            }
            .ok()
        );
        assert!(
            VerifyReport {
                extra: 3,
                repaired: 1,
                ..VerifyReport::default()
            }
            .ok()
        );
        let failed = LegacyReport {
            ok: false,
            ..LegacyReport::default()
        };
        assert!(
            !VerifyReport {
                legacy: Some(failed),
                ..VerifyReport::default()
            }
            .ok()
        );
    }

    fn gen_rows() -> impl Strategy<Value = Vec<LegacyRow>> {
        prop::collection::vec(
            (
                "(note|book|tag|file):[a-z0-9]{1,4}",
                "[1-9]-[0-9a-f]{2}",
                any::<bool>(),
                prop::option::of("[0-9a-f]{32}"),
            )
                .prop_map(|(id, rev, deleted, md5digest)| LegacyRow {
                    id,
                    rev,
                    deleted,
                    md5digest,
                }),
            0..8,
        )
        .prop_map(|mut rows| {
            rows.sort_by(|a, b| a.id.cmp(&b.id));
            rows.dedup_by(|a, b| a.id == b.id);
            rows
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Metamorphic: the hash ignores row order and `exported_at`, and any
        /// change to a row's revision, deleted flag or digest changes it.
        #[test]
        fn prop_manifest_hash_is_order_free_and_sensitive(rows in gen_rows(), seed in any::<u64>(), at in any::<prop::sample::Index>()) {
            let mut shuffled = rows.clone();
            let n = shuffled.len();
            for i in (1..n).rev() {
                shuffled.swap(i, (seed as usize).wrapping_mul(31).wrapping_add(i) % (i + 1));
            }
            prop_assert_eq!(manifest_sha256(&rows), manifest_sha256(&shuffled));
            if !rows.is_empty() {
                let i = at.index(rows.len());
                let mut changed = rows.clone();
                changed[i].deleted = !changed[i].deleted;
                prop_assert_ne!(manifest_sha256(&rows), manifest_sha256(&changed));
                let mut changed = rows.clone();
                changed[i].md5digest = match changed[i].md5digest { Some(_) => None, None => Some("0".repeat(32)) };
                prop_assert_ne!(manifest_sha256(&rows), manifest_sha256(&changed));
            }
        }
    }
}
