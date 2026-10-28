//! The library's database backups: the daily one, the one made on request,
//! the list and deleting one made on request. The engine decides where they
//! go and keeps them ([`kanso_engine::backup`]); the folder is inside the
//! library's own, so it is out of the device's backups with the rest of it
//! (`backup.rs`).

use std::path::{Path, PathBuf};

use kanso_core::now_ms;
use kanso_engine::backup;

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::library::Library;
use crate::model_history::BackupFile;

/// What the screen says of backups on a phone.
pub const BACKUPS_SUMMARY: &str = backup::PHONE_SUMMARY;

impl Library {
    /// The folder that holds the library's database and its backups.
    fn library_folder(&self) -> &Path {
        self.inner
            .path
            .parent()
            .filter(|dir| !dir.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."))
    }

    fn backups_folder(&self) -> PathBuf {
        backup::backups_dir(self.library_folder())
    }

    /// Makes the daily backup if the newest is more than a day old, and
    /// drops the daily ones beyond the last 14. The new backup, or `None`
    /// when one was made within the day.
    pub fn backup_if_due(&self) -> Result<Option<BackupFile>, KansoError> {
        guard(|| {
            let made = backup::run_daily(&self.inner.store, self.library_folder(), now_ms())?;
            Ok(made.map(file))
        })
    }

    /// Makes a backup now. It stays until [`Library::delete_backup`].
    pub fn back_up_now(&self) -> Result<BackupFile, KansoError> {
        guard(|| {
            let made = backup::back_up_now(&self.inner.store, self.library_folder(), now_ms())?;
            Ok(file(made))
        })
    }

    /// The backups, newest first.
    pub fn backups(&self) -> Result<Vec<BackupFile>, KansoError> {
        guard(|| {
            Ok(backup::list(&self.backups_folder())?
                .into_iter()
                .map(file)
                .collect())
        })
    }

    /// Deletes a backup made with [`Library::back_up_now`], by the path
    /// [`Library::backups`] gave. Refuses a daily backup, which the schedule
    /// keeps, and anything else ([`KansoError::Invalid`]).
    pub fn delete_backup(&self, path: &str) -> Result<(), KansoError> {
        guard(|| {
            Ok(backup::delete_manual(
                &self.backups_folder(),
                Path::new(path),
            )?)
        })
    }
}

fn file(file: backup::BackupFile) -> BackupFile {
    BackupFile {
        path: file.path.to_string_lossy().into_owned(),
        made_at_ms: file.made_at_ms,
        manual: file.manual,
        bytes: file.bytes,
    }
}
