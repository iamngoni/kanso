//! Restoring a backup in place of the library (issue #243 step 11).
//!
//! A backup of an encrypted library opens only with the library key it was
//! made with: this computer's, or on another computer the 24 words. The
//! running app can't swap a library out from under itself, so a restore has
//! two halves:
//!
//! 1. [`stage_restore`] copies the backup to a private file (the backup
//!    itself is only ever opened read-only), opens the copy with the key it
//!    needs, writes it encrypted with *this* library's key to `<db>.restore`,
//!    and checks that row for row. The library is untouched, and a failure
//!    (wrong words, a file that isn't a backup, a full disk) leaves nothing
//!    behind.
//! 2. [`apply_staged_restore`], at the next launch before the library opens,
//!    checks the staged copy, gives it the library's API keys in place of
//!    the backup's (keys never travel with a backup, so a revoked key stays
//!    revoked), keeps the library as a backup of its own (a
//!    second name for the same file, or a finished copy), then renames the
//!    staged copy over `<db>`. The rename is atomic, so `<db>` is always the
//!    old library or the restored one, never missing and never half of
//!    either. A crash anywhere leaves the staged copy, and the next launch
//!    runs the step again.
//!
//! The caller holds the data-folder lock.

use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};

use crate::encrypt::{export, map_full, open_encrypted, open_plain, shred};
use crate::encrypt::{sync_dir, sync_file, verify, with_suffix};
use crate::{EncryptOptions, LibraryFile, LibraryKey, Result, StoreError, schema};

/// A database file's side files.
const SIDECARS: [&str; 3] = ["-wal", "-shm", "-journal"];

/// Where a restore waits for the next launch.
pub fn staged_restore_path(db: &Path) -> PathBuf {
    with_suffix(db, ".restore")
}

/// Whether a restore is waiting for the next launch.
pub fn restore_staged(db: &Path) -> bool {
    staged_restore_path(db).exists()
}

/// The private copy of the backup that [`stage_restore`] reads from.
fn source_copy_path(db: &Path) -> PathBuf {
    with_suffix(&staged_restore_path(db), ".src")
}

/// Opens `backup` read-only and proves it is a Kanso library that `key`
/// (none for a plain backup) unlocks. [`StoreError::WrongKey`] says the key
/// doesn't fit; [`StoreError::Invalid`] that the file isn't a library.
pub fn check_backup(backup: &Path, key: Option<&LibraryKey>) -> Result<()> {
    open_backup(backup, key, false).map(drop)
}

/// `writable` is for [`stage_restore`]'s private copy: SQLite attaches the
/// copy it exports to with the same flags, so a read-only source can't be
/// exported. The person's backup is only ever opened read-only.
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

/// Copies `backup`, and a write-ahead log beside it, to `to`. The copy is
/// what [`stage_restore`] opens read-write, so the backup is never written
/// to, and one on a read-only disk can be restored.
fn copy_source(backup: &Path, to: &Path) -> Result<()> {
    std::fs::copy(backup, to)?;
    let wal = with_suffix(backup, "-wal");
    if wal.exists() {
        std::fs::copy(&wal, with_suffix(to, "-wal"))?;
    }
    Ok(())
}

/// Writes the backup, re-encrypted with `library_key`, beside the library
/// as `<db>.restore`, checked against the backup. `backup_key` is the key
/// the backup was made with (none when it is plain). Nothing about the
/// library changes until the next launch applies it, and a failure leaves
/// no file behind (a restore staged before stays as it was).
pub fn stage_restore(
    db: &Path,
    backup: &Path,
    backup_key: Option<&LibraryKey>,
    library_key: &LibraryKey,
) -> Result<()> {
    let staged = staged_restore_path(db);
    let temp = with_suffix(&staged, ".tmp");
    let source = source_copy_path(db);
    // The private copy, then the re-encrypted one.
    let needed = std::fs::metadata(backup)
        .map(|meta| meta.len().saturating_mul(2))
        .unwrap_or(0);
    // The key is checked on the backup itself first, read-only, so wrong
    // words cost no copy.
    check_backup(backup, backup_key)?;
    remove_with_sidecars(&temp);
    remove_with_sidecars(&source);
    let made = (|| {
        copy_source(backup, &source)?;
        let src = open_backup(&source, backup_key, true)?;
        export(&src, &temp, library_key, None)?;
        verify(
            &src,
            &temp,
            library_key,
            &EncryptOptions::default(),
            0.0,
            1.0,
        )?;
        drop(src);
        sync_file(&temp)?;
        Ok::<(), StoreError>(())
    })();
    // A plain backup's copy is plaintext: overwritten, not just unlinked.
    shred_with_sidecars(&source);
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
    for suffix in SIDECARS {
        let _ = std::fs::remove_file(with_suffix(path, suffix));
    }
}

fn shred_with_sidecars(path: &Path) {
    let _ = shred(path);
    for suffix in SIDECARS {
        let _ = shred(&with_suffix(path, suffix));
    }
}

/// Forgets a staged restore.
pub fn cancel_staged_restore(db: &Path) {
    remove_with_sidecars(&staged_restore_path(db));
}

/// If a restore is staged, makes it the library: the library it replaces is
/// kept in `keep_dir` as a backup made now (`now_ms`), and the staged copy is
/// renamed over `db`. Returns whether it did.
///
/// A staged copy that doesn't open with `library_key`, or fails its check,
/// can never be applied: it is deleted and reported, and the library is
/// left alone. Any other failure (the library in use, a full disk, a folder
/// that can't be written) leaves the library *and* the staged copy as they
/// are, so the next launch tries again; nothing is lost either way.
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
    if let Err(err) = check_staged(&staged, library_key) {
        if !matches!(err, StoreError::Io(_)) {
            cancel_staged_restore(db);
        }
        return Err(err);
    }
    keep_api_keys(&staged, db, library_key)?;
    if LibraryFile::of(db) != LibraryFile::New {
        keep_library(db, library_key, keep_dir, now_ms)?;
    }
    std::fs::rename(&staged, db)?;
    sync_dir(db);
    remove_with_sidecars(&staged);
    Ok(true)
}

/// The columns of `api_keys`, the same in every library that has it.
const API_KEY_COLUMNS: &str =
    "id, name, client, token_hash, access, books, created_at, last_used_at, revoked_at";

/// API keys never travel with a backup (issue #243: a key works on one
/// computer only, and a revoked key stays revoked). The staged copy's
/// `api_keys` are replaced by the library's as they are now; with no
/// library, it has none. Run again after a crash, it gives the same table.
fn keep_api_keys(staged: &Path, db: &Path, key: &LibraryKey) -> Result<()> {
    let conn = open_encrypted(staged, key)?;
    // A backup from before API keys has no table yet; the open that
    // migrates it would make one, empty. Made now, so it can be filled.
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS api_keys (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            client TEXT NOT NULL,
            token_hash BLOB NOT NULL UNIQUE,
            access TEXT NOT NULL CHECK (access IN ('read', 'suggest', 'write')),
            books TEXT,
            created_at INTEGER NOT NULL,
            last_used_at INTEGER,
            revoked_at INTEGER
        ) WITHOUT ROWID;
        DELETE FROM api_keys;",
    )?;
    let kind = LibraryFile::of(db);
    if kind == LibraryFile::New {
        return Ok(());
    }
    let attach_key = zeroize::Zeroizing::new(if kind == LibraryFile::Encrypted {
        format!("x'{}'", key.to_hex().as_str())
    } else {
        String::new()
    });
    conn.execute(
        "ATTACH DATABASE ?1 AS current KEY ?2",
        rusqlite::params![db.to_string_lossy(), attach_key.as_str()],
    )?;
    let has_keys: i64 = conn.query_row(
        "SELECT count(*) FROM current.sqlite_master WHERE type = 'table' AND name = 'api_keys'",
        [],
        |row| row.get(0),
    )?;
    if has_keys > 0 {
        conn.execute_batch(&format!(
            "BEGIN;
             DELETE FROM main.api_keys;
             INSERT INTO main.api_keys ({API_KEY_COLUMNS})
                 SELECT {API_KEY_COLUMNS} FROM current.api_keys;
             COMMIT;"
        ))?;
    }
    conn.execute_batch("DETACH DATABASE current")?;
    Ok(())
}

/// The staged copy is the one thing that must be sound.
fn check_staged(staged: &Path, key: &LibraryKey) -> Result<()> {
    let conn = open_encrypted(staged, key)?;
    let check: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if check != "ok" {
        return Err(StoreError::Invalid(format!(
            "the restored copy failed its check ({check})"
        )));
    }
    Ok(())
}

/// Where the library is kept: a manual backup made at `now_ms`, or a
/// millisecond later while that name is taken, so no backup is replaced.
fn kept_path(keep_dir: &Path, now_ms: i64) -> PathBuf {
    let mut at = now_ms;
    loop {
        let path = crate::backup::manual_backup_path(keep_dir, at);
        if !path.exists() && !with_suffix(&path, ".tmp").exists() {
            return path;
        }
        at = at.saturating_add(1);
    }
}

/// Keeps the library at `db` in `keep_dir` as a backup, leaving `db` in
/// place: its write-ahead log goes into it first, then it gets a second
/// name there (a hard link, which costs no space), or, across disks, a copy
/// that is finished and flushed before it takes a backup's name.
fn keep_library(db: &Path, key: &LibraryKey, keep_dir: &Path, now_ms: i64) -> Result<()> {
    std::fs::create_dir_all(keep_dir)?;
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
    // The log is empty now; nothing in it may be replayed into the restored
    // library. The library itself holds everything.
    for suffix in SIDECARS {
        shred(&with_suffix(db, suffix))?;
    }
    let kept = kept_path(keep_dir, now_ms);
    if std::fs::hard_link(db, &kept).is_err() {
        let temp = with_suffix(&kept, ".tmp");
        std::fs::copy(db, &temp)?;
        sync_file(&temp)?;
        std::fs::rename(&temp, &kept)?;
    }
    sync_dir(&kept);
    Ok(())
}
