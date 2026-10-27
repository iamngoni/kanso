//! Periodic copies of the database made with `VACUUM INTO`.

use std::path::{Path, PathBuf};

use crate::{Result, Store};

/// Where backups go, how many to keep, and how often to make one.
#[derive(Clone, Debug)]
pub struct BackupPolicy {
    pub dir: PathBuf,
    pub keep: usize,
    pub min_interval_ms: i64,
}

impl BackupPolicy {
    /// One backup a day, two weeks kept.
    pub fn daily(dir: impl Into<PathBuf>) -> Self {
        BackupPolicy {
            dir: dir.into(),
            keep: 14,
            min_interval_ms: 24 * 60 * 60 * 1000,
        }
    }
}

const PREFIX: &str = "kanso-";
const SUFFIX: &str = ".db";
/// What sets a backup made on request apart from the daily ones.
const MANUAL: &str = "manual-";

/// The file name of the daily backup made at `stamp`.
fn daily_name(stamp: i64) -> String {
    format!("{PREFIX}{stamp}{SUFFIX}")
}

/// When the daily backup named `name` was made; `None` for any other file,
/// a backup made on request among them.
fn daily_stamp(name: &str) -> Option<i64> {
    name.strip_prefix(PREFIX)?
        .strip_suffix(SUFFIX)?
        .parse::<i64>()
        .ok()
}

/// What a backup file's name says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackupName {
    /// When the backup was made, in milliseconds.
    pub made_at_ms: i64,
    /// Made on request ([`manual_backup_path`]), not by the daily schedule.
    pub manual: bool,
}

/// What the file name `name` says of a backup; `None` for any other file.
/// Pure.
pub fn backup_name(name: &str) -> Option<BackupName> {
    if let Some(made_at_ms) = daily_stamp(name) {
        return Some(BackupName {
            made_at_ms,
            manual: false,
        });
    }
    let stamp = name
        .strip_prefix(PREFIX)?
        .strip_prefix(MANUAL)?
        .strip_suffix(SUFFIX)?
        .parse::<i64>()
        .ok()?;
    Some(BackupName {
        made_at_ms: stamp,
        manual: true,
    })
}

/// Where a backup made on request at `now_ms` goes: beside the daily ones,
/// under a name their rotation passes over. It neither counts toward the
/// daily backups kept nor stands in for the next one, and stays until
/// someone deletes it.
pub fn manual_backup_path(dir: &Path, now_ms: i64) -> PathBuf {
    dir.join(format!("{PREFIX}{MANUAL}{now_ms}{SUFFIX}"))
}

/// The daily backups in `dir`, oldest first.
fn backups(dir: &Path) -> Result<Vec<(i64, PathBuf)>> {
    let mut found = Vec::new();
    if !dir.exists() {
        return Ok(found);
    }
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        let stamp = path
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(daily_stamp);
        if let Some(stamp) = stamp {
            found.push((stamp, path));
        }
    }
    found.sort();
    Ok(found)
}

impl Store {
    /// Writes a consistent copy of the database to `path`.
    pub fn backup_to(&self, path: &Path) -> Result<()> {
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        let target = path.to_string_lossy().into_owned();
        self.read(|conn| {
            conn.execute("VACUUM INTO ?", [target])?;
            Ok(())
        })
    }

    /// Makes a backup in `dir` now, whatever the daily ones, and keeps it out
    /// of their rotation. Returns its path.
    pub fn backup_now(&self, dir: &Path, now_ms: i64) -> Result<PathBuf> {
        std::fs::create_dir_all(dir)?;
        let path = manual_backup_path(dir, now_ms);
        self.backup_to(&path)?;
        Ok(path)
    }

    /// Makes a backup when the newest one is older than the policy's interval,
    /// then deletes the oldest beyond `keep`. Returns the new backup's path.
    pub fn backup_if_due(&self, policy: &BackupPolicy, now_ms: i64) -> Result<Option<PathBuf>> {
        std::fs::create_dir_all(&policy.dir)?;
        let existing = backups(&policy.dir)?;
        if existing
            .last()
            .is_some_and(|(stamp, _)| now_ms - stamp < policy.min_interval_ms)
        {
            return Ok(None);
        }
        let path = policy.dir.join(daily_name(now_ms));
        self.backup_to(&path)?;
        let all = backups(&policy.dir)?;
        let excess = all.len().saturating_sub(policy.keep.max(1));
        for (_, old) in all.into_iter().take(excess) {
            std::fs::remove_file(old)?;
        }
        Ok(Some(path))
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    use super::{BackupName, backup_name, daily_name, daily_stamp, manual_backup_path};

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    fn name_of(path: &Path) -> &str {
        path.file_name().unwrap().to_str().unwrap()
    }

    /// The bug this guards: "Back up now" wrote `kanso-<ms>.db`, which the
    /// daily rotation counted and pruned.
    #[test]
    fn names_examples() {
        assert_eq!(daily_name(1_779_954_300_000), "kanso-1779954300000.db");
        assert_eq!(
            daily_stamp("kanso-1779954300000.db"),
            Some(1_779_954_300_000)
        );
        let manual = manual_backup_path(Path::new("/b"), 1_779_954_300_000);
        assert_eq!(name_of(&manual), "kanso-manual-1779954300000.db");
        assert_eq!(
            backup_name(name_of(&manual)),
            Some(BackupName {
                made_at_ms: 1_779_954_300_000,
                manual: true
            })
        );
        assert_eq!(
            backup_name("kanso-5.db"),
            Some(BackupName {
                made_at_ms: 5,
                manual: false
            })
        );
        assert_eq!(backup_name("kanso-manual-.db"), None);
        assert_eq!(backup_name("kanso-manual-x.db"), None);
        assert_eq!(daily_stamp(name_of(&manual)), None);
        for other in [
            "kanso.db",
            "kanso-.db",
            "kanso-12.db-wal",
            "notes-12.db",
            ".DS_Store",
        ] {
            assert_eq!(daily_stamp(other), None, "{other}");
        }
    }

    proptest! {
        #![proptest_config(config())]

        /// Invariants: a daily backup's name gives back its time; a backup
        /// made on request is never taken for a daily one, whatever its
        /// time; and it lands in the folder it was asked for.
        #[test]
        fn prop_manual_backups_stay_out_of_the_rotation(stamp in any::<i64>(), dir in "[a-z]{1,8}(/[a-z]{1,8}){0,2}") {
            prop_assert_eq!(daily_stamp(&daily_name(stamp)), Some(stamp));
            let manual = manual_backup_path(Path::new(&dir), stamp);
            prop_assert_eq!(daily_stamp(name_of(&manual)), None);
            prop_assert_eq!(manual.parent(), Some(Path::new(&dir)));
            prop_assert_eq!(
                backup_name(name_of(&manual)),
                Some(BackupName { made_at_ms: stamp, manual: true })
            );
            prop_assert_eq!(
                backup_name(&daily_name(stamp)),
                Some(BackupName { made_at_ms: stamp, manual: false })
            );
        }
    }
}
