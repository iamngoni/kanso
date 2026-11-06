//! `kanso update`: checks for a newer release and installs it from the
//! command line, without opening a window or the library.
//!
//! It is the same updater as Settings › Updates (`updates.rs`): the feed is
//! fetched from [`kanso_update::FEED_URL`] only, its signature is checked
//! against the keys compiled into this build, the download's size and
//! SHA-256 against the signed feed, and on macOS the new app's code
//! signature against this one's. Nothing here accepts another feed or key.
//! Because it never starts the app, it also works when the installed Kanso
//! can't get past its first frame.

use std::io::{IsTerminal as _, Write as _};
use std::sync::atomic::AtomicBool;

use anyhow::{Result, bail};
use kanso_update::{Check, LocateOptions, Platform, UrlPolicy};

use crate::flavour;
use crate::updates::{current_version, off_reason, percent};

const HELP: &str = "\
Usage: kanso update [--check] [--yes]

Checks for a newer Kanso and installs it. Kanso doesn't need to be open,
so this works even when the app won't start.

  --check   Only say whether there is a newer version. Exits 0 when this
            one is the newest, 10 when there is a newer one.
  --yes     Install without asking.

The new version is used the next time Kanso starts. A running Kanso keeps
its version until you quit it.
";

/// The exit code of `kanso update --check` when a newer version exists.
pub const EXIT_UPDATE_AVAILABLE: i32 = 10;

/// What was asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Help,
    Check,
    Install { yes: bool },
}

/// Reads `kanso update`'s arguments. Pure.
pub fn parse(args: &[String]) -> Result<Mode> {
    let (mut check, mut yes) = (false, false);
    for arg in args {
        match arg.as_str() {
            "--check" => check = true,
            "--yes" | "-y" => yes = true,
            "help" | "--help" | "-h" => return Ok(Mode::Help),
            other => bail!("update doesn't take `{other}`\n\n{HELP}"),
        }
    }
    if check && yes {
        bail!("--check only checks; leave out --yes\n\n{HELP}");
    }
    Ok(if check {
        Mode::Check
    } else {
        Mode::Install { yes }
    })
}

/// Whether an answer to "Install it?" means yes. Pure.
pub fn is_yes(answer: &str) -> bool {
    matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

// no-prop-test: network and file system wiring; `parse`, `is_yes` and the
// `kanso_update` steps it calls are tested.
pub fn run_cli(args: &[String]) -> Result<()> {
    let mode = parse(args)?;
    if mode == Mode::Help {
        println!("{HELP}");
        return Ok(());
    }
    let current = current_version();
    let keys = kanso_update::trusted_keys();
    let platform = Platform::current();
    if let Some(reason) = off_reason(&flavour::CURRENT, !keys.is_empty(), platform) {
        bail!("{reason}");
    }
    let Some(platform) = platform else {
        bail!("Kanso isn't released for this system.");
    };
    let transport = kanso_update::HttpTransport::new(&format!("Kanso/{current}"));
    let policy = UrlPolicy::RELEASE;
    eprintln!("Checking for updates (this is Kanso {current})…");
    let check = kanso_update::check(
        &transport,
        &policy,
        kanso_update::FEED_URL,
        &keys,
        &current,
        platform,
    )
    .map_err(|err| anyhow::anyhow!("couldn't check for updates: {err}"))?;
    let update = match check {
        Check::UpToDate { latest } => {
            println!("Kanso {current} is the newest version (latest release: {latest}).");
            return Ok(());
        }
        Check::Available(update) => update,
    };
    println!(
        "Kanso {} is available (you have {current}).",
        update.version
    );
    println!("Release notes: {}", update.release_url);
    if mode == Mode::Check {
        std::io::stdout().flush().ok();
        std::process::exit(EXIT_UPDATE_AVAILABLE);
    }

    let exe = std::env::current_exe()?;
    let install = kanso_update::install::locate(
        &exe,
        Some(platform),
        LocateOptions {
            dev_flavour: flavour::CURRENT.is_dev(),
            debug_build: cfg!(debug_assertions),
            appimage: cfg!(target_os = "linux") && std::env::var_os("APPIMAGE").is_some(),
        },
    )
    .map_err(|why| anyhow::anyhow!("{} Download it from {}", why.reason(), update.release_url))?;

    if mode == (Mode::Install { yes: false }) {
        if !std::io::stdin().is_terminal() {
            bail!("not installing without --yes when nobody can answer");
        }
        print!(
            "Install Kanso {} into {}? [y/N] ",
            update.version,
            install.target.display()
        );
        std::io::stdout().flush().ok();
        let mut answer = String::new();
        std::io::stdin().read_line(&mut answer)?;
        if !is_yes(&answer) {
            println!("Not installed.");
            return Ok(());
        }
    }

    let mut last = None;
    let mut progress = |done: u64, total: u64| {
        let now = percent(done, total);
        if last != Some(now) {
            last = Some(now);
            eprint!("\rDownloading… {now}%");
        }
    };
    let verify = |payload: &std::path::Path| -> Result<(), String> {
        #[cfg(target_os = "macos")]
        return kanso_update::install::verify_bundle(payload, Some(&install.target));
        #[cfg(not(target_os = "macos"))]
        {
            let _ = payload;
            Ok(())
        }
    };
    let cancel = AtomicBool::new(false);
    let pending = kanso_update::stage::prepare(
        &transport,
        &policy,
        &update,
        &install,
        &mut progress,
        &cancel,
        &verify,
    );
    eprintln!();
    let pending = pending.map_err(|err| anyhow::anyhow!("couldn't download the update: {err}"))?;
    kanso_update::install::apply(&pending, &install)
        .map_err(|err| anyhow::anyhow!("couldn't install the update: {err}"))?;
    #[cfg(target_os = "macos")]
    kanso_update::install::remove_quarantine(&install.target);
    println!(
        "Installed Kanso {}. It's used the next time Kanso starts; quit a running Kanso to switch.",
        update.version
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn parses_the_modes() {
        assert_eq!(parse(&[]).unwrap(), Mode::Install { yes: false });
        assert_eq!(
            parse(&args(&["--yes"])).unwrap(),
            Mode::Install { yes: true }
        );
        assert_eq!(parse(&args(&["-y"])).unwrap(), Mode::Install { yes: true });
        assert_eq!(parse(&args(&["--check"])).unwrap(), Mode::Check);
        assert_eq!(parse(&args(&["--help"])).unwrap(), Mode::Help);
        assert!(parse(&args(&["--check", "--yes"])).is_err());
        assert!(parse(&args(&["--feed", "https://example.com"])).is_err());
    }

    proptest! {
        /// No argument outside the documented ones is ever accepted, so a
        /// feed URL or key can't be slipped in.
        #[test]
        fn prop_unknown_arguments_are_refused(arg in "[a-z/:.=-]{1,20}") {
            prop_assume!(!matches!(
                arg.as_str(),
                "--check" | "--yes" | "-y" | "help" | "--help" | "-h"
            ));
            prop_assert!(parse(&[arg]).is_err());
        }

        /// Only y or yes, in any case and with spaces around, means yes.
        #[test]
        fn prop_only_y_or_yes_is_yes(answer in "[ a-zA-Z]{0,6}") {
            let word = answer.trim().to_ascii_lowercase();
            prop_assert_eq!(is_yes(&answer), word == "y" || word == "yes");
        }
    }
}
