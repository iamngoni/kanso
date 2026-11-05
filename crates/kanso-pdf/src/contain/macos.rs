//! macOS: a Seatbelt profile applied to the worker itself.
//!
//! `sandbox_init` is the call Chromium, Firefox and Apple's own helpers use
//! to confine a process after start-up. It is marked deprecated in the SDK
//! (Apple steers apps to the App Sandbox entitlement) but is not going away,
//! and unlike a `sandbox-exec` wrapper it is applied by the worker, so a
//! worker that is started some other way still contains itself or refuses.
//! The profile is deny-by-default: reads under the allowed folders and the
//! system's fonts and frameworks, named sysctls, nothing else, no network,
//! no processes. Descriptors open before it is applied are not checked by
//! it, so the worker closes every inherited one first (`contain`).

use std::ffi::{CStr, CString, c_char, c_int};
use std::path::Path;

use super::{Allow, ContainError};

unsafe extern "C" {
    fn sandbox_init(profile: *const c_char, flags: u64, errorbuf: *mut *mut c_char) -> c_int;
    fn sandbox_free_error(errorbuf: *mut c_char);
}

/// What the system's own code needs to run and draw text: the OS, its
/// libraries and fonts. Nothing in a user's home or in /Users, /Volumes,
/// /private or /tmp.
const SYSTEM_READS: &[&str] = &[
    "/System",
    "/usr/lib",
    "/usr/share",
    "/Library/Apple",
    "/Library/Fonts",
    "/private/var/db/dyld",
];

/// Sysctls the C library, the allocator and PDFium read: the processor and
/// the OS version. Never `kern.proc*`, `kern.procargs*` or the host name.
const SYSCTL_PREFIXES: &[&str] = &["hw.", "kern.os"];
const SYSCTL_NAMES: &[&str] = &[
    "kern.version",
    "kern.argmax",
    "kern.maxfilesperproc",
    "kern.usrstack64",
    "kern.secure_kernel",
    "kern.hv_vmm_present",
    "sysctl.proc_cputype",
    "sysctl.proc_native",
    "vm.footprint_suspend",
];

/// The profile text for these allowed folders. Pure.
pub(super) fn profile(allow: &Allow) -> Result<String, ContainError> {
    let mut text = String::from("(version 1)\n(deny default)\n");
    // Reading the stdin/stdout pipes needs nothing; these are for dyld and
    // the C library at start-up and for PDFium's font and locale lookups.
    text.push_str("(allow file-read* file-map-executable\n");
    for dir in SYSTEM_READS {
        text.push_str(&format!("    (subpath {})\n", quote(Path::new(dir))?));
    }
    for dir in &allow.read_dirs {
        text.push_str(&format!("    (subpath {})\n", quote(dir)?));
    }
    text.push_str(")\n");
    text.push_str(
        "(allow file-read-metadata (literal \"/\") (literal \"/private\") (literal \"/Library\") (literal \"/var\") (literal \"/tmp\") (literal \"/etc\"))\n",
    );
    text.push_str("(allow file-read* file-write* (literal \"/dev/null\"))\n");
    text.push_str("(allow file-read* (literal \"/dev/urandom\") (literal \"/dev/random\"))\n");
    // Named system facts only. A blanket `sysctl-read` would hand over the
    // process table and, through `kern.procargs2`, the arguments and
    // environment of every program the user runs, the app included.
    text.push_str("(allow sysctl-read\n");
    for name in SYSCTL_PREFIXES {
        text.push_str(&format!("    (sysctl-name-prefix \"{name}\")\n"));
    }
    for name in SYSCTL_NAMES {
        text.push_str(&format!("    (sysctl-name \"{name}\")\n"));
    }
    text.push_str(")\n");
    text.push_str("(allow signal (target self))\n");
    text.push_str("(allow process-info-pidinfo (target self))\n");
    text.push_str("(allow mach-lookup (global-name \"com.apple.system.notification_center\"))\n");
    Ok(text)
}

/// A path as a Seatbelt string: absolute, no control characters, quotes and
/// backslashes escaped.
fn quote(path: &Path) -> Result<String, ContainError> {
    let text = path
        .to_str()
        .filter(|text| text.starts_with('/') && !text.chars().any(char::is_control))
        .ok_or_else(|| ContainError(format!("an unusable folder for the sandbox: {path:?}")))?;
    let mut out = String::from("\"");
    for c in text.chars() {
        if matches!(c, '"' | '\\') {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('"');
    Ok(out)
}

pub(super) fn apply(allow: &Allow) -> Result<(), ContainError> {
    let profile = CString::new(profile(allow)?)
        .map_err(|_| ContainError("the sandbox profile has a stray NUL".into()))?;
    let mut error: *mut c_char = std::ptr::null_mut();
    // SAFETY: `profile` is a NUL-terminated string; `error` receives either
    // null or a string the system allocated and `sandbox_free_error` frees.
    let result = unsafe { sandbox_init(profile.as_ptr(), 0, &mut error) };
    if result == 0 {
        return Ok(());
    }
    let message = if error.is_null() {
        "unknown error".to_owned()
    } else {
        // SAFETY: a non-null `error` is a NUL-terminated string.
        let text = unsafe { CStr::from_ptr(error) }
            .to_string_lossy()
            .into_owned();
        // SAFETY: allocated by `sandbox_init`, freed once.
        unsafe { sandbox_free_error(error) };
        text
    };
    Err(ContainError(format!(
        "couldn't apply the sandbox: {message}"
    )))
}

pub(super) fn verify() -> Result<(), ContainError> {
    super::verify_unix()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::path::PathBuf;

    proptest! {
        /// Whatever folder names come in, the profile's quotes balance: a
        /// name can't close its string and add rules of its own.
        #[test]
        fn a_folder_name_cannot_inject_rules(name in "[ -~]{0,30}") {
            let allow = Allow { read_dirs: vec![PathBuf::from(format!("/opt/{name}"))] };
            let text = profile(&allow).unwrap();
            // Unescaped quotes come in pairs.
            let mut open = false;
            let mut escaped = false;
            for c in text.chars() {
                match (escaped, c) {
                    (true, _) => escaped = false,
                    (false, '\\') => escaped = true,
                    (false, '"') => open = !open,
                    _ => {}
                }
            }
            prop_assert!(!open, "{text}");
        }
    }

    #[test]
    fn a_relative_or_odd_folder_is_refused() {
        for dir in ["lib", "", "/a\nb"] {
            let allow = Allow {
                read_dirs: vec![PathBuf::from(dir)],
            };
            assert!(profile(&allow).is_err(), "{dir:?}");
        }
    }

    #[test]
    fn the_profile_denies_by_default_and_names_no_home() {
        let text = profile(&Allow {
            read_dirs: vec![PathBuf::from("/Applications/Kanso.app/Contents/Frameworks")],
        })
        .unwrap();
        assert!(text.contains("(deny default)"));
        assert!(text.contains("/Applications/Kanso.app/Contents/Frameworks"));
        assert!(!text.contains("network"));
        assert!(!text.contains("/Users"));
        // Sysctls by name only: no process table, no host name.
        assert!(!text.contains("(allow sysctl-read)"));
        assert!(!text.contains("kern.proc"));
        assert!(!text.contains("kern.hostname"));
    }
}
