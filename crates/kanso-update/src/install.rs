//! Where Kanso is installed, whether it can update itself there, and the
//! swap that puts a staged update in its place.
//!
//! The swap is two renames on one volume: what runs now moves aside, the
//! staged update moves in, and if the second rename fails the first is
//! undone. A running program may be renamed on every system Kanso ships
//! for, so the swap happens while the old Kanso is quitting; the new one
//! then waits for its data-folder lock (`--after-update`).
//!
//! Beside `kanso` on Windows and Linux sits the plugin host. When the update
//! has one it is swapped in the same way, first, and put back if `kanso`
//! then fails, so the update lands whole or not at all.

use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use crate::platform::Platform;
use crate::stage::{self, Pending};

/// The staging folder's name, beside what it replaces.
pub const STAGE_DIR_NAME: &str = ".kanso-update";

/// An install Kanso can update.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Install {
    /// The `.app` on macOS, the executable elsewhere.
    pub target: PathBuf,
    /// Beside the target, on its volume.
    pub stage_dir: PathBuf,
    pub platform: Platform,
}

/// What the app knows about itself that decides whether it may update.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LocateOptions {
    /// Kanso Dev: never updated from releases.
    pub dev_flavour: bool,
    /// A debug build.
    pub debug_build: bool,
    /// Running as an AppImage, whose image the updater doesn't replace.
    pub appimage: bool,
}

/// Why this copy of Kanso can't update itself; the app offers the release
/// page instead and says why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotInstallable {
    DevFlavour,
    DevBuild,
    CargoBuild,
    UnsupportedSystem,
    NotInBundle,
    Translocated,
    DiskImage,
    AppImage,
    /// Installed from a package (apt), which updates it: `/usr/bin`.
    PackageManaged,
    NotWritable(PathBuf),
    /// The install belongs to another account on this computer.
    OwnedByAnother(PathBuf),
}

impl NotInstallable {
    /// One sentence for the Updates page.
    pub fn reason(&self) -> String {
        match self {
            NotInstallable::DevFlavour => "Development build — updates are off.".into(),
            NotInstallable::DevBuild | NotInstallable::CargoBuild => {
                "This copy was built from source, so it doesn't replace itself. Download the release instead.".into()
            }
            NotInstallable::UnsupportedSystem => "There are no release builds for this system.".into(),
            NotInstallable::NotInBundle => "Kanso isn't running from Kanso.app, so it can't replace itself.".into(),
            NotInstallable::Translocated => {
                "macOS is running Kanso from a temporary copy. Move Kanso.app to Applications, open it from there, and updates install themselves.".into()
            }
            NotInstallable::DiskImage => "Kanso is running from its disk image. Drag it to Applications first.".into(),
            NotInstallable::AppImage => "AppImages update by downloading the new image.".into(),
            NotInstallable::PackageManaged => format!(
                "Kanso was installed by your package manager. Update it with {APT_UPGRADE}."
            ),
            NotInstallable::NotWritable(dir) => format!(
                "Kanso can't write to {}, so it can't replace itself. Download the update instead.",
                dir.display()
            ),
            NotInstallable::OwnedByAnother(target) => format!(
                "{} belongs to another account on this computer, so Kanso doesn't replace it. Download the update instead.",
                target.display()
            ),
        }
    }
}

/// The install `exe` belongs to, if this copy may update itself there:
/// not Kanso Dev, not a debug or `cargo` build, and in a folder it can
/// write to. On macOS that is the enclosing `.app`.
pub fn locate(
    exe: &Path,
    platform: Option<Platform>,
    options: LocateOptions,
) -> Result<Install, NotInstallable> {
    if options.dev_flavour {
        return Err(NotInstallable::DevFlavour);
    }
    if options.debug_build {
        return Err(NotInstallable::DevBuild);
    }
    let platform = platform.ok_or(NotInstallable::UnsupportedSystem)?;
    if is_cargo_build(exe) {
        return Err(NotInstallable::CargoBuild);
    }
    if options.appimage {
        return Err(NotInstallable::AppImage);
    }
    if is_package_managed(exe, platform) {
        return Err(NotInstallable::PackageManaged);
    }
    let target = if platform.is_bundle() {
        let app = exe
            .ancestors()
            .find(|p| p.extension().is_some_and(|e| e == "app"))
            .ok_or(NotInstallable::NotInBundle)?
            .to_path_buf();
        let text = app.to_string_lossy();
        if text.contains("/AppTranslocation/") {
            return Err(NotInstallable::Translocated);
        }
        if text.starts_with("/Volumes/") && !writable_dir(app.parent().unwrap_or(&app)) {
            return Err(NotInstallable::DiskImage);
        }
        app
    } else {
        exe.to_path_buf()
    };
    let parent = target
        .parent()
        .ok_or(NotInstallable::NotInBundle)?
        .to_path_buf();
    let Some(writer) = probe_dir(&parent) else {
        return Err(NotInstallable::NotWritable(parent));
    };
    // Replacing what another account installed (a root-owned binary in a
    // folder this account can write to, say) would hand that account's
    // copy to this one: the swap leaves the new copy owned by whoever ran
    // it. Only one's own install is updated.
    if let (Some(me), Some(owner)) = (writer, owner_of(&target))
        && me != owner
    {
        return Err(NotInstallable::OwnedByAnother(target));
    }
    Ok(Install {
        stage_dir: parent.join(STAGE_DIR_NAME),
        target,
        platform,
    })
}

/// What updates a copy apt installed.
pub const APT_UPGRADE: &str = "sudo apt upgrade";

/// Whether `exe` is a Linux copy a package installed: under `/usr`, which
/// dpkg owns, but not `/usr/local`, which is the administrator's own.
/// Symbolic links are followed first, so a link in `~/.local/bin` to the
/// packaged copy counts too.
pub fn is_package_managed(exe: &Path, platform: Platform) -> bool {
    if !matches!(platform, Platform::LinuxX64 | Platform::LinuxArm64) {
        return false;
    }
    let real = exe.canonicalize().unwrap_or_else(|_| exe.to_path_buf());
    real.starts_with("/usr") && !real.starts_with("/usr/local")
}

/// Whether `exe` sits in a Cargo target folder: `target/debug`,
/// `target/release` or `target/<triple>/release`.
pub fn is_cargo_build(exe: &Path) -> bool {
    let parts: Vec<String> = exe
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    parts
        .windows(2)
        .any(|w| w[0] == "target" && (w[1] == "debug" || w[1] == "release"))
        || parts.windows(3).any(|w| {
            w[0] == "target" && (w[2] == "debug" || w[2] == "release") && w[1].contains('-')
        })
}

/// Whether a file can be created in `dir` (tried, then removed).
pub fn writable_dir(dir: &Path) -> bool {
    probe_dir(dir).is_some()
}

/// Creates and removes a file in `dir`: `None` when it can't, and
/// otherwise the account that owns what this process creates there (on
/// Unix; elsewhere `Some(None)`).
fn probe_dir(dir: &Path) -> Option<Option<u32>> {
    let probe = dir.join(format!(".kanso-write-test-{}", std::process::id()));
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)
        .ok()?;
    let owner = file.metadata().ok().and_then(|m| uid(&m));
    drop(file);
    fs::remove_file(&probe).ok();
    Some(owner)
}

/// The account that owns `path` itself (not a link's target), on Unix.
fn owner_of(path: &Path) -> Option<u32> {
    fs::symlink_metadata(path).ok().and_then(|m| uid(&m))
}

#[cfg(unix)]
fn uid(meta: &fs::Metadata) -> Option<u32> {
    use std::os::unix::fs::MetadataExt as _;
    Some(meta.uid())
}

#[cfg(not(unix))]
fn uid(_: &fs::Metadata) -> Option<u32> {
    None
}

/// Where what runs now goes during the swap: inside the staging folder for
/// a bundle (removed with it), and `<exe>.old` beside an executable, which
/// Windows only lets go of once the old Kanso has quit.
pub fn backup_path(target: &Path, stage_dir: &Path, platform: Platform) -> PathBuf {
    if platform.is_bundle() {
        stage_dir.join("previous.app")
    } else {
        let mut name = target.file_name().unwrap_or_default().to_os_string();
        name.push(".old");
        target.with_file_name(name)
    }
}

/// Where the plugin host is installed: beside the executable. `None` on
/// macOS, where it is inside the `.app` that is swapped whole.
pub fn host_target(install: &Install) -> Option<PathBuf> {
    (!install.platform.is_bundle())
        .then(|| install.target.with_file_name(install.platform.host_name()))
}

/// The plugin host's swap: the staged file, the installed one it replaces
/// (which may not exist yet), and where the installed one goes aside.
struct HostSwap {
    staged: PathBuf,
    target: PathBuf,
    backup: PathBuf,
}

impl HostSwap {
    /// Puts the staged host in place, moving an installed one aside as
    /// [`swap_in`] does. Gives whether there was one. On failure nothing has
    /// changed.
    fn swap_in(&self) -> io::Result<bool> {
        match fs::symlink_metadata(&self.target) {
            Ok(_) => swap_in(&self.staged, &self.target, &self.backup).map(|()| true),
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                fs::rename(&self.staged, &self.target).map(|()| false)
            }
            Err(err) => Err(err),
        }
    }

    /// Undoes [`HostSwap::swap_in`]: the new host goes back to the staging
    /// folder (or is removed if it can't), and the old one, if there was
    /// one, back to its place.
    fn undo(&self, had_old: bool) -> io::Result<()> {
        if fs::rename(&self.target, &self.staged).is_err() {
            remove_any(&self.target)?;
        }
        if had_old {
            fs::rename(&self.backup, &self.target)?;
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ApplyError {
    #[error("the staged update is for {staged}, not this install")]
    WrongTarget { staged: PathBuf },
    #[error("the staged update is gone")]
    Missing,
    /// Not staged by this run of Kanso, or changed since it was checked.
    #[error("the staged update changed after it was checked")]
    Tampered,
    #[error("couldn't swap the update in: {0}")]
    Io(#[from] io::Error),
}

/// Swaps `pending` in for `install.target`, and leaves the old one at
/// [`backup_path`] only if it couldn't be removed. On failure everything is
/// as it was.
///
/// On Windows and Linux a plugin host staged beside the payload goes in
/// beside the installed `kanso`, before it, with its own backup; if `kanso`
/// then fails the host is put back. An update without a host leaves any
/// installed one alone.
///
/// Only an update this run staged is swapped in, and only as it was when
/// it was checked: its [`stage::Seal`] must be set, the staging folder must
/// be the one [`stage::prepare`] made, and the payload's digest must be
/// unchanged, and so must the host's.
pub fn apply(pending: &Pending, install: &Install) -> Result<(), ApplyError> {
    if pending.target != install.target {
        return Err(ApplyError::WrongTarget {
            staged: pending.target.clone(),
        });
    }
    if !is_inside(&pending.payload, &install.stage_dir) {
        return Err(ApplyError::Missing);
    }
    if fs::symlink_metadata(&pending.payload).is_err() {
        return Err(ApplyError::Missing);
    }
    check_seal(pending, install)?;
    let backup = backup_path(&install.target, &install.stage_dir, install.platform);
    remove_any(&backup)?;
    let host = match (
        stage::staged_host(pending, install.platform),
        host_target(install),
    ) {
        (Some(staged), Some(target)) => {
            let backup = backup_path(&target, &install.stage_dir, install.platform);
            remove_any(&backup)?;
            Some(HostSwap {
                staged,
                target,
                backup,
            })
        }
        _ => None,
    };
    let had_old_host = host.as_ref().map(HostSwap::swap_in).transpose()?;
    if let Err(err) = swap_in(&pending.payload, &install.target, &backup) {
        if let (Some(host), Some(had_old)) = (&host, had_old_host)
            && let Err(undo) = host.undo(had_old)
        {
            return Err(io::Error::other(format!(
                "{err}; and putting the plugin host back failed: {undo}"
            ))
            .into());
        }
        return Err(err.into());
    }
    if install.platform.is_bundle() {
        fs::remove_dir_all(&install.stage_dir).ok();
    } else {
        // Windows keeps the running executable: it goes on the next start.
        remove_any(&backup).ok();
        if let Some(host) = &host {
            remove_any(&host.backup).ok();
        }
        fs::remove_dir_all(&install.stage_dir).ok();
    }
    Ok(())
}

/// Whether `path` is strictly inside `dir`, by normal names only: no `..`
/// climbs back out.
fn is_inside(path: &Path, dir: &Path) -> bool {
    path.strip_prefix(dir).is_ok_and(|rel| {
        !rel.as_os_str().is_empty() && rel.components().all(|c| matches!(c, Component::Normal(_)))
    })
}

fn check_seal(pending: &Pending, install: &Install) -> Result<(), ApplyError> {
    let Some(expected) = &pending.seal.digest else {
        return Err(ApplyError::Tampered);
    };
    let stage_id = stage::dir_identity(&install.stage_dir).map_err(|_| ApplyError::Tampered)?;
    if pending.seal.stage_id.is_some() && stage_id != pending.seal.stage_id {
        return Err(ApplyError::Tampered);
    }
    match stage::payload_digest(&pending.payload) {
        Ok(digest) if digest == *expected => {}
        _ => return Err(ApplyError::Tampered),
    }
    // The host is held to the same: sealed when staged, unchanged now.
    if let Some(expected) = &pending.seal.host_digest {
        let host = stage::staged_host(pending, install.platform).ok_or(ApplyError::Tampered)?;
        if install.platform.is_bundle() {
            return Err(ApplyError::Tampered);
        }
        match stage::payload_digest(&host) {
            Ok(digest) if digest == *expected => {}
            _ => return Err(ApplyError::Tampered),
        }
    }
    Ok(())
}

/// The two renames: `target` to `backup`, then `staged` to `target`,
/// undoing the first if the second fails.
pub fn swap_in(staged: &Path, target: &Path, backup: &Path) -> io::Result<()> {
    fs::rename(target, backup)?;
    if let Err(err) = fs::rename(staged, target) {
        // Put the running copy back; if even that fails the error says so.
        if let Err(undo) = fs::rename(backup, target) {
            return Err(io::Error::other(format!(
                "{err}; and putting Kanso back failed: {undo}"
            )));
        }
        return Err(err);
    }
    Ok(())
}

/// After a start: removes what an earlier update left, the `.old`
/// executables (`kanso` and the plugin host) and the staging folder, whatever is in it. An update staged
/// by an earlier run is never installed: it is downloaded and checked
/// again, since nothing on disk can say it wasn't changed in between.
/// Links are removed, never followed.
pub fn clean_up(install: &Install) {
    let backup = backup_path(&install.target, &install.stage_dir, install.platform);
    if !install.platform.is_bundle() {
        remove_any(&backup).ok();
    }
    if let Some(host) = host_target(install) {
        let backup = backup_path(&host, &install.stage_dir, install.platform);
        remove_any(&backup).ok();
    }
    remove_any(&install.stage_dir).ok();
}

fn remove_any(path: &Path) -> io::Result<()> {
    stage::remove_entry(path)
}

/// Starts the installed Kanso again, telling it to wait for this one to
/// let go of the data folder. `args` are this run's own (a `--data-dir`,
/// say).
///
/// On macOS this is what Sparkle and Zed do: a helper outside this process
/// (its own process group, so it outlives the quit) waits until this
/// process has exited, then opens the bundle through LaunchServices with a
/// plain `open`. The new copy is then a normal launch of the one app, not a
/// second instance (`open -n`) racing the old one, and macOS brings it to
/// the front as it would any app the person opened. Elsewhere the new
/// executable starts at once and waits for the data-folder lock itself.
pub fn relaunch(install: &Install, args: &[String]) -> io::Result<()> {
    let mut command = if install.platform.is_bundle() {
        relaunch_after_exit(
            std::process::id(),
            Path::new("/usr/bin/open"),
            &install.target,
        )
    } else {
        std::process::Command::new(&install.target)
    };
    command.arg("--after-update").args(args);
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(&mut command, 0);
    command
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(drop)
}

/// The script [`relaunch_after_exit`] runs: `$1` is the process to wait
/// for, `$2` the opener (`/usr/bin/open`), `$3` the bundle, the rest its
/// arguments. It gives up waiting after a minute, since the new copy waits
/// for the data folder anyway.
pub const RELAUNCH_SCRIPT: &str = r#"pid="$1"; opener="$2"; app="$3"; shift 3
i=0
while kill -0 "$pid" 2>/dev/null && [ "$i" -lt 600 ]; do sleep 0.1; i=$((i + 1)); done
exec "$opener" "$app" --args "$@"
"#;

/// `/bin/sh` running [`RELAUNCH_SCRIPT`]: once `pid` has exited, `opener`
/// opens `app`. The caller appends the new copy's arguments.
pub fn relaunch_after_exit(pid: u32, opener: &Path, app: &Path) -> std::process::Command {
    let mut c = std::process::Command::new("/bin/sh");
    c.arg("-c")
        .arg(RELAUNCH_SCRIPT)
        .arg("kanso-relaunch")
        .arg(pid.to_string())
        .arg(opener)
        .arg(app);
    c
}

/// The Team ID in `codesign -dv` output, unless it is "not set".
pub fn parse_team_id(codesign_output: &str) -> Option<String> {
    codesign_field(codesign_output, "TeamIdentifier")
}

/// The signing identifier (the bundle's) in `codesign -dv` output.
pub fn parse_identifier(codesign_output: &str) -> Option<String> {
    codesign_field(codesign_output, "Identifier")
}

fn codesign_field(codesign_output: &str, name: &str) -> Option<String> {
    codesign_output
        .lines()
        .find_map(|line| line.trim().strip_prefix(name)?.strip_prefix('='))
        .map(str::trim)
        .filter(|id| !id.is_empty() && *id != "not set")
        .map(str::to_owned)
}

/// macOS's own checks on a staged `.app`: its code signature verifies
/// (`codesign --verify --deep --strict`), it is signed as the same app
/// (identifier) as the running Kanso, and when the running Kanso is signed
/// by a developer, the update is signed by the same Team ID.
#[cfg(target_os = "macos")]
pub fn verify_bundle(staged: &Path, running: Option<&Path>) -> Result<(), String> {
    use std::process::Command;
    let out = Command::new("/usr/bin/codesign")
        .args(["--verify", "--deep", "--strict"])
        .arg(staged)
        .output()
        .map_err(|e| format!("couldn't run codesign: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_owned());
    }
    let describe = |app: &Path| {
        Command::new("/usr/bin/codesign")
            .args(["-dv", "--verbose=2"])
            .arg(app)
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stderr).into_owned())
            .unwrap_or_default()
    };
    let Some(running) = running.map(describe) else {
        return Ok(());
    };
    let staged_info = describe(staged);
    if let Some(expected) = parse_identifier(&running) {
        match parse_identifier(&staged_info) {
            Some(got) if got == expected => {}
            got => {
                return Err(format!(
                    "signed as {} instead of {expected}",
                    got.as_deref().unwrap_or("nothing")
                ));
            }
        }
    }
    if let Some(expected) = parse_team_id(&running) {
        match parse_team_id(&staged_info) {
            Some(got) if got == expected => {}
            got => {
                return Err(format!(
                    "signed by {} instead of {expected}",
                    got.as_deref().unwrap_or("no developer")
                ));
            }
        }
    }
    Ok(())
}

/// Removes the quarantine flag macOS may have put on a download, so the
/// new Kanso opens without asking.
#[cfg(target_os = "macos")]
pub fn remove_quarantine(app: &Path) {
    std::process::Command::new("/usr/bin/xattr")
        .args(["-dr", "com.apple.quarantine"])
        .arg(app)
        .output()
        .ok();
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// An opener that writes each argument it got, NUL-terminated, to `$OUT`.
    #[cfg(unix)]
    fn recording_opener(dir: &Path) -> (PathBuf, PathBuf) {
        use std::os::unix::fs::PermissionsExt as _;
        let opener = dir.join("open");
        let out = dir.join("opened");
        fs::write(
            &opener,
            format!(
                "#!/bin/sh\nprintf '%s\\0' \"$@\" > '{}.tmp' && mv '{}.tmp' '{}'\n",
                out.display(),
                out.display(),
                out.display()
            ),
        )
        .unwrap();
        fs::set_permissions(&opener, fs::Permissions::from_mode(0o755)).unwrap();
        (opener, out)
    }

    #[cfg(unix)]
    fn wait_for(path: &Path) -> Vec<String> {
        for _ in 0..200 {
            if let Ok(bytes) = fs::read(path) {
                let text = String::from_utf8(bytes).unwrap();
                return text.split_terminator('\0').map(str::to_owned).collect();
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        panic!("the opener never ran");
    }

    /// A process id that has already exited.
    #[cfg(unix)]
    fn gone_pid() -> u32 {
        let mut child = std::process::Command::new("/usr/bin/true").spawn().unwrap();
        let pid = child.id();
        child.wait().unwrap();
        pid
    }

    #[cfg(unix)]
    #[test]
    fn relaunch_waits_for_the_old_process_to_exit() {
        let dir = tempfile::tempdir().unwrap();
        let (opener, out) = recording_opener(dir.path());
        let mut old = std::process::Command::new("/bin/sleep")
            .arg("0.6")
            .spawn()
            .unwrap();
        let app = dir.path().join("Kanso.app");
        let mut helper = relaunch_after_exit(old.id(), &opener, &app)
            .args(["--after-update", "--data-dir", "/a b/K's \"lib\""])
            .spawn()
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(300));
        assert!(!out.exists(), "opened while the old copy still ran");
        old.wait().unwrap();
        helper.wait().unwrap();
        assert_eq!(
            wait_for(&out),
            [
                app.to_str().unwrap(),
                "--args",
                "--after-update",
                "--data-dir",
                "/a b/K's \"lib\""
            ]
        );
    }

    #[cfg(unix)]
    proptest! {
        #![proptest_config(ProptestConfig { cases: 24, ..ProptestConfig::default() })]
        /// Round trip: whatever arguments the new copy is given reach the
        /// opener unchanged, after `--args`, with nothing the shell expands.
        #[test]
        fn prop_relaunch_passes_arguments_through(
            args in prop::collection::vec("[ -~]{0,12}", 0..4),
        ) {
            let dir = tempfile::tempdir().unwrap();
            let (opener, out) = recording_opener(dir.path());
            let app = dir.path().join("Kanso $HOME `x`.app");
            relaunch_after_exit(gone_pid(), &opener, &app).args(&args).spawn().unwrap().wait().unwrap();
            let mut expected = vec![app.to_str().unwrap().to_owned(), "--args".to_owned()];
            expected.extend(args);
            prop_assert_eq!(wait_for(&out), expected);
        }
    }

    fn opts() -> LocateOptions {
        LocateOptions::default()
    }

    #[test]
    fn a_bundle_in_a_writable_folder_installs() {
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("Kanso.app/Contents/MacOS/kanso");
        fs::create_dir_all(exe.parent().unwrap()).unwrap();
        fs::write(&exe, b"x").unwrap();
        let install = locate(&exe, Some(Platform::MacosArm64), opts()).unwrap();
        assert_eq!(install.target, dir.path().join("Kanso.app"));
        assert_eq!(install.stage_dir, dir.path().join(STAGE_DIR_NAME));
        let install = locate(&dir.path().join("kanso"), Some(Platform::LinuxX64), opts()).unwrap();
        assert_eq!(install.target, dir.path().join("kanso"));
    }

    #[test]
    fn each_reason_not_to_install_is_named() {
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("kanso");
        let mac = Some(Platform::MacosArm64);
        assert_eq!(
            locate(
                &exe,
                mac,
                LocateOptions {
                    dev_flavour: true,
                    ..opts()
                }
            ),
            Err(NotInstallable::DevFlavour)
        );
        assert_eq!(
            locate(
                &exe,
                mac,
                LocateOptions {
                    debug_build: true,
                    ..opts()
                }
            ),
            Err(NotInstallable::DevBuild)
        );
        assert_eq!(
            locate(&exe, None, opts()),
            Err(NotInstallable::UnsupportedSystem)
        );
        assert_eq!(
            locate(
                Path::new("/src/kanso/target/release/Kanso.app/Contents/MacOS/kanso"),
                mac,
                opts()
            ),
            Err(NotInstallable::CargoBuild)
        );
        assert_eq!(locate(&exe, mac, opts()), Err(NotInstallable::NotInBundle));
        assert_eq!(
            locate(
                Path::new(
                    "/private/var/folders/x/AppTranslocation/ABC/d/Kanso.app/Contents/MacOS/kanso"
                ),
                mac,
                opts()
            ),
            Err(NotInstallable::Translocated)
        );
        assert_eq!(
            locate(
                &exe,
                Some(Platform::LinuxX64),
                LocateOptions {
                    appimage: true,
                    ..opts()
                }
            ),
            Err(NotInstallable::AppImage)
        );
        let missing = dir.path().join("no/such/folder/kanso");
        assert_eq!(
            locate(&missing, Some(Platform::LinuxX64), opts()),
            Err(NotInstallable::NotWritable(
                dir.path().join("no/such/folder")
            ))
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let locked = dir.path().join("locked");
            fs::create_dir(&locked).unwrap();
            fs::set_permissions(&locked, fs::Permissions::from_mode(0o555)).unwrap();
            // Root can write anywhere; the check only means something
            // for an ordinary user.
            if !writable_dir(&locked) {
                assert_eq!(
                    locate(&locked.join("kanso"), Some(Platform::LinuxX64), opts()),
                    Err(NotInstallable::NotWritable(locked.clone()))
                );
            }
            fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
        }
        for reason in [
            NotInstallable::DevFlavour,
            NotInstallable::PackageManaged,
            NotInstallable::NotWritable("/x".into()),
            NotInstallable::OwnedByAnother("/x/kanso".into()),
        ] {
            assert!(reason.reason().ends_with('.'));
        }
    }

    /// A copy apt put in /usr/bin is left to apt; a copy the tarball's
    /// installer put in ~/.local/bin, or one in /usr/local, is as before;
    /// macOS and Windows never count as packaged.
    #[test]
    fn a_packaged_copy_is_left_to_apt() {
        for linux in [Platform::LinuxX64, Platform::LinuxArm64] {
            assert_eq!(
                locate(Path::new("/usr/bin/kanso"), Some(linux), opts()),
                Err(NotInstallable::PackageManaged)
            );
            assert!(is_package_managed(Path::new("/usr/lib/kanso/kanso"), linux));
            assert!(!is_package_managed(
                Path::new("/usr/local/bin/kanso"),
                linux
            ));
            assert!(!is_package_managed(Path::new("/usrx/bin/kanso"), linux));
        }
        let dir = tempfile::tempdir().unwrap();
        let local = dir.path().join(".local/bin");
        fs::create_dir_all(&local).unwrap();
        let install = locate(&local.join("kanso"), Some(Platform::LinuxX64), opts()).unwrap();
        assert_eq!(install.target, local.join("kanso"));
        for other in [Platform::MacosArm64, Platform::WindowsX64] {
            assert!(!is_package_managed(Path::new("/usr/bin/kanso"), other));
            assert_ne!(
                locate(Path::new("/usr/bin/kanso"), Some(other), opts()),
                Err(NotInstallable::PackageManaged)
            );
        }
        assert!(
            NotInstallable::PackageManaged
                .reason()
                .contains("sudo apt upgrade")
        );
    }

    fn fake_install(dir: &Path, platform: Platform) -> (Install, Pending) {
        let (target, payload) = if platform.is_bundle() {
            let target = dir.join("Kanso.app");
            fs::create_dir_all(target.join("Contents")).unwrap();
            fs::write(target.join("Contents/version"), "old").unwrap();
            let payload = dir.join(STAGE_DIR_NAME).join("unpacked-0.2.0/Kanso.app");
            fs::create_dir_all(payload.join("Contents")).unwrap();
            fs::write(payload.join("Contents/version"), "new").unwrap();
            (target, payload)
        } else {
            let target = dir.join("kanso");
            fs::write(&target, "old").unwrap();
            let payload = dir.join(STAGE_DIR_NAME).join("unpacked-0.2.0/kanso");
            fs::create_dir_all(payload.parent().unwrap()).unwrap();
            fs::write(&payload, "new").unwrap();
            (target, payload)
        };
        let install = Install {
            target: target.clone(),
            stage_dir: dir.join(STAGE_DIR_NAME),
            platform,
        };
        let pending = Pending {
            version: "0.2.0".into(),
            seal: seal(&install.stage_dir, &payload),
            payload,
            target,
            pub_date: String::new(),
            release_url: String::new(),
            notes: String::new(),
            whats_new: None,
            earlier: Vec::new(),
        };
        (install, pending)
    }

    /// The seal `stage::prepare` would have given what is there now.
    fn seal(stage_dir: &Path, payload: &Path) -> stage::Seal {
        stage::Seal {
            digest: Some(stage::payload_digest(payload).unwrap()),
            host_digest: None,
            stage_id: stage::dir_identity(stage_dir).unwrap(),
        }
    }

    fn installed(install: &Install) -> String {
        if install.platform.is_bundle() {
            fs::read_to_string(install.target.join("Contents/version")).unwrap()
        } else {
            fs::read_to_string(&install.target).unwrap()
        }
    }

    #[test]
    fn applying_swaps_and_cleans_up() {
        for platform in Platform::ALL {
            let dir = tempfile::tempdir().unwrap();
            let (install, pending) = fake_install(dir.path(), platform);
            apply(&pending, &install).unwrap();
            assert_eq!(installed(&install), "new", "{platform:?}");
            assert!(!install.stage_dir.exists());
            assert!(!backup_path(&install.target, &install.stage_dir, platform).exists());
        }
    }

    /// Stages a plugin host beside `pending`'s payload (when `staged` has
    /// bytes) and installs one beside the target (when `installed` does),
    /// and seals what is staged as `stage::prepare` would.
    fn with_host(
        install: &Install,
        mut pending: Pending,
        staged: Option<&[u8]>,
        installed: Option<&[u8]>,
    ) -> Pending {
        if let Some(bytes) = staged {
            let host = pending.payload.with_file_name(install.platform.host_name());
            fs::write(&host, bytes).unwrap();
            pending.seal = seal(&install.stage_dir, &pending.payload);
            pending.seal.host_digest = Some(stage::payload_digest(&host).unwrap());
        }
        if let Some(bytes) = installed {
            fs::write(host_target(install).unwrap(), bytes).unwrap();
        }
        pending
    }

    fn installed_host(install: &Install) -> Option<Vec<u8>> {
        fs::read(host_target(install).unwrap()).ok()
    }

    #[test]
    fn a_host_goes_in_beside_kanso_with_it() {
        for platform in [Platform::LinuxX64, Platform::WindowsX64] {
            // Replacing an installed host, and adding one to a folder that
            // had none.
            for before in [Some(&b"old host"[..]), None] {
                let dir = tempfile::tempdir().unwrap();
                let (install, pending) = fake_install(dir.path(), platform);
                let pending = with_host(&install, pending, Some(b"new host"), before);
                apply(&pending, &install).unwrap();
                assert_eq!(installed(&install), "new");
                assert_eq!(installed_host(&install).unwrap(), b"new host");
                assert!(!install.stage_dir.exists());
                let host = host_target(&install).unwrap();
                assert_eq!(host.parent(), install.target.parent());
                assert!(
                    !backup_path(&host, &install.stage_dir, platform).exists(),
                    "{platform:?}"
                );
            }
        }
    }

    #[test]
    fn an_update_without_a_host_leaves_the_installed_one_alone() {
        for platform in [Platform::LinuxArm64, Platform::WindowsX64] {
            let dir = tempfile::tempdir().unwrap();
            let (install, pending) = fake_install(dir.path(), platform);
            let pending = with_host(&install, pending, None, Some(b"old host"));
            apply(&pending, &install).unwrap();
            assert_eq!(installed(&install), "new");
            assert_eq!(installed_host(&install).unwrap(), b"old host");
            // And one with neither stays without.
            let dir = tempfile::tempdir().unwrap();
            let (install, pending) = fake_install(dir.path(), platform);
            apply(&pending, &install).unwrap();
            assert_eq!(installed_host(&install), None);
        }
    }

    #[test]
    fn a_host_that_was_not_sealed_is_not_swapped_in() {
        let dir = tempfile::tempdir().unwrap();
        let (install, pending) = fake_install(dir.path(), Platform::LinuxX64);
        // Planted in the staging folder after the checks, with no seal for it.
        fs::write(
            pending.payload.with_file_name(install.platform.host_name()),
            b"planted",
        )
        .unwrap();
        let installed_before = Some(b"old host".to_vec());
        fs::write(host_target(&install).unwrap(), b"old host").unwrap();
        apply(&pending, &install).unwrap();
        assert_eq!(installed(&install), "new");
        assert_eq!(installed_host(&install), installed_before);
    }

    #[test]
    fn a_changed_or_missing_staged_host_is_never_swapped_in() {
        for change in 0..3 {
            let dir = tempfile::tempdir().unwrap();
            let (install, pending) = fake_install(dir.path(), Platform::WindowsX64);
            let pending = with_host(&install, pending, Some(b"new host"), Some(b"old host"));
            let staged = stage::staged_host(&pending, install.platform).unwrap();
            match change {
                0 => fs::write(&staged, b"evil host").unwrap(),
                1 => fs::remove_file(&staged).unwrap(),
                _ => {
                    fs::remove_file(&staged).unwrap();
                    fs::create_dir(&staged).unwrap();
                }
            }
            assert!(matches!(
                apply(&pending, &install),
                Err(ApplyError::Tampered)
            ));
            assert_eq!(installed(&install), "old");
            assert_eq!(installed_host(&install).unwrap(), b"old host");
        }
    }

    #[test]
    fn a_failed_kanso_swap_puts_the_host_back() {
        for before in [Some(&b"old host"[..]), None] {
            let dir = tempfile::tempdir().unwrap();
            let (install, pending) = fake_install(dir.path(), Platform::LinuxX64);
            let pending = with_host(&install, pending, Some(b"new host"), before);
            let staged = stage::staged_host(&pending, install.platform).unwrap();
            // Kanso is gone from where it was, so its swap fails after the
            // host's has gone through.
            fs::remove_file(&install.target).unwrap();
            assert!(matches!(apply(&pending, &install), Err(ApplyError::Io(_))));
            assert_eq!(installed_host(&install).as_deref(), before);
            assert!(!install.target.exists());
            // The new host is back where it was staged, so a retry has it.
            assert_eq!(fs::read(&staged).unwrap(), b"new host");
            assert!(pending.payload.exists());
            let backup = backup_path(
                &host_target(&install).unwrap(),
                &install.stage_dir,
                install.platform,
            );
            assert!(!backup.exists());
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_host_that_cannot_be_swapped_changes_nothing() {
        use std::os::unix::fs::PermissionsExt as _;
        let dir = tempfile::tempdir().unwrap();
        let (install, pending) = fake_install(dir.path(), Platform::LinuxX64);
        let pending = with_host(&install, pending, Some(b"new host"), Some(b"old host"));
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o555)).unwrap();
        // Root can write anywhere; the check only means something for an
        // ordinary user.
        let blocked = !writable_dir(dir.path());
        let result = apply(&pending, &install);
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o755)).unwrap();
        if blocked {
            assert!(matches!(result, Err(ApplyError::Io(_))));
            assert_eq!(installed(&install), "old");
            assert_eq!(installed_host(&install).unwrap(), b"old host");
            assert!(pending.payload.exists());
        }
    }

    #[test]
    fn a_host_never_touches_a_bundle_swap() {
        let dir = tempfile::tempdir().unwrap();
        let (install, pending) = fake_install(dir.path(), Platform::MacosArm64);
        assert_eq!(host_target(&install), None);
        // A host file in the staging folder, sealed or not, is not looked at.
        fs::write(
            pending.payload.with_file_name("kanso-plugin-host"),
            b"stray",
        )
        .unwrap();
        assert_eq!(stage::staged_host(&pending, install.platform), None);
        apply(&pending, &install).unwrap();
        assert_eq!(installed(&install), "new");
        assert!(!dir.path().join("kanso-plugin-host").exists());
        // And a seal that claims one on a bundle is refused.
        let dir = tempfile::tempdir().unwrap();
        let (install, mut pending) = fake_install(dir.path(), Platform::MacosArm64);
        pending.seal.host_digest = Some("ab".repeat(32));
        assert!(matches!(
            apply(&pending, &install),
            Err(ApplyError::Tampered)
        ));
        assert_eq!(installed(&install), "old");
    }

    #[test]
    fn clean_up_removes_the_hosts_backup_too() {
        for platform in [Platform::LinuxX64, Platform::WindowsX64] {
            let dir = tempfile::tempdir().unwrap();
            let (install, _) = fake_install(dir.path(), platform);
            let host = host_target(&install).unwrap();
            fs::write(&host, "host").unwrap();
            let old = backup_path(&host, &install.stage_dir, platform);
            fs::write(&old, "previous host").unwrap();
            clean_up(&install);
            assert!(!old.exists(), "{platform:?}");
            assert_eq!(fs::read(&host).unwrap(), b"host", "the host itself stays");
        }
    }

    /// Where a swap is made to fail.
    #[derive(Clone, Copy, Debug)]
    enum Fault {
        None,
        /// Kanso isn't where it should be, so its swap fails.
        KansoGone,
        MissingStagedHost,
        MissingStagedKanso,
        ChangedStagedHost,
    }

    #[test]
    fn a_pending_update_for_elsewhere_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let (install, mut pending) = fake_install(dir.path(), Platform::MacosArm64);
        pending.target = "/Applications/Kanso.app".into();
        assert!(matches!(
            apply(&pending, &install),
            Err(ApplyError::WrongTarget { .. })
        ));
        let (install, mut pending) = fake_install(dir.path(), Platform::LinuxX64);
        pending.payload = dir.path().join("elsewhere");
        assert!(matches!(
            apply(&pending, &install),
            Err(ApplyError::Missing)
        ));
        assert_eq!(installed(&install), "old");
    }

    #[test]
    fn a_failed_swap_puts_the_old_one_back() {
        let dir = tempfile::tempdir().unwrap();
        let (install, pending) = fake_install(dir.path(), Platform::LinuxX64);
        let backup = backup_path(&install.target, &install.stage_dir, install.platform);
        // The staged file vanished between the check and the rename.
        let missing = dir.path().join("gone");
        assert!(swap_in(&missing, &install.target, &backup).is_err());
        assert_eq!(installed(&install), "old");
        assert!(!backup.exists());
        assert!(pending.payload.exists());
    }

    #[test]
    fn clean_up_removes_every_leftover() {
        let dir = tempfile::tempdir().unwrap();
        let (install, _) = fake_install(dir.path(), Platform::WindowsX64);
        let old = backup_path(&install.target, &install.stage_dir, install.platform);
        fs::write(&old, "previous").unwrap();
        clean_up(&install);
        assert!(!old.exists(), "the .old executable goes");
        assert!(
            !install.stage_dir.exists(),
            "an earlier run's update goes too"
        );
        assert_eq!(installed(&install), "old");
    }

    #[cfg(unix)]
    #[test]
    fn clean_up_never_follows_links() {
        let dir = tempfile::tempdir().unwrap();
        let (install, _) = fake_install(dir.path(), Platform::LinuxX64);
        fs::remove_dir_all(&install.stage_dir).unwrap();
        let victim = dir.path().join("victim");
        fs::create_dir(&victim).unwrap();
        fs::write(victim.join("keep"), "x").unwrap();
        std::os::unix::fs::symlink(&victim, &install.stage_dir).unwrap();
        let old = backup_path(&install.target, &install.stage_dir, install.platform);
        std::os::unix::fs::symlink(victim.join("keep"), &old).unwrap();
        clean_up(&install);
        assert!(victim.join("keep").exists());
        assert!(fs::symlink_metadata(&install.stage_dir).is_err());
        assert!(fs::symlink_metadata(&old).is_err());
    }

    #[test]
    fn a_planted_or_changed_update_is_never_swapped_in() {
        for platform in Platform::ALL {
            // Read back from disk, or made up: no seal.
            let dir = tempfile::tempdir().unwrap();
            let (install, mut pending) = fake_install(dir.path(), platform);
            pending.seal = stage::Seal::default();
            assert!(matches!(
                apply(&pending, &install),
                Err(ApplyError::Tampered)
            ));
            assert_eq!(installed(&install), "old");

            // Changed after it was checked.
            let dir = tempfile::tempdir().unwrap();
            let (install, pending) = fake_install(dir.path(), platform);
            let file = if platform.is_bundle() {
                pending.payload.join("Contents/version")
            } else {
                pending.payload.clone()
            };
            fs::write(&file, "evil").unwrap();
            assert!(matches!(
                apply(&pending, &install),
                Err(ApplyError::Tampered)
            ));
            assert_eq!(installed(&install), "old", "{platform:?}");

            // A file added to a bundle.
            if platform.is_bundle() {
                let dir = tempfile::tempdir().unwrap();
                let (install, pending) = fake_install(dir.path(), platform);
                fs::write(pending.payload.join("Contents/extra.dylib"), "evil").unwrap();
                assert!(matches!(
                    apply(&pending, &install),
                    Err(ApplyError::Tampered)
                ));
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_swapped_staging_folder_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let (install, pending) = fake_install(dir.path(), Platform::LinuxX64);
        // Someone moves the checked folder aside and puts an identical one,
        // theirs, in its place.
        let aside = dir.path().join("aside");
        fs::rename(&install.stage_dir, &aside).unwrap();
        fs::create_dir_all(pending.payload.parent().unwrap()).unwrap();
        fs::copy(aside.join("unpacked-0.2.0/kanso"), &pending.payload).unwrap();
        assert!(matches!(
            apply(&pending, &install),
            Err(ApplyError::Tampered)
        ));
        assert_eq!(installed(&install), "old");
    }

    #[test]
    fn a_payload_that_climbs_out_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let (install, mut pending) = fake_install(dir.path(), Platform::LinuxX64);
        let outside = dir.path().join("outside");
        fs::write(&outside, "evil").unwrap();
        pending.payload = install.stage_dir.join("..").join("outside");
        assert!(matches!(
            apply(&pending, &install),
            Err(ApplyError::Missing)
        ));
        assert_eq!(installed(&install), "old");
    }

    #[cfg(unix)]
    #[test]
    fn another_accounts_install_is_not_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("kanso");
        // A file this account doesn't own, if the system lets one be linked
        // here (another volume or a protected file won't).
        if fs::hard_link("/bin/sh", &exe).is_err() {
            return;
        }
        if owner_of(&exe) == probe_dir(dir.path()).flatten() {
            return; // Running as root.
        }
        assert_eq!(
            locate(&exe, Some(Platform::LinuxX64), opts()),
            Err(NotInstallable::OwnedByAnother(exe.clone()))
        );
    }

    #[test]
    fn team_ids_are_read_from_codesign() {
        let out = "Executable=/Applications/Kanso.app/Contents/MacOS/kanso\nAuthority=Developer ID Application: Ngonidzashe Mangudya (4973Y692JU)\nTeamIdentifier=4973Y692JU\n";
        assert_eq!(parse_team_id(out).as_deref(), Some("4973Y692JU"));
        assert_eq!(
            parse_team_id("Signature=adhoc\nTeamIdentifier=not set\n"),
            None
        );
        assert_eq!(parse_team_id(""), None);
        let out =
            "Executable=/x\nIdentifier=app.kanso\nFormat=app bundle\nTeamIdentifier=4973Y692JU\n";
        assert_eq!(parse_identifier(out).as_deref(), Some("app.kanso"));
        assert_eq!(parse_identifier("TeamIdentifier=4973Y692JU\n"), None);
    }

    proptest! {
        /// Every swap ends with exactly one of the two copies installed and
        /// nothing lost: the new one when the staged copy was there, the
        /// old one when it wasn't.
        #[test]
        fn prop_swaps_never_lose_kanso(staged_exists in any::<bool>(), bundle in any::<bool>(), backup_left in any::<bool>()) {
            let dir = tempfile::tempdir().unwrap();
            let platform = if bundle { Platform::MacosArm64 } else { Platform::LinuxArm64 };
            let (install, pending) = fake_install(dir.path(), platform);
            let backup = backup_path(&install.target, &install.stage_dir, platform);
            if backup_left {
                // An earlier run's leftovers.
                if bundle { fs::create_dir_all(&backup).unwrap() } else { fs::write(&backup, "older").unwrap() }
            }
            if !staged_exists {
                remove_any(&pending.payload).unwrap();
            }
            let result = apply(&pending, &install);
            prop_assert_eq!(result.is_ok(), staged_exists);
            prop_assert_eq!(installed(&install), if staged_exists { "new" } else { "old" });
        }

        /// A swap with a host ends whole or not at all: both installed
        /// files are the staged ones, or both are what they were (a host
        /// that was absent stays absent), wherever it was made to fail.
        #[test]
        fn prop_a_host_swap_is_whole_or_nothing(
            windows in any::<bool>(),
            kanso in prop::collection::vec(any::<u8>(), 0..64),
            old_host in prop::option::of(prop::collection::vec(any::<u8>(), 0..64)),
            new_host in prop::option::of(prop::collection::vec(any::<u8>(), 0..64)),
            fault in prop::sample::select(vec![
                Fault::None,
                Fault::KansoGone,
                Fault::MissingStagedHost,
                Fault::MissingStagedKanso,
                Fault::ChangedStagedHost,
            ]),
            backups_left in any::<bool>(),
        ) {
            let platform = if windows { Platform::WindowsX64 } else { Platform::LinuxX64 };
            let dir = tempfile::tempdir().unwrap();
            let (install, pending) = fake_install(dir.path(), platform);
            fs::write(&pending.payload, &kanso).unwrap();
            let mut pending = pending;
            pending.seal = seal(&install.stage_dir, &pending.payload);
            let pending = with_host(&install, pending, new_host.as_deref(), old_host.as_deref());
            let host = host_target(&install).unwrap();
            if backups_left {
                fs::write(backup_path(&host, &install.stage_dir, platform), b"older").unwrap();
            }
            let staged_host = pending.payload.with_file_name(platform.host_name());
            let old_kanso = fs::read(&install.target).unwrap();
            let mut expect_ok = true;
            match fault {
                Fault::None => {}
                Fault::KansoGone => {
                    fs::remove_file(&install.target).unwrap();
                    expect_ok = false;
                }
                Fault::MissingStagedHost if new_host.is_some() => {
                    fs::remove_file(&staged_host).unwrap();
                    expect_ok = false;
                }
                Fault::ChangedStagedHost if new_host.is_some() => {
                    fs::write(&staged_host, b"changed").unwrap();
                    expect_ok = false;
                }
                Fault::MissingStagedKanso => {
                    fs::remove_file(&pending.payload).unwrap();
                    expect_ok = false;
                }
                _ => {}
            }
            let result = apply(&pending, &install);
            prop_assert_eq!(result.is_ok(), expect_ok, "{:?}", result);
            if expect_ok {
                prop_assert_eq!(fs::read(&install.target).unwrap(), kanso);
                prop_assert_eq!(installed_host(&install), new_host.clone().or(old_host.clone()));
                prop_assert!(!install.stage_dir.exists());
            } else {
                match fault {
                    Fault::KansoGone => prop_assert!(!install.target.exists()),
                    _ => prop_assert_eq!(fs::read(&install.target).unwrap(), old_kanso),
                }
                prop_assert_eq!(installed_host(&install), old_host.clone());
            }
        }

        /// Only Cargo's own output folders count as a cargo build.
        #[test]
        fn prop_cargo_builds_are_recognised(
            prefix in prop::collection::vec("[a-s]{1,8}", 0..4),
            profile in prop::sample::select(vec!["debug", "release", "Applications", "bin"]),
            triple in any::<bool>(),
        ) {
            let mut path = PathBuf::from("/");
            for p in &prefix { path.push(p); }
            path.push("target");
            if triple { path.push("aarch64-apple-darwin"); }
            path.push(profile);
            path.push("kanso");
            let expect = profile == "debug" || profile == "release";
            prop_assert_eq!(is_cargo_build(&path), expect);
            // Without a target folder, never.
            let plain: PathBuf = ["/", "Applications", profile, "kanso"].iter().collect();
            prop_assert!(!is_cargo_build(&plain));
        }

        /// Only names inside the staging folder, by plain names, pass.
        #[test]
        fn prop_payloads_stay_inside_the_staging_folder(
            parts in prop::collection::vec(prop::sample::select(vec!["a", "Kanso.app", "..", "."]), 0..5),
        ) {
            let stage = Path::new("/Applications/.kanso-update");
            let mut path = stage.to_path_buf();
            for p in &parts { path.push(p); }
            let plain: Vec<&&str> = parts.iter().filter(|p| **p != ".").collect();
            let expect = !plain.is_empty() && !plain.contains(&&"..");
            prop_assert_eq!(is_inside(&path, stage), expect, "{}", path.display());
        }

        /// Team IDs come back exactly, whatever surrounds them.
        #[test]
        fn prop_team_ids_round_trip(id in "[A-Z0-9]{10}", before in "[a-z/ ]{0,30}") {
            let out = format!("{before}\nTeamIdentifier={id}\nSealed Resources version=2\n");
            prop_assert_eq!(parse_team_id(&out), Some(id));
        }
    }
}
