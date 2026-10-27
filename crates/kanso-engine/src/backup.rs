//! The library's backups without a window: where they go, the daily one and
//! the one made on request, what is in the folder, and deleting one made on
//! request. The store makes the copies ([`kanso_store::Store::backup_if_due`],
//! [`kanso_store::Store::backup_now`]) and reads their names
//! ([`kanso_store::backup_name`]); the desktop's Settings and the phones'
//! screen call this, so both keep, list and delete the same files.
//!
//! The folder is inside the library's own, so a backup is wherever the
//! library is: on a phone that is a folder the device's backups leave out.
//! Nothing here reads a clock of its own: the caller passes the time.

use std::path::{Path, PathBuf};

use kanso_store::{BackupPolicy, Result, Store, StoreError, backup_name};

/// What Settings says of backups on the desktop.
pub const SUMMARY: &str = "Kanso backs up its database once a day and keeps the last 14. A backup made with Back up now is kept until you delete it.";

/// What the phones' screen says of backups.
pub const PHONE_SUMMARY: &str = "Kanso backs up its database on this device once a day and keeps the last 14. A backup made with Back up now is kept until you delete it here.";

/// A backup file in the backups folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackupFile {
    pub path: PathBuf,
    /// When it was made, in milliseconds, as its name says.
    pub made_at_ms: i64,
    /// Made with Back up now, so kept until deleted.
    pub manual: bool,
    pub bytes: u64,
}

/// Where the backups of the library in `folder` go. Pure.
pub fn backups_dir(folder: &Path) -> PathBuf {
    folder.join("backups")
}

/// One backup a day, two weeks kept, in the library `folder`'s backups
/// folder. Pure.
pub fn daily_policy(folder: &Path) -> BackupPolicy {
    BackupPolicy::daily(backups_dir(folder))
}

/// Makes the daily backup when the newest is more than a day old, and drops
/// the daily ones beyond the last 14; the ones made on request stay. The new
/// backup, or `None` when one was made within the day.
pub fn run_daily(store: &Store, folder: &Path, now_ms: i64) -> Result<Option<BackupFile>> {
    let made = store.backup_if_due(&daily_policy(folder), now_ms)?;
    made.map(|path| file(&path)).transpose()
}

/// Makes a backup now, out of the daily rotation: it stays until deleted.
pub fn back_up_now(store: &Store, folder: &Path, now_ms: i64) -> Result<BackupFile> {
    let path = store.backup_now(&backups_dir(folder), now_ms)?;
    file(&path)
}

fn file(path: &Path) -> Result<BackupFile> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(backup_name)
        .ok_or(StoreError::NotFound)?;
    Ok(BackupFile {
        path: path.to_owned(),
        made_at_ms: name.made_at_ms,
        manual: name.manual,
        bytes: std::fs::metadata(path)?.len(),
    })
}

/// The backups in `dir`, newest first: files the daily schedule or Back up
/// now named, and nothing else in the folder. A folder that isn't there has
/// none.
pub fn list(dir: &Path) -> Result<Vec<BackupFile>> {
    let mut found = Vec::new();
    if !dir.is_dir() {
        return Ok(found);
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let Some(name) = entry.file_name().to_str().and_then(backup_name) else {
            continue;
        };
        let meta = entry.metadata()?;
        if !meta.is_file() {
            continue;
        }
        found.push(BackupFile {
            path: entry.path(),
            made_at_ms: name.made_at_ms,
            manual: name.manual,
            bytes: meta.len(),
        });
    }
    found.sort_by(|a, b| {
        b.made_at_ms
            .cmp(&a.made_at_ms)
            .then_with(|| a.path.cmp(&b.path))
    });
    Ok(found)
}

/// Deletes the backup made on request at `path`. Refuses anything else: a
/// daily backup (the schedule keeps those), a file that isn't a backup, and
/// any path that isn't directly inside `dir`, `..` and all.
pub fn delete_manual(dir: &Path, path: &Path) -> Result<()> {
    let refused =
        || StoreError::Invalid("Only a backup made with Back up now can be deleted.".into());
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(refused)?;
    if path.parent() != Some(dir) || !backup_name(name).is_some_and(|n| n.manual) {
        return Err(refused());
    }
    // What goes is named from `dir` and the checked name, never the path as
    // given.
    let target = dir.join(name);
    let meta = std::fs::symlink_metadata(&target).map_err(|err| match err.kind() {
        std::io::ErrorKind::NotFound => StoreError::NotFound,
        _ => err.into(),
    })?;
    if !meta.is_file() {
        return Err(refused());
    }
    std::fs::remove_file(target)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    use super::*;

    const DAY: i64 = 24 * 60 * 60 * 1000;

    fn config(cases: u32) -> ProptestConfig {
        ProptestConfig {
            cases,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    fn store(dir: &Path) -> Store {
        Store::open(dir.join("library.db")).unwrap()
    }

    // no-prop-test: the words desktop Settings keeps showing, word for word.
    #[test]
    fn the_words_are_the_desktops() {
        assert_eq!(
            SUMMARY,
            "Kanso backs up its database once a day and keeps the last 14. A backup made with Back up now is kept until you delete it."
        );
        assert!(PHONE_SUMMARY.contains("on this device"));
    }

    // no-prop-test: a fixed path.
    #[test]
    fn backups_go_in_a_folder_inside_the_library_folder() {
        let folder = Path::new("/library");
        assert_eq!(backups_dir(folder), Path::new("/library/backups"));
        assert_eq!(daily_policy(folder).dir, backups_dir(folder));
        assert_eq!(daily_policy(folder).keep, 14);
    }

    #[derive(Clone, Debug)]
    enum Op {
        Daily(i64),
        OnRequest,
    }

    fn ops() -> impl Strategy<Value = Vec<Op>> {
        prop::collection::vec(
            prop_oneof![
                3 => (0..3 * DAY).prop_map(Op::Daily),
                1 => Just(Op::OnRequest),
            ],
            0..26,
        )
    }

    proptest! {
        #![proptest_config(config(12))]

        /// However the days and the requests fall, at most 14 daily backups
        /// are kept, none of those made on request is ever dropped, and the
        /// list is newest first.
        #[test]
        fn prop_rotation_keeps_14_daily_and_every_one_made_on_request(ops in ops()) {
            let dir = tempfile::tempdir().unwrap();
            let store = store(dir.path());
            let mut now = 1_000_000_000_000_i64;
            let mut manual = BTreeSet::new();
            for op in ops {
                now += 1;
                match op {
                    Op::Daily(advance) => {
                        now += advance;
                        run_daily(&store, dir.path(), now).unwrap();
                    }
                    Op::OnRequest => {
                        let made = back_up_now(&store, dir.path(), now).unwrap();
                        prop_assert!(made.manual);
                        prop_assert!(made.bytes > 0);
                        manual.insert(made.path);
                    }
                }
                let listed = list(&backups_dir(dir.path())).unwrap();
                prop_assert!(listed.iter().filter(|b| !b.manual).count() <= 14);
                let kept: BTreeSet<_> = listed.iter().filter(|b| b.manual).map(|b| b.path.clone()).collect();
                prop_assert_eq!(&kept, &manual);
                prop_assert!(listed.windows(2).all(|w| w[0].made_at_ms >= w[1].made_at_ms));
            }
        }
    }

    // no-prop-test: three calls at fixed times.
    #[test]
    fn the_daily_backup_is_made_once_a_day() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(dir.path());
        let first = run_daily(&store, dir.path(), 5 * DAY).unwrap().unwrap();
        assert!(!first.manual);
        assert_eq!(first.made_at_ms, 5 * DAY);
        assert!(first.path.starts_with(backups_dir(dir.path())));
        assert!(
            run_daily(&store, dir.path(), 5 * DAY + DAY - 1)
                .unwrap()
                .is_none()
        );
        assert!(run_daily(&store, dir.path(), 6 * DAY).unwrap().is_some());
        assert_eq!(list(&backups_dir(dir.path())).unwrap().len(), 2);
    }

    fn name() -> impl Strategy<Value = String> {
        prop_oneof![
            (0..i64::MAX / 2).prop_map(|n| format!("kanso-{n}.db")),
            (0..i64::MAX / 2).prop_map(|n| format!("kanso-manual-{n}.db")),
            "[a-z0-9.+-]{1,14}",
            "kanso-[a-z0-9+-]{0,8}(\\.db)?",
            "kanso-manual-[a-z0-9+-]{0,8}(\\.db)?",
        ]
    }

    proptest! {
        #![proptest_config(config(48))]

        /// Only files named as a daily or an on-request backup are listed,
        /// each as its name says; nothing else in the folder is.
        #[test]
        fn prop_list_parses_only_valid_names(names in prop::collection::btree_set(name(), 0..10)) {
            let dir = tempfile::tempdir().unwrap();
            // `.` and `..` name folders rather than files, so a write to
            // either would fail for a reason the listing knows nothing about.
            let names: BTreeSet<String> =
                names.into_iter().filter(|n| n != "." && n != "..").collect();
            for name in &names {
                std::fs::write(dir.path().join(name), b"x").unwrap();
            }
            let listed = list(dir.path()).unwrap();
            let expected: BTreeSet<&String> = names
                .iter()
                .filter(|n| backup_name(n).is_some())
                .collect();
            let got: BTreeSet<String> = listed
                .iter()
                .map(|b| b.path.file_name().unwrap().to_str().unwrap().to_owned())
                .collect();
            prop_assert_eq!(got.iter().collect::<BTreeSet<_>>(), expected);
            for b in &listed {
                let parsed = backup_name(b.path.file_name().unwrap().to_str().unwrap()).unwrap();
                prop_assert_eq!((b.made_at_ms, b.manual), (parsed.made_at_ms, parsed.manual));
            }
        }

        /// Whatever path is given, only a file made on request that sits
        /// directly in the folder is ever deleted: not a daily backup, not
        /// another file, not anything outside the folder.
        #[test]
        fn prop_delete_manual_deletes_only_a_manual_file_inside_the_folder(
            target in prop_oneof![
                Just("kanso-manual-5.db".to_owned()),
                Just("kanso-7.db".to_owned()),
                Just("notes.txt".to_owned()),
                Just("../outside/kanso-manual-9.db".to_owned()),
                Just("../outside/kanso-9.db".to_owned()),
                Just("sub/../kanso-manual-5.db".to_owned()),
                Just("sub/kanso-manual-6.db".to_owned()),
                Just("./kanso-manual-5.db".to_owned()),
                Just("kanso-manual-5.db/..".to_owned()),
                Just("..".to_owned()),
                Just("".to_owned()),
            ],
            absolute_outside: bool,
        ) {
            let root = tempfile::tempdir().unwrap();
            let dir = root.path().join("backups");
            std::fs::create_dir_all(dir.join("sub")).unwrap();
            std::fs::create_dir_all(root.path().join("outside")).unwrap();
            let all = [
                "backups/kanso-manual-5.db",
                "backups/kanso-7.db",
                "backups/notes.txt",
                "backups/sub/kanso-manual-6.db",
                "outside/kanso-manual-9.db",
                "outside/kanso-9.db",
            ];
            for f in all {
                std::fs::write(root.path().join(f), b"x").unwrap();
            }
            let path = if absolute_outside && target.contains("outside") {
                root.path().join("outside").join(target.rsplit('/').next().unwrap())
            } else {
                dir.join(&target)
            };
            let result = delete_manual(&dir, &path);
            let gone: Vec<&str> = all
                .iter()
                .copied()
                .filter(|f| !root.path().join(f).exists())
                .collect();
            if result.is_ok() {
                prop_assert_eq!(gone, vec!["backups/kanso-manual-5.db"]);
                prop_assert!(path.file_name().unwrap() == "kanso-manual-5.db");
            } else {
                prop_assert!(gone.is_empty(), "{gone:?}");
            }
        }
    }

    // no-prop-test: the three kinds of refusal, by name.
    #[test]
    fn deleting_refuses_the_daily_ones_and_deletes_the_one_asked_for() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(dir.path());
        let daily = run_daily(&store, dir.path(), 5 * DAY).unwrap().unwrap();
        let manual = back_up_now(&store, dir.path(), 6 * DAY).unwrap();
        let folder = backups_dir(dir.path());
        assert!(matches!(
            delete_manual(&folder, &daily.path),
            Err(StoreError::Invalid(_))
        ));
        assert!(daily.path.exists());
        delete_manual(&folder, &manual.path).unwrap();
        assert!(!manual.path.exists());
        assert!(matches!(
            delete_manual(&folder, &manual.path),
            Err(StoreError::NotFound)
        ));
    }
}
