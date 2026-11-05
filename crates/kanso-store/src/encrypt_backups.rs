//! Encrypting the backups a plain library made (issue #243 step 10): each
//! plain backup is copied to an encrypted file, checked, and swapped in, and
//! its plaintext is overwritten and deleted.

use std::path::{Path, PathBuf};

use crate::encrypt::{
    export, map_full, open_plain, shred, sync_dir, sync_file, verify, with_suffix,
};
use crate::{
    BackupsReport, EncryptOptions, EncryptStage, LibraryFile, LibraryKey, Result, StoreError,
    backup_name,
};

fn is_backup(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| backup_name(name).is_some())
}

/// What a crash left beside the backups: a partial copy is deleted, and the
/// plaintext kept while a backup was swapped is overwritten and deleted once
/// its backup is the encrypted one (before that, it is the same file as the
/// backup, and only its name is removed).
fn sweep(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if let Some(stem) = name.strip_suffix(".enc")
            && is_backup(&dir.join(stem))
        {
            let _ = std::fs::remove_file(&path);
        } else if let Some(stem) = name.strip_suffix(".old")
            && is_backup(&dir.join(stem))
        {
            if LibraryFile::of(&dir.join(stem)) == LibraryFile::Encrypted {
                let _ = shred(&path);
            } else {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
}

/// Encrypts every plain backup in `dir` with `key`. A backup that can't be
/// is left as it was and named in the report; a disk too small stops the
/// rest. Backups that are encrypted already are left alone, so this can run
/// at every launch.
pub fn encrypt_backups(
    dir: &Path,
    key: &LibraryKey,
    opts: &EncryptOptions,
) -> Result<BackupsReport> {
    let mut report = BackupsReport::default();
    if !dir.is_dir() {
        return Ok(report);
    }
    sweep(dir);
    let mut plain: Vec<PathBuf> = std::fs::read_dir(dir)?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| is_backup(path) && LibraryFile::of(path) == LibraryFile::Plain)
        .collect();
    plain.sort();
    let total = plain.len().max(1) as f32;
    for (index, path) in plain.iter().enumerate() {
        let from = index as f32 / total;
        let to = (index + 1) as f32 / total;
        if let Some(progress) = opts.progress {
            progress(crate::EncryptProgress {
                stage: EncryptStage::Backups,
                fraction: from,
            });
        }
        match encrypt_one(path, key, opts, from, to) {
            Ok(()) => report.converted += 1,
            Err(err @ StoreError::DiskFull { .. }) => {
                report.failed.push((path.clone(), err.to_string()));
                break;
            }
            Err(err) => report.failed.push((path.clone(), err.to_string())),
        }
    }
    Ok(report)
}

fn encrypt_one(
    path: &Path,
    key: &LibraryKey,
    opts: &EncryptOptions,
    from: f32,
    to: f32,
) -> Result<()> {
    let copy = with_suffix(path, ".enc");
    let old = with_suffix(path, ".old");
    let needed = std::fs::metadata(path).map(|meta| meta.len()).unwrap_or(0);
    let made = (|| {
        let _ = std::fs::remove_file(&copy);
        let src = open_plain(path)?;
        export(&src, &copy, key, opts.max_copy_bytes)?;
        verify(&src, &copy, key, opts, from, to)?;
        sync_file(&copy)?;
        Ok::<(), StoreError>(())
    })();
    if let Err(err) = made {
        let _ = std::fs::remove_file(&copy);
        for side in [with_suffix(&copy, "-journal"), with_suffix(&copy, "-wal")] {
            let _ = std::fs::remove_file(side);
        }
        return Err(map_full(err, needed));
    }
    // A second name for the plaintext, so it can be overwritten after the
    // swap. Without one (a disk that can't link), it is simply deleted.
    let linked = std::fs::hard_link(path, &old).is_ok();
    let swapped = std::fs::rename(&copy, path);
    sync_dir(path);
    if let Err(err) = swapped {
        let _ = std::fs::remove_file(&copy);
        let _ = std::fs::remove_file(&old);
        return Err(err.into());
    }
    if linked {
        shred(&old)?;
    }
    Ok(())
}
