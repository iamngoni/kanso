//! Restoring a backup in place of the library (issue #243 step 11).
//!
//! A backup of an encrypted library opens only with the library key it was
//! made with: this computer's, or on another computer the 24 words. The
//! running app can't swap a library out from under itself, so a restore has
//! two halves:
//!
//! 1. [`stage_restore`] opens the backup with the key it needs, writes a copy
//!    encrypted with *this* library's key to `<db>.restore`, and checks the
//!    copy row for row. The library is untouched.
//! 2. [`apply_staged_restore`], at the next launch before the library opens,
//!    keeps the library it replaces as a backup of its own and renames the
//!    copy over it. A rename is atomic, and the step can be run again after a
//!    crash: whichever of the two names is missing is finished.
//!
//! The caller holds the data-folder lock.

use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};

use crate::encrypt::{export, map_full, open_encrypted, open_plain, shred};
use crate::encrypt::{sync_dir, sync_file, verify, with_suffix};
use crate::{EncryptOptions, LibraryFile, LibraryKey, Result, StoreError, schema};

/// Where a restore waits for the next launch.
pub fn staged_restore_path(db: &Path) -> PathBuf {
    with_suffix(db, ".restore")
}

/// Whether a restore is waiting for the next launch.
pub fn restore_staged(db: &Path) -> bool {
    staged_restore_path(db).exists()
}

/// Opens `backup` read-only and proves it is a Kanso library that `key`
/// (none for a plain backup) unlocks. [`StoreError::WrongKey`] says the key
/// doesn't fit; [`StoreError::Invalid`] that the file isn't a library.
pub fn check_backup(backup: &Path, key: Option<&LibraryKey>) -> Result<()> {
    open_backup(backup, key, false).map(drop)
}

/// `writable` is for [`stage_restore`]: SQLite attaches the copy it exports
/// to with the same flags, so a read-only backup can't be exported. Nothing
/// is written to the backup either way (it exists, so it is never created;
/// the flag is for the attached copy).
fn open_backup(backup: &Path, key: Option<&LibraryKey>, writable: bool) -> Result<Connection> {
    let kind = LibraryFile::of(backup);
    match (kind, key) {
        (LibraryFile::New, _) => {
            return Err(StoreError::Invalid("That file is empty.".to_owned()));
        }
        (LibraryFile::Encrypted, None) => return Err(StoreError::WrongKey),
        _ => {}
    }
    let mode = if writable {
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE
    } else {
        OpenFlags::SQLITE_OPEN_READ_ONLY
    };
    let conn = Connection::open_with_flags(backup, mode | OpenFlags::SQLITE_OPEN_NO_MUTEX)?;
    // A plain backup is opened without a key it wasn't written with.
    let key = (kind == LibraryFile::Encrypted).then_some(key).flatten();
    crate::library_key::unlock(&conn, key).map_err(|err| match err {
        // A file that isn't a database at all, with no key to blame.
        StoreError::WrongKey if kind == LibraryFile::Plain => {
            StoreError::Invalid("That file isn't a Kanso backup.".to_owned())
        }
        other => other,
    })?;
    let library: i64 = conn.query_row(
        "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'meta'",
        [],
        |row| row.get(0),
    )?;
    if library == 0 {
        return Err(StoreError::Invalid(
            "That file isn't a Kanso backup.".to_owned(),
        ));
    }
    schema::register_functions(&conn)?;
    Ok(conn)
}

/// Writes the backup, re-encrypted with `library_key`, beside the library
/// as `<db>.restore`, checked against the backup. `backup_key` is the key
/// the backup was made with (none when it is plain). Nothing about the
/// library changes until the next launch applies it.
pub fn stage_restore(
    db: &Path,
    backup: &Path,
    backup_key: Option<&LibraryKey>,
    library_key: &LibraryKey,
) -> Result<()> {
    let staged = staged_restore_path(db);
    let temp = with_suffix(&staged, ".tmp");
    let needed = std::fs::metadata(backup)
        .map(|meta| meta.len())
        .unwrap_or(0);
    remove_with_sidecars(&temp);
    let made = (|| {
        let src = open_backup(backup, backup_key, true)?;
        export(&src, &temp, library_key, None)?;
        verify(
            &src,
            &temp,
            library_key,
            &EncryptOptions::default(),
            0.0,
            1.0,
        )?;
        sync_file(&temp)?;
        Ok::<(), StoreError>(())
    })();
    if let Err(err) = made {
        remove_with_sidecars(&temp);
        return Err(map_full(err, needed));
    }
    std::fs::rename(&temp, &staged)?;
    sync_dir(&staged);
    Ok(())
}

fn remove_with_sidecars(path: &Path) {
    let _ = std::fs::remove_file(path);
    for suffix in ["-wal", "-shm", "-journal"] {
        let _ = std::fs::remove_file(with_suffix(path, suffix));
    }
}

/// Forgets a staged restore.
pub fn cancel_staged_restore(db: &Path) {
    remove_with_sidecars(&staged_restore_path(db));
}

/// If a restore is staged, makes it the library: the library it replaces is
/// kept in `keep_dir` as a backup made now (`now_ms`), and the staged copy is
/// renamed over `db`. Returns whether it did. A staged copy that doesn't open
/// with `library_key` is deleted and reported, so a bad restore can't stop
/// Kanso from starting at every launch.
pub fn apply_staged_restore(
    db: &Path,
    library_key: &LibraryKey,
    keep_dir: &Path,
    now_ms: i64,
) -> Result<bool> {
    let staged = staged_restore_path(db);
    if !staged.exists() {
        return Ok(false);
    }
    let applied = (|| {
        // The copy is the one thing that must be sound.
        let conn = open_encrypted(&staged, library_key)?;
        let check: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
        if check != "ok" {
            return Err(StoreError::Invalid(format!(
                "the restored copy failed its check ({check})"
            )));
        }
        drop(conn);
        if db.exists() {
            keep_library(db, library_key, keep_dir, now_ms)?;
        }
        std::fs::rename(&staged, db)?;
        sync_dir(db);
        Ok(())
    })();
    if applied.is_err() && staged.exists() {
        cancel_staged_restore(db);
    }
    applied.map(|()| true)
}

/// Moves the library at `db` into `keep_dir` as a backup. Its write-ahead log
/// goes into it first, and the log and shared memory files are deleted.
fn keep_library(db: &Path, key: &LibraryKey, keep_dir: &Path, now_ms: i64) -> Result<()> {
    std::fs::create_dir_all(keep_dir)?;
    let kept = crate::backup::manual_backup_path(keep_dir, now_ms);
    let conn = if LibraryFile::of(db) == LibraryFile::Plain {
        open_plain(db)?
    } else {
        open_encrypted(db, key)?
    };
    let busy: i64 = conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| row.get(0))?;
    if busy != 0 {
        return Err(StoreError::Invalid(
            "the library is in use by another program".to_owned(),
        ));
    }
    drop(conn);
    if std::fs::rename(db, &kept).is_err() {
        std::fs::copy(db, &kept)?;
        sync_file(&kept)?;
        std::fs::remove_file(db)?;
    }
    for suffix in ["-wal", "-shm", "-journal"] {
        shred(&with_suffix(db, suffix))?;
    }
    sync_dir(&kept);
    Ok(())
}
