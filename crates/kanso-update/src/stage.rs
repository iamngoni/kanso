//! Downloading an update, checking it against the signed feed, and
//! unpacking it beside the install, ready to swap in.
//!
//! The staging folder sits next to what it replaces (on the same volume, so
//! the swap is two renames). It is made afresh for every download, readable
//! only by its owner, and holds nothing but the unpacked update.
//!
//! Nothing on disk is trusted later: what [`prepare`] staged is described by
//! the [`Pending`] it returns, which carries a [`Seal`] (a digest of the
//! unpacked update and the staging folder's identity) that only lives in
//! memory. [`crate::install::apply`] checks the seal before swapping, so an
//! update planted or altered in the staging folder is never installed.

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::feed::Update;
use crate::fetch::{self, FetchError, Transport};
use crate::install::Install;
use crate::platform::{ArchiveKind, Platform};
use crate::policy::UrlPolicy;

/// The most an archive may unpack to: well above Kanso's size, well below
/// a disk.
const MAX_UNPACKED_BYTES: u64 = 2 * 1024 * 1024 * 1024;

/// What an unpack may produce: total bytes and entries. An app update takes
/// the generous defaults; the theme gallery passes its own, much smaller,
/// limits so a bundle cannot fill the disk or the inode table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExtractLimits {
    pub max_bytes: u64,
    pub max_entries: usize,
}

impl ExtractLimits {
    /// The limits a Kanso app update is held to.
    pub const UPDATER: ExtractLimits = ExtractLimits {
        max_bytes: MAX_UNPACKED_BYTES,
        max_entries: usize::MAX,
    };
}

#[derive(Debug, thiserror::Error)]
pub enum StageError {
    #[error(transparent)]
    Fetch(#[from] FetchError),
    #[error("the download's size doesn't match the release")]
    SizeMismatch,
    #[error("the download's checksum doesn't match the release")]
    ChecksumMismatch,
    #[error("there isn't enough free space for the update")]
    NoSpace,
    #[error("the update's archive isn't as expected: {0}")]
    BadArchive(String),
    #[error("the update's signature check failed: {0}")]
    CodeSignature(String),
    #[error("stopped")]
    Cancelled,
    #[error("couldn't write the update: {0}")]
    Io(io::Error),
}

impl From<io::Error> for StageError {
    fn from(err: io::Error) -> StageError {
        if is_no_space(&err) {
            StageError::NoSpace
        } else {
            StageError::Io(err)
        }
    }
}

/// Whether an I/O error means the disk is full.
pub fn is_no_space(err: &io::Error) -> bool {
    matches!(
        err.kind(),
        io::ErrorKind::StorageFull | io::ErrorKind::QuotaExceeded
    )
}

/// A staged update: what `install::apply` swaps in.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pending {
    pub version: String,
    /// The unpacked `.app` or executable, in the staging folder.
    pub payload: PathBuf,
    /// What it replaces.
    pub target: PathBuf,
    pub pub_date: String,
    pub release_url: String,
    pub notes: String,
    /// The release's checked What's new video, if any.
    #[serde(default)]
    pub whats_new: Option<crate::feed::Asset>,
    #[serde(default)]
    pub earlier: Vec<crate::feed::Earlier>,
    /// What was checked when it was staged. Never written to disk: a
    /// `Pending` read back from anywhere has an empty seal and is refused.
    #[serde(skip)]
    pub seal: Seal,
}

/// What [`prepare`] saw when it staged an update, checked again by
/// [`crate::install::apply`] just before the swap.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Seal {
    /// [`payload_digest`] of the payload, taken right after unpacking and
    /// before the code-signature check.
    pub(crate) digest: Option<String>,
    /// [`payload_digest`] of the plugin host staged beside the payload,
    /// taken at the same time; `None` when the update has no host. Only a
    /// host with a digest here is ever swapped in.
    pub(crate) host_digest: Option<String>,
    /// The staging folder's device and inode, where the system has them.
    pub(crate) stage_id: Option<(u64, u64)>,
}

impl Seal {
    /// Whether this seal came from [`prepare`].
    pub fn is_set(&self) -> bool {
        self.digest.is_some()
    }
}

/// Downloads `update` into `install`'s staging folder, checks it, unpacks
/// it, runs `verify` on what will be swapped in (the code signature on
/// macOS), and records it as pending. `progress` gets the bytes so far and
/// the total.
pub fn prepare(
    transport: &dyn Transport,
    policy: &UrlPolicy,
    update: &Update,
    install: &Install,
    progress: &mut dyn FnMut(u64, u64),
    cancel: &AtomicBool,
    verify: &dyn Fn(&Path) -> Result<(), String>,
) -> Result<Pending, StageError> {
    let dir = &install.stage_dir;
    fresh_stage_dir(dir)?;
    let stage_id = dir_identity(dir)?;
    let archive = dir.join(install.platform.asset_name(&update.version));
    download(transport, policy, &update.asset, &archive, progress, cancel)?;
    let unpacked = dir.join(format!("unpacked-{}", update.version));
    let result = (|| {
        extract(&archive, install.platform.archive(), &unpacked)?;
        let payload = find_payload(&unpacked, install.platform)?;
        // The digest first: whatever the signature check then sees, only
        // these bytes are ever swapped in.
        let digest = payload_digest(&payload)?;
        let host_digest = match find_host(&payload, install.platform)? {
            Some(host) => Some(payload_digest(&host)?),
            None => None,
        };
        verify(&payload).map_err(StageError::CodeSignature)?;
        Ok((payload, digest, host_digest))
    })();
    fs::remove_file(&archive).ok();
    let (payload, digest, host_digest) = match result {
        Ok(found) => found,
        Err(err) => {
            fs::remove_dir_all(&unpacked).ok();
            return Err(err);
        }
    };
    let pending = Pending {
        version: update.version.clone(),
        payload,
        target: install.target.clone(),
        pub_date: update.pub_date.clone(),
        release_url: update.release_url.clone(),
        notes: update.notes.clone(),
        whats_new: update.whats_new.clone(),
        earlier: update.earlier.clone(),
        seal: Seal {
            digest: Some(digest),
            host_digest,
            stage_id,
        },
    };
    Ok(pending)
}

/// Removes whatever is at `path` without following a link there: a link
/// goes, not what it points at.
pub fn remove_entry(path: &Path) -> io::Result<()> {
    let result = match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_dir() => fs::remove_dir_all(path),
        Ok(_) => fs::remove_file(path),
        Err(err) => Err(err),
    };
    match result {
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

/// Makes `dir` anew: whatever was there (a leftover, a link someone put
/// there, a folder someone else owns) is removed, never followed, and the
/// folder is created by this process, readable only by its owner. Fails
/// rather than reuse a folder that appears in between.
pub fn fresh_stage_dir(dir: &Path) -> io::Result<()> {
    remove_entry(dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt as _;
        fs::DirBuilder::new().mode(0o700).create(dir)
    }
    #[cfg(not(unix))]
    fs::create_dir(dir)
}

/// The device and inode of the folder at `dir` (not of a link's target),
/// where the system has them; `None` elsewhere.
pub fn dir_identity(dir: &Path) -> io::Result<Option<(u64, u64)>> {
    let meta = fs::symlink_metadata(dir)?;
    if !meta.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "the staging folder isn't a folder",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt as _;
        Ok(Some((meta.dev(), meta.ino())))
    }
    #[cfg(not(unix))]
    Ok(None)
}

/// A SHA-256 over everything at `path`: each file's relative name, kind,
/// permissions (on Unix) and contents, in name order. Links, devices and
/// anything but plain files and folders are refused, as unpacking refuses
/// them.
pub fn payload_digest(path: &Path) -> io::Result<String> {
    let mut hasher = Sha256::new();
    digest_entry(path, Path::new(""), &mut hasher)?;
    Ok(hex(&hasher.finalize()))
}

fn digest_entry(path: &Path, rel: &Path, hasher: &mut Sha256) -> io::Result<()> {
    let meta = fs::symlink_metadata(path)?;
    let name = rel.as_os_str().as_encoded_bytes();
    hasher.update((name.len() as u64).to_le_bytes());
    hasher.update(name);
    #[cfg(unix)]
    let mode = {
        use std::os::unix::fs::PermissionsExt as _;
        meta.permissions().mode() & 0o7777
    };
    #[cfg(not(unix))]
    let mode = 0u32;
    if meta.is_dir() {
        hasher.update(b"D");
        let mut children: Vec<_> = fs::read_dir(path)?
            .map(|e| e.map(|e| e.file_name()))
            .collect::<io::Result<_>>()?;
        children.sort();
        hasher.update((children.len() as u64).to_le_bytes());
        for child in children {
            digest_entry(&path.join(&child), &rel.join(&child), hasher)?;
        }
    } else if meta.is_file() {
        hasher.update(b"F");
        hasher.update(mode.to_le_bytes());
        let mut file = File::open(path)?;
        let mut content = Sha256::new();
        let mut buf = vec![0u8; 64 * 1024];
        let mut n = 0u64;
        loop {
            let read = file.read(&mut buf)?;
            if read == 0 {
                break;
            }
            n += read as u64;
            content.update(&buf[..read]);
        }
        hasher.update(n.to_le_bytes());
        hasher.update(content.finalize());
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{} isn't a plain file", rel.display()),
        ));
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Streams `asset` into `to`, hashing as it goes, and keeps it only when
/// its size and SHA-256 are the feed's.
pub fn download(
    transport: &dyn Transport,
    policy: &UrlPolicy,
    asset: &crate::feed::Asset,
    to: &Path,
    progress: &mut dyn FnMut(u64, u64),
    cancel: &AtomicBool,
) -> Result<(), StageError> {
    let part = to.with_extension("part");
    let result = (|| {
        let mut body = fetch::open(transport, policy, &asset.url)?;
        let mut file = File::create(&part)?;
        let mut hasher = Sha256::new();
        let mut buf = vec![0u8; 64 * 1024];
        let mut total = 0u64;
        loop {
            if cancel.load(Ordering::Relaxed) {
                return Err(StageError::Cancelled);
            }
            let n = body.read(&mut buf).map_err(FetchError::Io)?;
            if n == 0 {
                break;
            }
            total += n as u64;
            if total > asset.size {
                return Err(StageError::SizeMismatch);
            }
            hasher.update(&buf[..n]);
            file.write_all(&buf[..n])?;
            progress(total, asset.size);
        }
        if total != asset.size {
            return Err(StageError::SizeMismatch);
        }
        file.sync_all()?;
        if hex(&hasher.finalize()) != asset.sha256 {
            return Err(StageError::ChecksumMismatch);
        }
        fs::rename(&part, to)?;
        Ok(())
    })();
    if result.is_err() {
        fs::remove_file(&part).ok();
    }
    result
}

/// Unpacks `archive` into `dest`, which must not exist yet. Only plain
/// files and folders are taken, each inside `dest`: links, devices and
/// paths that climb out are refused, and so is an archive that unpacks to
/// more than [`MAX_UNPACKED_BYTES`].
pub fn extract(archive: &Path, kind: ArchiveKind, dest: &Path) -> Result<(), StageError> {
    extract_limited(archive, kind, dest, ExtractLimits::UPDATER)
}

/// [`extract`] with the caller's limits, enforced while unpacking.
pub fn extract_limited(
    archive: &Path,
    kind: ArchiveKind,
    dest: &Path,
    limits: ExtractLimits,
) -> Result<(), StageError> {
    remove_entry(dest)?;
    fs::create_dir_all(dest)?;
    match kind {
        ArchiveKind::TarGz => extract_tar_gz(archive, dest, limits),
        ArchiveKind::Zip => extract_zip(archive, dest, limits),
    }
}

/// `name` as a path inside the destination: relative, with only normal
/// parts.
fn safe_relative(name: &Path) -> Option<PathBuf> {
    let mut out = PathBuf::new();
    for part in name.components() {
        match part {
            Component::Normal(p) => out.push(p),
            Component::CurDir => {}
            _ => return None,
        }
    }
    (!out.as_os_str().is_empty()).then_some(out)
}

fn extract_tar_gz(archive: &Path, dest: &Path, limits: ExtractLimits) -> Result<(), StageError> {
    let file = File::open(archive)?;
    let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(file));
    let mut total = 0u64;
    let mut count = 0usize;
    let entries = tar
        .entries()
        .map_err(|e| StageError::BadArchive(e.to_string()))?;
    for entry in entries {
        count += 1;
        if count > limits.max_entries {
            return Err(StageError::BadArchive("it holds too many entries".into()));
        }
        let mut entry = entry.map_err(|e| StageError::BadArchive(e.to_string()))?;
        let path = entry
            .path()
            .map_err(|e| StageError::BadArchive(e.to_string()))?
            .into_owned();
        let Some(rel) = safe_relative(&path) else {
            return Err(StageError::BadArchive(format!(
                "{} leaves the folder",
                path.display()
            )));
        };
        let out = dest.join(&rel);
        let kind = entry.header().entry_type();
        if kind.is_dir() {
            fs::create_dir_all(&out)?;
            continue;
        }
        // GNU and PAX headers that carry long names arrive as entries of
        // their own; `entries` folds them in, so anything else here is a
        // link or a device.
        if !kind.is_file() {
            return Err(StageError::BadArchive(format!(
                "{} isn't a plain file",
                rel.display()
            )));
        }
        total += entry.size();
        if total > limits.max_bytes {
            return Err(StageError::BadArchive("it unpacks too large".into()));
        }
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent)?;
        }
        let size = entry.size();
        let mut file = create_new_file(&out)?;
        io::copy(&mut (&mut entry).take(size), &mut file).map_err(copy_error)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let mode = entry.header().mode().unwrap_or(0o644) & 0o755;
            fs::set_permissions(&out, fs::Permissions::from_mode(mode))?;
        }
    }
    Ok(())
}

fn extract_zip(archive: &Path, dest: &Path, limits: ExtractLimits) -> Result<(), StageError> {
    let file = File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| StageError::BadArchive(e.to_string()))?;
    if zip.len() > limits.max_entries {
        return Err(StageError::BadArchive("it holds too many entries".into()));
    }
    let mut total = 0u64;
    for ix in 0..zip.len() {
        let mut entry = zip
            .by_index(ix)
            .map_err(|e| StageError::BadArchive(e.to_string()))?;
        let Some(rel) = entry.enclosed_name().and_then(|p| safe_relative(&p)) else {
            return Err(StageError::BadArchive(format!(
                "{} leaves the folder",
                entry.name()
            )));
        };
        let out = dest.join(&rel);
        if entry.is_dir() {
            fs::create_dir_all(&out)?;
            continue;
        }
        if entry.is_symlink() {
            return Err(StageError::BadArchive(format!(
                "{} isn't a plain file",
                rel.display()
            )));
        }
        total += entry.size();
        if total > limits.max_bytes {
            return Err(StageError::BadArchive("it unpacks too large".into()));
        }
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent)?;
        }
        // The declared size is what counts: a deflate stream that inflates
        // past it is refused before it can fill the disk.
        let size = entry.size();
        let mut file = create_new_file(&out)?;
        io::copy(&mut (&mut entry).take(size), &mut file).map_err(copy_error)?;
        let mut extra = [0u8; 1];
        if entry.read(&mut extra).map_err(copy_error)? != 0 {
            return Err(StageError::BadArchive(format!(
                "{} is larger than it says",
                rel.display()
            )));
        }
        #[cfg(unix)]
        if let Some(mode) = entry.unix_mode() {
            use std::os::unix::fs::PermissionsExt as _;
            fs::set_permissions(&out, fs::Permissions::from_mode(mode & 0o755))?;
        }
    }
    Ok(())
}

/// A new file at `out`, replacing a plain file an earlier entry of the
/// same name wrote; never following a link (unpacking makes none, but the
/// folder is checked rather than assumed).
fn create_new_file(out: &Path) -> Result<File, StageError> {
    if fs::symlink_metadata(out).is_ok_and(|m| m.is_file()) {
        fs::remove_file(out)?;
    }
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out)
        .map_err(|e| {
            if e.kind() == io::ErrorKind::AlreadyExists {
                StageError::BadArchive(format!("{} is there twice", out.display()))
            } else {
                e.into()
            }
        })
}

fn copy_error(e: io::Error) -> StageError {
    if is_no_space(&e) {
        StageError::NoSpace
    } else {
        StageError::BadArchive(e.to_string())
    }
}

/// What in an unpacked update replaces the install: the one `.app` on
/// macOS; `kanso` or `kanso.exe`, at the top or in the one folder there,
/// elsewhere. The plugin host beside it is found by [`find_host`].
pub fn find_payload(unpacked: &Path, platform: Platform) -> Result<PathBuf, StageError> {
    let top: Vec<PathBuf> = fs::read_dir(unpacked)?
        .flatten()
        .map(|e| e.path())
        .filter(|p| !is_hidden(p))
        .collect();
    let bad = |why: &str| StageError::BadArchive(why.to_owned());
    if platform.is_bundle() {
        let apps: Vec<&PathBuf> = top
            .iter()
            .filter(|p| p.is_dir() && p.extension().is_some_and(|e| e == "app"))
            .collect();
        let [app] = apps.as_slice() else {
            return Err(bad("expected one .app"));
        };
        if !app.join("Contents/MacOS/kanso").is_file() || !app.join("Contents/Info.plist").is_file()
        {
            return Err(bad("the .app has no Kanso inside"));
        }
        return Ok((*app).clone());
    }
    let exe = if platform == Platform::WindowsX64 {
        "kanso.exe"
    } else {
        "kanso"
    };
    let direct = unpacked.join(exe);
    if direct.is_file() {
        return Ok(direct);
    }
    if let [only] = top.as_slice() {
        let nested = only.join(exe);
        if only.is_dir() && nested.is_file() {
            return Ok(nested);
        }
    }
    Err(bad(&format!("no {exe} in it")))
}

/// The plugin host staged beside `payload` (`kanso-plugin-host` or
/// `kanso-plugin-host.exe`), if the update has one. Older archives have
/// none, and macOS keeps its host inside the `.app`, so both give `None`. A
/// host that is a folder, a link or anything but a plain file is refused.
pub fn find_host(payload: &Path, platform: Platform) -> Result<Option<PathBuf>, StageError> {
    if platform.is_bundle() {
        return Ok(None);
    }
    let host = payload.with_file_name(platform.host_name());
    match fs::symlink_metadata(&host) {
        Ok(meta) if meta.is_file() => Ok(Some(host)),
        Ok(_) => Err(StageError::BadArchive(format!(
            "{} isn't a plain file",
            platform.host_name()
        ))),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err.into()),
    }
}

/// Where the sealed plugin host is in the staging folder: beside the
/// payload, and only if the seal says [`prepare`] found one.
pub fn staged_host(pending: &Pending, platform: Platform) -> Option<PathBuf> {
    pending.seal.host_digest.as_ref()?;
    Some(pending.payload.with_file_name(platform.host_name()))
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with('.') || n == "__MACOSX")
}

#[cfg(test)]
pub(crate) mod archives {
    //! Builds update archives in tests.

    use std::io::Write as _;
    use std::path::Path;

    /// A gzipped tar of `files` (path, contents, mode).
    pub fn tar_gz(files: &[(&str, &[u8], u32)]) -> Vec<u8> {
        let mut builder = tar::Builder::new(flate2::write::GzEncoder::new(
            Vec::new(),
            flate2::Compression::fast(),
        ));
        for (path, bytes, mode) in files {
            let mut header = tar::Header::new_gnu();
            header.set_size(bytes.len() as u64);
            header.set_mode(*mode);
            header.set_cksum();
            builder.append_data(&mut header, path, *bytes).unwrap();
        }
        builder.into_inner().unwrap().finish().unwrap()
    }

    /// A tar whose one entry's raw name is `name`, bypassing the builder's
    /// own checks, as an attacker would write it.
    pub fn tar_gz_raw_name(name: &str, kind: tar::EntryType) -> Vec<u8> {
        let mut header = tar::Header::new_old();
        header.as_old_mut().name[..name.len()].copy_from_slice(name.as_bytes());
        header.set_entry_type(kind);
        header.set_size(if kind.is_file() { 1 } else { 0 });
        header.set_mode(0o644);
        if kind.is_symlink() {
            header.set_link_name("/etc/passwd").unwrap();
        }
        header.set_cksum();
        let mut raw = Vec::new();
        raw.extend_from_slice(header.as_bytes());
        if kind.is_file() {
            raw.push(b'x');
            raw.resize(1024, 0);
        }
        raw.resize(raw.len() + 1024, 0);
        let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        gz.write_all(&raw).unwrap();
        gz.finish().unwrap()
    }

    pub fn zip(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut out = std::io::Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut out);
            for (path, bytes) in files {
                zip.start_file(*path, zip::write::SimpleFileOptions::default())
                    .unwrap();
                zip.write_all(bytes).unwrap();
            }
            zip.finish().unwrap();
        }
        out.into_inner()
    }

    pub fn write(path: &Path, bytes: &[u8]) {
        std::fs::write(path, bytes).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::archives::*;
    use super::*;
    use crate::feed::Asset;
    use crate::fetch::fake::{Answer, FakeTransport};
    use proptest::prelude::*;

    const URL: &str = "https://github.com/iamngoni/kanso/releases/download/v0.2.0/Kanso-0.2.0-macos-arm64.app.tar.gz";

    fn sha(bytes: &[u8]) -> String {
        Sha256::digest(bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    fn fetch(body: &[u8], asset: &Asset, to: &Path) -> Result<(), StageError> {
        let t = FakeTransport::default().with(URL, Answer::Body(body.to_vec()));
        download(
            &t,
            &UrlPolicy::RELEASE,
            asset,
            to,
            &mut |_, _| {},
            &AtomicBool::new(false),
        )
    }

    #[test]
    fn a_matching_download_is_kept() {
        let dir = tempfile::tempdir().unwrap();
        let body = b"kanso update bytes";
        let asset = Asset {
            url: URL.into(),
            sha256: sha(body),
            size: body.len() as u64,
        };
        let to = dir.path().join("a.tar.gz");
        fetch(body, &asset, &to).unwrap();
        assert_eq!(fs::read(&to).unwrap(), body);
        assert!(!to.with_extension("part").exists());
    }

    #[test]
    fn a_wrong_size_or_checksum_is_thrown_away() {
        let dir = tempfile::tempdir().unwrap();
        let body = b"kanso update bytes";
        let to = dir.path().join("a.tar.gz");
        for (asset, expect) in [
            (
                Asset {
                    url: URL.into(),
                    sha256: sha(body),
                    size: body.len() as u64 + 1,
                },
                "size",
            ),
            (
                Asset {
                    url: URL.into(),
                    sha256: sha(body),
                    size: body.len() as u64 - 1,
                },
                "size",
            ),
            (
                Asset {
                    url: URL.into(),
                    sha256: sha(b"other"),
                    size: body.len() as u64,
                },
                "checksum",
            ),
        ] {
            let err = fetch(body, &asset, &to).unwrap_err();
            match expect {
                "size" => assert!(matches!(err, StageError::SizeMismatch), "{err:?}"),
                _ => assert!(matches!(err, StageError::ChecksumMismatch), "{err:?}"),
            }
            assert!(!to.exists() && !to.with_extension("part").exists());
        }
    }

    #[test]
    fn a_cancelled_download_stops() {
        let dir = tempfile::tempdir().unwrap();
        let body = b"bytes";
        let asset = Asset {
            url: URL.into(),
            sha256: sha(body),
            size: 5,
        };
        let t = FakeTransport::default().with(URL, Answer::Body(body.to_vec()));
        let err = download(
            &t,
            &UrlPolicy::RELEASE,
            &asset,
            &dir.path().join("a"),
            &mut |_, _| {},
            &AtomicBool::new(true),
        )
        .unwrap_err();
        assert!(matches!(err, StageError::Cancelled));
    }

    #[test]
    fn an_app_bundle_unpacks_with_its_modes() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("a.tar.gz");
        write(
            &archive,
            &tar_gz(&[
                ("Kanso.app/Contents/Info.plist", b"<plist/>", 0o644),
                ("Kanso.app/Contents/MacOS/kanso", b"#!/bin/sh\n", 0o755),
            ]),
        );
        let dest = dir.path().join("out");
        extract(&archive, ArchiveKind::TarGz, &dest).unwrap();
        let app = find_payload(&dest, Platform::MacosArm64).unwrap();
        assert_eq!(app, dest.join("Kanso.app"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let mode = fs::metadata(app.join("Contents/MacOS/kanso"))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o755);
        }
    }

    #[test]
    fn linux_and_windows_payloads_are_found() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("l.tar.gz");
        write(
            &archive,
            &tar_gz(&[
                ("Kanso-0.2.0-linux-x64/kanso", b"\x7fELF", 0o755),
                ("Kanso-0.2.0-linux-x64/install.sh", b"#!/bin/sh", 0o755),
            ]),
        );
        let dest = dir.path().join("l");
        extract(&archive, ArchiveKind::TarGz, &dest).unwrap();
        assert_eq!(
            find_payload(&dest, Platform::LinuxX64).unwrap(),
            dest.join("Kanso-0.2.0-linux-x64/kanso")
        );

        let archive = dir.path().join("w.zip");
        write(&archive, &zip(&[("kanso.exe", b"MZ")]));
        let dest = dir.path().join("w");
        extract(&archive, ArchiveKind::Zip, &dest).unwrap();
        assert_eq!(
            find_payload(&dest, Platform::WindowsX64).unwrap(),
            dest.join("kanso.exe")
        );
    }

    /// Unpacks `bytes` as a `kind` archive into a fresh folder.
    fn unpacked(dir: &Path, kind: ArchiveKind, bytes: &[u8]) -> PathBuf {
        let archive = dir.join("a.archive");
        write(&archive, bytes);
        let dest = dir.join("out");
        extract(&archive, kind, &dest).unwrap();
        dest
    }

    #[test]
    fn a_host_beside_kanso_is_found() {
        let dir = tempfile::tempdir().unwrap();
        let dest = unpacked(
            dir.path(),
            ArchiveKind::TarGz,
            &tar_gz(&[
                ("Kanso-0.2.0-linux-x64/kanso", b"\x7fELF", 0o755),
                ("Kanso-0.2.0-linux-x64/kanso-plugin-host", b"\x7fELF", 0o755),
            ]),
        );
        let payload = find_payload(&dest, Platform::LinuxX64).unwrap();
        assert_eq!(
            find_host(&payload, Platform::LinuxX64).unwrap(),
            Some(dest.join("Kanso-0.2.0-linux-x64/kanso-plugin-host"))
        );

        let dir = tempfile::tempdir().unwrap();
        let dest = unpacked(
            dir.path(),
            ArchiveKind::Zip,
            &zip(&[("kanso.exe", b"MZ"), ("kanso-plugin-host.exe", b"MZ")]),
        );
        let payload = find_payload(&dest, Platform::WindowsX64).unwrap();
        assert_eq!(
            find_host(&payload, Platform::WindowsX64).unwrap(),
            Some(dest.join("kanso-plugin-host.exe"))
        );
    }

    #[test]
    fn an_older_archive_has_no_host() {
        let dir = tempfile::tempdir().unwrap();
        let dest = unpacked(dir.path(), ArchiveKind::Zip, &zip(&[("kanso.exe", b"MZ")]));
        let payload = find_payload(&dest, Platform::WindowsX64).unwrap();
        assert_eq!(find_host(&payload, Platform::WindowsX64).unwrap(), None);
    }

    #[test]
    fn a_host_in_another_folder_is_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let dest = unpacked(
            dir.path(),
            ArchiveKind::TarGz,
            &tar_gz(&[
                ("kanso", b"\x7fELF", 0o755),
                ("bin/kanso-plugin-host", b"\x7fELF", 0o755),
                ("Other/kanso-plugin-host", b"\x7fELF", 0o755),
            ]),
        );
        let payload = find_payload(&dest, Platform::LinuxX64).unwrap();
        assert_eq!(find_host(&payload, Platform::LinuxX64).unwrap(), None);
    }

    #[test]
    fn a_host_that_is_a_folder_or_a_link_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let dest = unpacked(
            dir.path(),
            ArchiveKind::TarGz,
            &tar_gz(&[
                ("kanso", b"\x7fELF", 0o755),
                ("kanso-plugin-host/inner", b"x", 0o755),
            ]),
        );
        let payload = find_payload(&dest, Platform::LinuxX64).unwrap();
        assert!(matches!(
            find_host(&payload, Platform::LinuxX64),
            Err(StageError::BadArchive(_))
        ));
        #[cfg(unix)]
        {
            let dir = tempfile::tempdir().unwrap();
            let dest = unpacked(
                dir.path(),
                ArchiveKind::TarGz,
                &tar_gz(&[("kanso", b"\x7fELF", 0o755)]),
            );
            std::os::unix::fs::symlink("/bin/sh", dest.join("kanso-plugin-host")).unwrap();
            let payload = find_payload(&dest, Platform::LinuxX64).unwrap();
            assert!(matches!(
                find_host(&payload, Platform::LinuxX64),
                Err(StageError::BadArchive(_))
            ));
            // A link in an archive never even unpacks.
            let archive = dir.path().join("l.tar.gz");
            write(
                &archive,
                &tar_gz_raw_name("kanso-plugin-host", tar::EntryType::Symlink),
            );
            assert!(extract(&archive, ArchiveKind::TarGz, &dir.path().join("l")).is_err());
        }
    }

    #[test]
    fn a_host_never_matters_on_macos() {
        let dir = tempfile::tempdir().unwrap();
        let dest = unpacked(
            dir.path(),
            ArchiveKind::TarGz,
            &tar_gz(&[
                ("Kanso.app/Contents/Info.plist", b"<plist/>", 0o644),
                ("Kanso.app/Contents/MacOS/kanso", b"#!/bin/sh\n", 0o755),
                (
                    "Kanso.app/Contents/MacOS/kanso-plugin-host",
                    b"#!/bin/sh\n",
                    0o755,
                ),
                ("kanso-plugin-host", b"stray", 0o755),
            ]),
        );
        let app = find_payload(&dest, Platform::MacosArm64).unwrap();
        assert_eq!(app, dest.join("Kanso.app"));
        assert_eq!(find_host(&app, Platform::MacosArm64).unwrap(), None);
        // And with a folder where the host would be, nothing is said.
        fs::create_dir(dest.join("Kanso.app/kanso-plugin-host")).unwrap();
        assert_eq!(find_host(&app, Platform::MacosArm64).unwrap(), None);
    }

    /// Stages `archive` as `platform` would, with `kanso` named as it is.
    fn prepared(archive: Vec<u8>, platform: Platform, dir: &Path) -> Result<Pending, StageError> {
        let t = FakeTransport::default().with(URL, Answer::Body(archive.clone()));
        let asset = Asset {
            url: URL.into(),
            sha256: sha(&archive),
            size: archive.len() as u64,
        };
        let update = Update {
            version: "0.2.0".into(),
            pub_date: String::new(),
            release_url: String::new(),
            notes: String::new(),
            asset,
            whats_new: None,
            earlier: Vec::new(),
        };
        let install = Install {
            target: dir.join("kanso"),
            stage_dir: dir.join(".kanso-update"),
            platform,
        };
        fs::create_dir_all(dir).unwrap();
        prepare(
            &t,
            &UrlPolicy::RELEASE,
            &update,
            &install,
            &mut |_, _| {},
            &AtomicBool::new(false),
            &|_| Ok(()),
        )
    }

    #[test]
    fn staging_seals_the_host_beside_kanso() {
        let dir = tempfile::tempdir().unwrap();
        let with = prepared(
            tar_gz(&[
                ("Kanso-0.2.0-linux-x64/kanso", b"k", 0o755),
                ("Kanso-0.2.0-linux-x64/kanso-plugin-host", b"h", 0o755),
            ]),
            Platform::LinuxX64,
            &dir.path().join("with"),
        )
        .unwrap();
        let host = staged_host(&with, Platform::LinuxX64).unwrap();
        assert_eq!(fs::read(&host).unwrap(), b"h");
        assert_eq!(
            with.seal.host_digest.as_deref(),
            Some(payload_digest(&host).unwrap().as_str())
        );
        let without = prepared(
            tar_gz(&[("kanso", b"k", 0o755)]),
            Platform::LinuxX64,
            &dir.path().join("without"),
        )
        .unwrap();
        assert!(without.seal.is_set() && without.seal.host_digest.is_none());
        assert_eq!(staged_host(&without, Platform::LinuxX64), None);
        // A host that is a folder stops the update, and nothing stays staged.
        let err = prepared(
            tar_gz(&[("kanso", b"k", 0o755), ("kanso-plugin-host/x", b"h", 0o755)]),
            Platform::LinuxX64,
            &dir.path().join("dir"),
        )
        .unwrap_err();
        assert!(matches!(err, StageError::BadArchive(_)), "{err:?}");
    }

    #[test]
    fn archives_without_kanso_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("a.tar.gz");
        write(
            &archive,
            &tar_gz(&[("Other.app/Contents/Info.plist", b"x", 0o644)]),
        );
        let dest = dir.path().join("out");
        extract(&archive, ArchiveKind::TarGz, &dest).unwrap();
        assert!(matches!(
            find_payload(&dest, Platform::MacosArm64),
            Err(StageError::BadArchive(_))
        ));
        assert!(matches!(
            find_payload(&dest, Platform::LinuxX64),
            Err(StageError::BadArchive(_))
        ));
        // Two apps is one too many.
        write(
            &archive,
            &tar_gz(&[
                ("A.app/Contents/Info.plist", b"x", 0o644),
                ("A.app/Contents/MacOS/kanso", b"x", 0o755),
                ("B.app/Contents/Info.plist", b"x", 0o644),
                ("B.app/Contents/MacOS/kanso", b"x", 0o755),
            ]),
        );
        extract(&archive, ArchiveKind::TarGz, &dest).unwrap();
        assert!(find_payload(&dest, Platform::MacosArm64).is_err());
    }

    #[test]
    fn escaping_paths_and_links_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("out");
        let archive = dir.path().join("a.tar.gz");
        for (name, kind) in [
            ("../evil", tar::EntryType::Regular),
            ("/abs/evil", tar::EntryType::Regular),
            ("Kanso.app/link", tar::EntryType::Symlink),
            ("Kanso.app/dev", tar::EntryType::Char),
        ] {
            write(&archive, &tar_gz_raw_name(name, kind));
            let err = extract(&archive, ArchiveKind::TarGz, &dest).unwrap_err();
            assert!(matches!(err, StageError::BadArchive(_)), "{name}: {err:?}");
            assert!(!dir.path().join("evil").exists());
        }
        // A zip that climbs out is refused too.
        write(&archive, &zip(&[("../evil.exe", b"MZ")]));
        assert!(extract(&archive, ArchiveKind::Zip, &dest).is_err());
        assert!(!dir.path().join("evil.exe").exists());
    }

    #[test]
    fn a_pending_update_is_never_read_back_sealed() {
        let pending = Pending {
            version: "0.2.0".into(),
            payload: "/Applications/.kanso-update/unpacked-0.2.0/Kanso.app".into(),
            target: "/Applications/Kanso.app".into(),
            pub_date: "2026-10-20".into(),
            release_url: "https://github.com/iamngoni/kanso/releases/tag/v0.2.0".into(),
            notes: "- New".into(),
            whats_new: None,
            earlier: Vec::new(),
            seal: Seal {
                digest: Some("ab".repeat(32)),
                host_digest: Some("cd".repeat(32)),
                stage_id: Some((1, 2)),
            },
        };
        let json = serde_json::to_string(&pending).unwrap();
        assert!(
            !json.contains("abab") && !json.contains("cdcd"),
            "the seal stays in memory: {json}"
        );
        let back: Pending = serde_json::from_str(&json).unwrap();
        assert!(!back.seal.is_set());
    }

    #[cfg(unix)]
    #[test]
    fn the_staging_folder_is_made_afresh_and_private() {
        use std::os::unix::fs::PermissionsExt as _;
        let dir = tempfile::tempdir().unwrap();
        let victim = dir.path().join("victim");
        fs::create_dir(&victim).unwrap();
        fs::write(victim.join("notes.md"), b"keep me").unwrap();
        let stage = dir.path().join(".kanso-update");
        // Someone points the staging folder at another folder.
        std::os::unix::fs::symlink(&victim, &stage).unwrap();
        fresh_stage_dir(&stage).unwrap();
        assert_eq!(fs::read(victim.join("notes.md")).unwrap(), b"keep me");
        let meta = fs::symlink_metadata(&stage).unwrap();
        assert!(meta.is_dir() && !meta.file_type().is_symlink());
        assert_eq!(meta.permissions().mode() & 0o777, 0o700);
        // A leftover folder with files goes too.
        fs::write(stage.join("planted"), b"x").unwrap();
        fresh_stage_dir(&stage).unwrap();
        assert_eq!(fs::read_dir(&stage).unwrap().count(), 0);
        assert!(dir_identity(&stage).unwrap().is_some());
        // A file where the folder should be is replaced, not written into.
        fs::remove_dir(&stage).unwrap();
        fs::write(&stage, b"x").unwrap();
        fresh_stage_dir(&stage).unwrap();
        assert!(fs::symlink_metadata(&stage).unwrap().is_dir());
    }

    #[test]
    fn a_zip_entry_larger_than_it_says_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("out");
        let archive = dir.path().join("a.zip");
        let mut bytes = zip(&[("kanso.exe", &[0u8; 4096])]);
        // The central directory's uncompressed size, at offset 24 of its
        // entry: claim one byte where 4096 inflate.
        let at = bytes
            .windows(4)
            .position(|w| w == [0x50, 0x4b, 0x01, 0x02])
            .unwrap();
        bytes[at + 24..at + 28].copy_from_slice(&1u32.to_le_bytes());
        write(&archive, &bytes);
        let err = extract(&archive, ArchiveKind::Zip, &dest).unwrap_err();
        assert!(matches!(err, StageError::BadArchive(_)), "{err:?}");
        assert!(fs::metadata(dest.join("kanso.exe")).map_or(true, |m| m.len() <= 1));
    }

    #[cfg(unix)]
    #[test]
    fn digests_refuse_links() {
        let dir = tempfile::tempdir().unwrap();
        let app = dir.path().join("Kanso.app");
        fs::create_dir_all(app.join("Contents")).unwrap();
        fs::write(app.join("Contents/kanso"), b"x").unwrap();
        assert!(payload_digest(&app).is_ok());
        std::os::unix::fs::symlink("/bin/sh", app.join("Contents/sh")).unwrap();
        assert!(payload_digest(&app).is_err());
    }

    #[test]
    fn duplicate_archive_entries_replace_files_but_not_folders() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("out");
        let archive = dir.path().join("a.tar.gz");
        write(
            &archive,
            &tar_gz(&[("kanso", b"first", 0o755), ("kanso", b"second", 0o755)]),
        );
        extract(&archive, ArchiveKind::TarGz, &dest).unwrap();
        assert_eq!(fs::read(dest.join("kanso")).unwrap(), b"second");
    }

    #[test]
    fn a_full_disk_reads_as_no_space() {
        let err: StageError = io::Error::from(io::ErrorKind::StorageFull).into();
        assert!(matches!(err, StageError::NoSpace));
        let err: StageError = io::Error::from(io::ErrorKind::PermissionDenied).into();
        assert!(matches!(err, StageError::Io(_)));
    }

    proptest! {
        /// Whatever the body, only the bytes the feed names are kept.
        #[test]
        fn prop_only_the_signed_bytes_are_kept(
            body in prop::collection::vec(any::<u8>(), 1..4096),
            other in prop::collection::vec(any::<u8>(), 1..4096),
        ) {
            let dir = tempfile::tempdir().unwrap();
            let to = dir.path().join("a");
            let asset = Asset { url: URL.into(), sha256: sha(&body), size: body.len() as u64 };
            let result = fetch(&other, &asset, &to);
            prop_assert_eq!(result.is_ok(), other == body);
            prop_assert_eq!(to.exists(), other == body);
        }

        /// Any change to a staged update (a file's bytes, a new file, a
        /// rename, and on Unix a permission) changes its digest; nothing
        /// else does.
        #[test]
        fn prop_digests_catch_every_change(
            files in prop::collection::btree_map("[a-c]{1,3}", prop::collection::vec(any::<u8>(), 0..64), 1..6),
            change in 0u8..4,
            pick in any::<prop::sample::Index>(),
        ) {
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path().join("Kanso.app");
            fs::create_dir_all(root.join("Contents")).unwrap();
            for (name, bytes) in &files {
                fs::write(root.join("Contents").join(name), bytes).unwrap();
            }
            let before = payload_digest(&root).unwrap();
            prop_assert_eq!(&payload_digest(&root).unwrap(), &before, "stable");
            let names: Vec<&String> = files.keys().collect();
            let victim = root.join("Contents").join(names[pick.index(names.len())]);
            match change {
                0 => {
                    let mut bytes = fs::read(&victim).unwrap();
                    bytes.push(0);
                    fs::write(&victim, bytes).unwrap();
                }
                1 => fs::write(root.join("Contents/zz-planted"), b"").unwrap(),
                2 => fs::rename(&victim, root.join("Contents/zz-renamed")).unwrap(),
                _ => {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt as _;
                        let mode = fs::metadata(&victim).unwrap().permissions().mode();
                        fs::set_permissions(&victim, fs::Permissions::from_mode(mode ^ 0o100)).unwrap();
                    }
                    #[cfg(not(unix))]
                    fs::write(root.join("Contents/zz-planted"), b"").unwrap();
                }
            }
            prop_assert_ne!(payload_digest(&root).unwrap(), before);
        }

        /// The host is found iff the archive has it beside `kanso`, at the
        /// top or in the one folder; one anywhere else is never taken.
        #[test]
        fn prop_the_host_is_found_iff_it_is_beside_kanso(
            windows in any::<bool>(),
            nested in any::<bool>(),
            host_beside in any::<bool>(),
            host_elsewhere in prop::sample::select(vec!["", "bin", "other", "a/b"]),
            content in prop::collection::vec(any::<u8>(), 0..64),
        ) {
            let platform = if windows { Platform::WindowsX64 } else { Platform::LinuxX64 };
            let (exe, host) = (if windows { "kanso.exe" } else { "kanso" }, platform.host_name());
            let folder = if nested { "Kanso-0.2.0/" } else { "" };
            let mut names = vec![format!("{folder}{exe}")];
            if host_beside {
                names.push(format!("{folder}{host}"));
            }
            if !host_elsewhere.is_empty() {
                // Another folder's host: below the folder with `kanso`, or
                // beside the one folder, never next to the payload.
                names.push(format!("{folder}{host_elsewhere}/{host}"));
            }
            let dir = tempfile::tempdir().unwrap();
            let bytes = if windows {
                let files: Vec<(&str, &[u8])> = names.iter().map(|n| (n.as_str(), &content[..])).collect();
                zip(&files)
            } else {
                let files: Vec<(&str, &[u8], u32)> = names.iter().map(|n| (n.as_str(), &content[..], 0o755)).collect();
                tar_gz(&files)
            };
            let kind = platform.archive();
            let dest = unpacked(dir.path(), kind, &bytes);
            let payload = find_payload(&dest, platform).unwrap();
            let found = find_host(&payload, platform).unwrap();
            if host_beside {
                prop_assert_eq!(found, Some(dest.join(format!("{folder}{host}"))));
            } else {
                prop_assert_eq!(found, None);
            }
        }

        /// A macOS update never has a host to find, whatever is beside.
        #[test]
        fn prop_hosts_never_affect_macos_bundles(stray in any::<bool>(), inside in any::<bool>()) {
            let dir = tempfile::tempdir().unwrap();
            let mut files: Vec<(&str, &[u8], u32)> = vec![
                ("Kanso.app/Contents/Info.plist", b"<plist/>", 0o644),
                ("Kanso.app/Contents/MacOS/kanso", b"x", 0o755),
            ];
            if inside {
                files.push(("Kanso.app/Contents/MacOS/kanso-plugin-host", b"x", 0o755));
            }
            if stray {
                files.push(("kanso-plugin-host", b"x", 0o755));
            }
            let dest = unpacked(dir.path(), ArchiveKind::TarGz, &tar_gz(&files));
            let app = find_payload(&dest, Platform::MacosArm64).unwrap();
            prop_assert_eq!(&app, &dest.join("Kanso.app"));
            prop_assert_eq!(find_host(&app, Platform::MacosArm64).unwrap(), None);
        }

        /// Paths inside the folder are kept as they are; anything else is
        /// refused.
        #[test]
        fn prop_safe_paths_stay_inside(parts in prop::collection::vec(prop::sample::select(vec!["a", "b.app", ".", "..", "Contents"]), 1..6), absolute in any::<bool>()) {
            let joined = parts.join("/");
            let path = if absolute { format!("/{joined}") } else { joined };
            let safe = safe_relative(Path::new(&path));
            let climbs = absolute || parts.contains(&"..");
            if climbs {
                prop_assert!(safe.is_none());
            } else if let Some(rel) = safe {
                prop_assert!(rel.is_relative());
                prop_assert!(rel.components().all(|c| matches!(c, Component::Normal(_))));
            }
        }
    }
}
