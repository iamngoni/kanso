//! Turning a plain library into an encrypted one (issue #243 step 10).
//!
//! The library is copied, never changed in place:
//!
//! 1. A marker, `<db>.encrypting`, says a copy is being made.
//! 2. `sqlcipher_export` writes `<db>.enc`, keyed.
//! 3. The copy is opened with the key and compared with the library: the
//!    integrity check, the schema, and every row of every table.
//! 4. The marker says the copy is good. The plain library's write-ahead log
//!    goes, the plain library is kept under a second name (`<db>.plain`, a
//!    hard link, so it costs no space), and the copy is renamed over
//!    `<db>`. A rename is atomic, so `<db>` is always one library or the
//!    other, never missing and never both.
//! 5. The new library is opened and compared with the plain one again. Only
//!    then is the marker moved to its last phase and the plaintext
//!    overwritten and deleted. If the check fails, the plain library is put
//!    back.
//!
//! A crash anywhere leaves a marker. The next launch reads it and either
//! rolls back (before the copy is checked) or goes on from where it was
//! (after). [`Store::open`] refuses while a conversion is unfinished, so no
//! program opens the library halfway.
//!
//! The caller holds the data-folder lock: [`lock_data_dir`](crate::lock_data_dir).

use std::collections::BTreeMap;
use std::hash::{Hash as _, Hasher as _};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use rusqlite::types::ValueRef;
use rusqlite::{Connection, ErrorCode};

use crate::encrypt_model::{Marker, Phase};
use crate::{
    EncryptOptions, EncryptProgress, EncryptStage, EncryptStep, Encrypted, LibraryFile, LibraryKey,
    Result, StoreError, schema,
};

/// SQLCipher's page size, for turning a size cap into pages.
const PAGE_BYTES: u64 = 4096;

pub(crate) fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

fn marker_path(db: &Path) -> PathBuf {
    with_suffix(db, ".encrypting")
}

fn copy_path(db: &Path) -> PathBuf {
    with_suffix(db, ".enc")
}

/// Where the plain library is kept until the encrypted one is confirmed.
fn safety_path(db: &Path) -> PathBuf {
    with_suffix(db, ".plain")
}

/// A database file's side files.
fn sidecars(path: &Path) -> [PathBuf; 3] {
    [
        with_suffix(path, "-wal"),
        with_suffix(path, "-shm"),
        with_suffix(path, "-journal"),
    ]
}

/// Whether a conversion of the library at `db` is unfinished in a way that
/// keeps it from opening: the copy is being made, or the swap has not been
/// confirmed. (Once it is confirmed only plaintext waits to be deleted, and
/// the library opens.)
pub fn encryption_unfinished(db: &Path) -> bool {
    match std::fs::read(marker_path(db)) {
        Ok(bytes) => serde_json::from_slice::<Marker>(&bytes)
            .map(|marker| marker.phase != Phase::Cleanup)
            .unwrap_or(true),
        Err(_) => false,
    }
}

fn step(opts: &EncryptOptions, step: EncryptStep) {
    if let Some(hook) = opts.after_step {
        hook(step);
    }
}

fn report(opts: &EncryptOptions, stage: EncryptStage, fraction: f32) {
    if let Some(progress) = opts.progress {
        progress(EncryptProgress { stage, fraction });
    }
}

pub(crate) fn sync_dir(path: &Path) {
    #[cfg(unix)]
    if let Some(dir) = path.parent()
        && let Ok(dir) = std::fs::File::open(dir)
    {
        let _ = dir.sync_all();
    }
    #[cfg(not(unix))]
    let _ = path;
}

pub(crate) fn sync_file(path: &Path) -> std::io::Result<()> {
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)?
        .sync_all()
}

fn write_marker(db: &Path, phase: Phase) -> std::io::Result<()> {
    let path = marker_path(db);
    let temp = with_suffix(&path, ".tmp");
    {
        let mut file = std::fs::File::create(&temp)?;
        file.write_all(&serde_json::to_vec(&Marker { v: 1, phase }).expect("a marker is JSON"))?;
        file.sync_all()?;
    }
    std::fs::rename(&temp, &path)?;
    sync_dir(&path);
    Ok(())
}

/// Overwrites `path` with random bytes, flushes, shortens it and deletes it.
/// A file that is not there is fine. The disk may keep the old blocks anyway
/// (a flash drive or APFS remaps them); the full-disk encryption the system
/// offers covers that.
pub(crate) fn shred(path: &Path) -> std::io::Result<()> {
    let len = match std::fs::metadata(path) {
        Ok(meta) if meta.is_file() => meta.len(),
        Ok(_) => return Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err),
    };
    {
        let mut file = std::fs::OpenOptions::new().write(true).open(path)?;
        let mut chunk = vec![0u8; 64 * 1024];
        let mut left = len;
        while left > 0 {
            let n = left.min(chunk.len() as u64) as usize;
            getrandom::fill(&mut chunk[..n]).expect("the system's random source");
            file.write_all(&chunk[..n])?;
            left -= n as u64;
        }
        file.sync_all()?;
        file.set_len(0)?;
        file.sync_all()?;
    }
    std::fs::remove_file(path)
}

fn remove(path: &Path) {
    let _ = std::fs::remove_file(path);
}

fn remove_copy(db: &Path) {
    let copy = copy_path(db);
    remove(&copy);
    for side in sidecars(&copy) {
        remove(&side);
    }
}

/// Maps a full disk to the plain error, whatever layer noticed.
pub(crate) fn map_full(err: StoreError, needed: u64) -> StoreError {
    let full = match &err {
        // `sqlcipher_export` reports a full copy as a plain error with the
        // message, not as SQLITE_FULL.
        StoreError::Sqlite(err) => {
            err.sqlite_error_code() == Some(ErrorCode::DiskFull)
                || err.to_string().contains("disk is full")
        }
        StoreError::Io(err) => matches!(
            err.kind(),
            std::io::ErrorKind::StorageFull | std::io::ErrorKind::QuotaExceeded
        ),
        _ => false,
    };
    if full {
        StoreError::DiskFull {
            needed_mb: needed.div_ceil(1_048_576).max(1),
        }
    } else {
        err
    }
}

fn file_len(path: &Path) -> u64 {
    std::fs::metadata(path).map(|meta| meta.len()).unwrap_or(0)
}

/// The library's size on disk, and so roughly what its copy needs.
fn library_bytes(db: &Path) -> u64 {
    file_len(db) + file_len(&with_suffix(db, "-wal"))
}

pub(crate) fn open_plain(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    schema::register_functions(&conn)?;
    conn.busy_timeout(Duration::from_secs(5))?;
    Ok(conn)
}

pub(crate) fn open_encrypted(path: &Path, key: &LibraryKey) -> Result<Connection> {
    let conn = Connection::open(path)?;
    crate::library_key::unlock(&conn, Some(key))?;
    schema::register_functions(&conn)?;
    conn.busy_timeout(Duration::from_secs(5))?;
    Ok(conn)
}

/// Writes a keyed copy of `src`'s main database to `copy`.
pub(crate) fn export(
    src: &Connection,
    copy: &Path,
    key: &LibraryKey,
    max_copy_bytes: Option<u64>,
) -> Result<()> {
    let version: i64 = src.pragma_query_value(None, "user_version", |row| row.get(0))?;
    let hex = key.to_hex();
    src.execute(
        "ATTACH DATABASE ?1 AS enc KEY ?2",
        rusqlite::params![
            copy.to_string_lossy(),
            format!("x'{}'", hex.as_str()).as_str()
        ],
    )?;
    let copied = (|| -> Result<()> {
        if let Some(max) = max_copy_bytes {
            src.execute_batch(&format!(
                "PRAGMA enc.max_page_count = {}",
                (max / PAGE_BYTES).max(1)
            ))?;
        }
        src.execute_batch("SELECT sqlcipher_export('enc')")?;
        // The export leaves out what is not a table's rows.
        src.execute_batch(&format!("PRAGMA enc.user_version = {version}"))?;
        Ok(())
    })();
    let detached = src.execute_batch("DETACH DATABASE enc");
    copied?;
    detached?;
    Ok(())
}

fn quote(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

/// What a database holds, for comparing a copy with its source.
struct Shape {
    user_version: i64,
    /// Every table, view, index and trigger: its kind and name.
    objects: Vec<(String, String)>,
    /// Table name -> whether it is virtual.
    tables: BTreeMap<String, bool>,
}

fn shape(conn: &Connection) -> Result<Shape> {
    let user_version = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    let mut objects = Vec::new();
    let mut tables = BTreeMap::new();
    let mut stmt = conn.prepare(
        "SELECT type, name, sql FROM sqlite_master WHERE name NOT LIKE 'sqlite\\_%' ESCAPE '\\'",
    )?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let kind: String = row.get(0)?;
        let name: String = row.get(1)?;
        let sql: Option<String> = row.get(2)?;
        if kind == "table" {
            let is_virtual = sql.as_deref().is_some_and(|sql| {
                sql.trim_start()
                    .to_ascii_uppercase()
                    .starts_with("CREATE VIRTUAL")
            });
            tables.insert(name.clone(), is_virtual);
        }
        objects.push((kind, name));
    }
    objects.sort();
    Ok(Shape {
        user_version,
        objects,
        tables,
    })
}

/// A table's row count and a hash of every value in it, in the order the
/// table is stored.
fn digest(conn: &Connection, table: &str, is_virtual: bool) -> Result<(i64, u64)> {
    let count: i64 = conn.query_row(
        &format!("SELECT count(*) FROM {}", quote(table)),
        [],
        |row| row.get(0),
    )?;
    if is_virtual {
        return Ok((count, 0));
    }
    let mut hasher = std::hash::DefaultHasher::new();
    let mut stmt = conn.prepare(&format!("SELECT * FROM {} NOT INDEXED", quote(table)))?;
    let columns = stmt.column_count();
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        for column in 0..columns {
            match row.get_ref(column)? {
                ValueRef::Null => 0u8.hash(&mut hasher),
                ValueRef::Integer(n) => {
                    1u8.hash(&mut hasher);
                    n.hash(&mut hasher);
                }
                ValueRef::Real(f) => {
                    2u8.hash(&mut hasher);
                    f.to_bits().hash(&mut hasher);
                }
                ValueRef::Text(text) => {
                    3u8.hash(&mut hasher);
                    text.hash(&mut hasher);
                }
                ValueRef::Blob(blob) => {
                    4u8.hash(&mut hasher);
                    blob.hash(&mut hasher);
                }
            }
        }
    }
    Ok((count, hasher.finish()))
}

fn mismatch(what: impl std::fmt::Display) -> StoreError {
    StoreError::EncryptFailed(format!(
        "the encrypted copy doesn't match the library ({what})"
    ))
}

/// Opens `copy` with `key` and proves it holds what `src` holds: the
/// integrity check passes, the schema and `user_version` are the same, and
/// every table has the same rows. `from..to` is the share of the progress
/// this check covers.
pub(crate) fn verify(
    src: &Connection,
    copy: &Path,
    key: &LibraryKey,
    opts: &EncryptOptions,
    from: f32,
    to: f32,
) -> Result<()> {
    let copy_conn = open_encrypted(copy, key)?;
    let check: String = copy_conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if check != "ok" {
        return Err(mismatch(format!("integrity check: {check}")));
    }
    let want = shape(src)?;
    let got = shape(&copy_conn)?;
    if want.user_version != got.user_version {
        return Err(mismatch("version"));
    }
    if want.objects != got.objects {
        return Err(mismatch("tables and indexes"));
    }
    let total = want.tables.len().max(1) as f32;
    for (index, (table, is_virtual)) in want.tables.iter().enumerate() {
        if digest(src, table, *is_virtual)? != digest(&copy_conn, table, *is_virtual)? {
            return Err(mismatch(format!("rows of {table}")));
        }
        report(
            opts,
            EncryptStage::Checking,
            from + (to - from) * (index + 1) as f32 / total,
        );
    }
    Ok(())
}

/// Encrypts the plain library at `db` with `key`, or finishes (or undoes)
/// a conversion a crash interrupted, then encrypts it. Nothing else may have
/// the library open; the caller holds the data-folder lock.
///
/// A failure leaves the library as it was, plain and openable, and its
/// error says why in words a person can act on ([`StoreError::DiskFull`]
/// for a disk too small).
pub fn encrypt_library(db: &Path, key: &LibraryKey, opts: &EncryptOptions) -> Result<Encrypted> {
    report(opts, EncryptStage::Preparing, 0.0);
    let resumed = recover(db, Some(key), opts)?;
    match LibraryFile::of(db) {
        LibraryFile::Encrypted => Ok(if resumed {
            Encrypted::Resumed
        } else {
            Encrypted::AlreadyEncrypted
        }),
        LibraryFile::New => Err(StoreError::EncryptFailed(
            "there is no library here to encrypt".into(),
        )),
        LibraryFile::Plain => {
            let needed = library_bytes(db);
            match convert(db, key, opts) {
                Ok(()) => Ok(Encrypted::Converted),
                Err(err) => {
                    abandon(db);
                    Err(map_full(err, needed))
                }
            }
        }
    }
}

/// Finishes or undoes a conversion a crash interrupted, without encrypting
/// anything new. `key` is needed to check a finished swap; without it, one
/// that has not been confirmed is undone (the plain library is put back and
/// the error is [`StoreError::WrongKey`]). Says whether one was finished.
pub fn recover_encryption(
    db: &Path,
    key: Option<&LibraryKey>,
    opts: &EncryptOptions,
) -> Result<bool> {
    recover(db, key, opts)
}

/// The steps from a plain library to an encrypted one.
fn convert(db: &Path, key: &LibraryKey, opts: &EncryptOptions) -> Result<()> {
    let copy = copy_path(db);
    let src = open_plain(db)?;
    // The write-ahead log goes into the library, so the copy and the
    // library the checks read are one thing.
    let busy: i64 = src.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| row.get(0))?;
    if busy != 0 {
        return Err(StoreError::EncryptFailed(
            "the library is in use by another program".into(),
        ));
    }
    remove_copy(db);
    write_marker(db, Phase::Exporting)?;
    step(opts, EncryptStep::MarkerWritten);
    report(opts, EncryptStage::Copying, 0.02);

    // The copy grows to about the library's size; its size drives the bar.
    let expected = file_len(db).max(1);
    let stop = AtomicBool::new(false);
    let exported = std::thread::scope(|scope| {
        scope.spawn(|| {
            while !stop.load(Ordering::Relaxed) {
                let share = (file_len(&copy) as f32 / expected as f32).min(1.0);
                report(opts, EncryptStage::Copying, 0.02 + 0.68 * share);
                std::thread::sleep(Duration::from_millis(100));
            }
        });
        let result = export(&src, &copy, key, opts.max_copy_bytes);
        stop.store(true, Ordering::Relaxed);
        result
    });
    exported?;
    step(opts, EncryptStep::Exported);

    verify(&src, &copy, key, opts, 0.70, 0.93)?;
    sync_file(&copy)?;
    sync_dir(&copy);
    step(opts, EncryptStep::Verified);
    // Closing the last connection to a library in WAL mode empties and
    // deletes its log.
    drop(src);

    write_marker(db, Phase::Ready)?;
    step(opts, EncryptStep::ReadyMarked);
    finish_swap(db, key, opts)
}

/// From a checked copy and a plain library to the encrypted library, then
/// its confirmation. The marker says `Ready`.
fn finish_swap(db: &Path, key: &LibraryKey, opts: &EncryptOptions) -> Result<()> {
    let copy = copy_path(db);
    let safety = safety_path(db);
    // A log left beside the new library would be replayed into it. It must
    // be empty: the checkpoint emptied it, and nothing has written since.
    let wal = with_suffix(db, "-wal");
    if file_len(&wal) > 0 {
        return Err(StoreError::EncryptFailed(
            "the library has changes that are not saved into it yet".into(),
        ));
    }
    for side in sidecars(db) {
        shred(&side)?;
    }
    sync_dir(db);
    step(opts, EncryptStep::WalRemoved);

    if !safety.exists() {
        if std::fs::hard_link(db, &safety).is_err() {
            std::fs::copy(db, &safety)?;
            sync_file(&safety)?;
        }
        sync_dir(&safety);
    }
    step(opts, EncryptStep::SafetyLinked);

    std::fs::rename(&copy, db)?;
    sync_dir(db);
    step(opts, EncryptStep::Swapped);
    confirm_and_clean(db, Some(key), opts)
}

/// With the encrypted library in place: opens it and compares it with the
/// plain one it replaced (when that is still there), then deletes the
/// plaintext. A library that does not match is swapped back.
fn confirm_and_clean(db: &Path, key: Option<&LibraryKey>, opts: &EncryptOptions) -> Result<()> {
    let safety = safety_path(db);
    let phase = read_phase(db);
    if phase != Some(Phase::Cleanup) {
        let confirmed = match (key, safety.exists()) {
            (Some(key), true) => open_plain(&safety).and_then(|plain| {
                let result = verify(&plain, db, key, opts, 0.93, 0.98);
                drop(plain);
                result
            }),
            (Some(key), false) => open_encrypted(db, key).map(|_| ()),
            (None, _) => Err(StoreError::WrongKey),
        };
        if let Err(err) = confirmed {
            if safety.exists() {
                swap_back(db)?;
            }
            return Err(err);
        }
        write_marker(db, Phase::Cleanup)?;
        step(opts, EncryptStep::Confirmed);
    }
    report(opts, EncryptStage::Finishing, 0.98);
    clean_up(db, opts);
    report(opts, EncryptStage::Finishing, 1.0);
    Ok(())
}

/// Deletes the plaintext and the marker. A file that will not delete stays
/// named in the marker's phase and is tried at the next launch; the library
/// is already good.
fn clean_up(db: &Path, opts: &EncryptOptions) {
    let safety = safety_path(db);
    let mut clean = shred(&safety).is_ok();
    for side in sidecars(&safety) {
        clean &= shred(&side).is_ok();
    }
    remove_copy(db);
    if !clean {
        eprintln!(
            "kanso: couldn't delete {}; it will be tried again at the next launch",
            safety.display()
        );
        return;
    }
    step(opts, EncryptStep::PlainShredded);
    remove(&marker_path(db));
    remove(&with_suffix(&marker_path(db), ".tmp"));
    sync_dir(db);
    step(opts, EncryptStep::Finished);
}

/// Puts the kept plain library back over a bad encrypted one.
fn swap_back(db: &Path) -> Result<()> {
    std::fs::rename(safety_path(db), db)?;
    sync_dir(db);
    remove(&marker_path(db));
    sync_dir(db);
    Ok(())
}

/// After a failure that was not a crash: leaves the library plain and
/// openable, with nothing of the attempt left.
fn abandon(db: &Path) {
    match (read_phase(db), LibraryFile::of(db)) {
        (Some(Phase::Exporting | Phase::Ready), LibraryFile::Plain) => {
            remove_copy(db);
            // A hard link to the library itself; the library stays.
            remove(&safety_path(db));
            remove(&marker_path(db));
            remove(&with_suffix(&marker_path(db), ".tmp"));
            sync_dir(db);
        }
        (Some(Phase::Ready), LibraryFile::Encrypted) if safety_path(db).exists() => {
            let _ = swap_back(db);
        }
        _ => {}
    }
}

fn read_phase(db: &Path) -> Option<Phase> {
    let bytes = std::fs::read(marker_path(db)).ok()?;
    Some(
        serde_json::from_slice::<Marker>(&bytes)
            .map(|marker| marker.phase)
            // A marker that can't be read: if the library is still plain
            // nothing was swapped, and if not, treat the swap as unconfirmed.
            .unwrap_or(if LibraryFile::of(db) == LibraryFile::Plain {
                Phase::Exporting
            } else {
                Phase::Ready
            }),
    )
}

/// Finishes or undoes a conversion a crash interrupted. Says whether one
/// was finished.
fn recover(db: &Path, key: Option<&LibraryKey>, opts: &EncryptOptions) -> Result<bool> {
    let Some(phase) = read_phase(db) else {
        // Without a marker only the copy can be left, from before it was
        // written; it is encrypted or partial, and is not the library.
        remove_copy(db);
        return Ok(false);
    };
    let file = LibraryFile::of(db);
    match (phase, file) {
        (Phase::Exporting, LibraryFile::Plain) => {
            abandon(db);
            Ok(false)
        }
        (Phase::Ready, LibraryFile::Plain) => {
            // The copy was checked, and the swap had not happened. Check
            // again and go on, or start over.
            let resume = key.is_some_and(|key| {
                copy_path(db).exists()
                    && open_plain(db).is_ok_and(|plain| {
                        let ok = verify(&plain, &copy_path(db), key, opts, 0.70, 0.93).is_ok();
                        drop(plain);
                        ok
                    })
            });
            match (resume, key) {
                (true, Some(key)) => {
                    finish_swap(db, key, opts)?;
                    Ok(true)
                }
                _ => {
                    abandon(db);
                    Ok(false)
                }
            }
        }
        (Phase::Ready | Phase::Cleanup, LibraryFile::Encrypted) => {
            confirm_and_clean(db, key, opts)?;
            Ok(true)
        }
        (phase, file) => Err(StoreError::EncryptFailed(format!(
            "an unfinished encryption of {} is in a state Kanso doesn't expect ({phase:?}, {file:?}); nothing was changed",
            db.display()
        ))),
    }
}
