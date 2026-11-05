//! Clearing out what recordings leave behind (issue #38, step 11): a
//! session folder that was never recovered or discarded is deleted once
//! nothing has touched it for seven days.
//!
//! A folder is swept only when it is certainly abandoned: it is not a
//! recording in progress (nor the system track of one), its newest file is
//! older than [`KEEP_MS`], and its name is a session ID. Anything the sweep
//! can't date is kept. A recording being written changes its folder every
//! second, so even one this process doesn't know about is never recent
//! enough to be swept.

use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

pub use crate::sweep_model::{SweepEntry, SweepReport};

use crate::session::{remove_session, valid_id};

/// How long an unfinished recording is kept: seven days.
pub const KEEP_MS: u64 = 7 * 24 * 60 * 60 * 1000;

/// What a system audio track's session ID adds to its recording's. The two
/// are one recording: while the first runs, so does the second.
const SYSTEM_SUFFIX: &str = "-sys";

/// Whether session `id` is one of the recordings in `live`, or the system
/// track beside one. Pure.
fn is_live(id: &str, live: &[&str]) -> bool {
    live.iter().any(|live| {
        id == *live
            || id.strip_suffix(SYSTEM_SUFFIX) == Some(*live)
            || live.strip_suffix(SYSTEM_SUFFIX) == Some(id)
    })
}

/// The sessions in `entries` that are due to be deleted: not live, and idle
/// for more than `keep_ms` at `now_ms`. Pure.
pub fn stale(entries: &[SweepEntry], live: &[&str], now_ms: u64, keep_ms: u64) -> Vec<String> {
    entries
        .iter()
        .filter(|entry| valid_id(&entry.id) && !is_live(&entry.id, live))
        .filter(|entry| {
            entry
                .last_active_ms
                .is_some_and(|at| now_ms.checked_sub(at).is_some_and(|idle| idle > keep_ms))
        })
        .map(|entry| entry.id.clone())
        .collect()
}

/// The newest change in a session folder: the folder itself or any file in
/// it (not below).
fn last_active(dir: &Path) -> Option<u64> {
    let ms = |meta: &fs::Metadata| {
        meta.modified()
            .ok()
            .and_then(|at| at.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
    };
    let mut newest = ms(&fs::metadata(dir).ok()?)?;
    for entry in fs::read_dir(dir).ok()?.flatten() {
        // A file that can't be dated makes the folder undatable: kept.
        newest = newest.max(ms(&entry.metadata().ok()?)?);
    }
    Some(newest)
}

/// The session folders under `root` (the in-progress folder), dated.
pub fn entries(root: &Path) -> Vec<SweepEntry> {
    let Ok(listing) = fs::read_dir(root) else {
        return Vec::new();
    };
    listing
        .flatten()
        // A link is never followed: only real folders are session folders.
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .filter_map(|entry| {
            let id = entry.file_name().to_str()?.to_owned();
            Some(SweepEntry {
                last_active_ms: last_active(&entry.path()),
                id,
            })
        })
        .collect()
}

/// Deletes the session folders under `root` that have sat idle for more
/// than `keep_ms`, except the recordings in `live`.
pub fn sweep_in_progress(root: &Path, live: &[&str], now_ms: u64, keep_ms: u64) -> SweepReport {
    let mut report = SweepReport::default();
    for id in stale(&entries(root), live, now_ms, keep_ms) {
        match remove_session(&root.join(&id)) {
            Ok(()) => report.removed.push(id),
            Err(_) => report.failed.push(id),
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const DAY: u64 = 24 * 60 * 60 * 1000;

    fn entry(id: &str, at: Option<u64>) -> SweepEntry {
        SweepEntry {
            id: id.to_owned(),
            last_active_ms: at,
        }
    }

    #[test]
    fn only_a_folder_idle_for_over_seven_days_is_swept() {
        let now = 100 * DAY;
        let entries = [
            entry("old-1", Some(now - 8 * DAY)),
            entry("edge-1", Some(now - 7 * DAY)),
            entry("new-1", Some(now - DAY)),
            entry("undated-1", None),
            entry("future-1", Some(now + DAY)),
        ];
        assert_eq!(stale(&entries, &[], now, KEEP_MS), vec!["old-1"]);
    }

    #[test]
    fn a_live_recording_and_its_system_track_are_never_swept() {
        let now = 100 * DAY;
        let old = Some(1);
        let entries = [entry("a-1", old), entry("a-1-sys", old), entry("b-2", old)];
        assert_eq!(stale(&entries, &["a-1"], now, KEEP_MS), vec!["b-2"]);
        // The system track alone is live: its microphone folder stays too.
        assert_eq!(stale(&entries, &["a-1-sys"], now, KEEP_MS), vec!["b-2"]);
    }

    // no-prop-test: filesystem glue over `stale`, which has the properties.
    #[test]
    fn the_sweep_deletes_old_folders_and_keeps_new_and_live_ones() {
        let dir = tempfile::tempdir().unwrap();
        for id in ["old-1", "live-1", "fresh-1"] {
            fs::create_dir(dir.path().join(id)).unwrap();
            fs::write(dir.path().join(id).join("part-0001.opus"), b"x").unwrap();
        }
        // A file that isn't a session folder is left alone.
        fs::write(dir.path().join("pending-transcript.json"), b"{}").unwrap();
        let later = last_active(&dir.path().join("old-1")).unwrap() + 8 * DAY;
        let report = sweep_in_progress(dir.path(), &["live-1"], later, KEEP_MS);
        let mut removed = report.removed.clone();
        removed.sort();
        assert_eq!(removed, vec!["fresh-1", "old-1"]);
        assert!(dir.path().join("live-1").is_dir());
        assert!(dir.path().join("pending-transcript.json").is_file());
        // At the real time nothing is old.
        let now = last_active(&dir.path().join("live-1")).unwrap();
        let report = sweep_in_progress(dir.path(), &[], now, KEEP_MS);
        assert!(report.removed.is_empty());
    }

    proptest! {
        #![proptest_config(ProptestConfig { rng_seed: proptest::test_runner::RngSeed::Fixed(20261017), ..ProptestConfig::default() })]

        /// The sweep never deletes a live or recent recording, whatever the
        /// folders, the clock and the live list.
        #[test]
        fn prop_the_sweep_never_deletes_a_live_or_recent_recording(
            folders in proptest::collection::vec(("[a-z0-9]{1,6}-[a-z0-9]{1,6}", proptest::option::of(0u64..400 * DAY)), 0..12),
            live_picks in proptest::collection::vec(0usize..12, 0..4),
            now in 0u64..400 * DAY,
            sys in proptest::bool::ANY,
        ) {
            let entries: Vec<SweepEntry> =
                folders.iter().map(|(id, at)| entry(id, *at)).collect();
            let mut live: Vec<String> = live_picks
                .iter()
                .filter_map(|ix| entries.get(*ix).map(|e| e.id.clone()))
                .collect();
            if sys {
                // A live recording whose system track is in the list too.
                let tracks: Vec<String> = live.iter().map(|id| format!("{id}-sys")).collect();
                live.extend(tracks);
            }
            let live_refs: Vec<&str> = live.iter().map(String::as_str).collect();
            let swept = stale(&entries, &live_refs, now, KEEP_MS);
            for id in &swept {
                prop_assert!(!live.contains(id), "swept a live recording {id}");
                let at = entries.iter().find(|e| &e.id == id).and_then(|e| e.last_active_ms);
                let at = at.expect("an undated folder was swept");
                prop_assert!(now > at && now - at > KEEP_MS, "swept a recent recording {id}");
            }
        }
    }
}
