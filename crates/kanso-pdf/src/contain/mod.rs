//! OS containment of the PDF worker (issue #43, step 3).
//!
//! A separate process is not a sandbox on its own, and PDFium is C++ that
//! parses hostile files. The worker therefore contains itself before it
//! loads PDFium or reads a byte of any document, and refuses to run if it
//! cannot:
//!
//! | OS | Mechanism |
//! |---|---|
//! | macOS | A deny-by-default Seatbelt profile applied to the process (`sandbox_init`): files only under the worker's own library folders and the system's fonts, no network, no new processes |
//! | Linux | Landlock (reads only under the library and font folders; TCP, signals and abstract sockets scoped where the kernel has them) plus a seccomp denylist (sockets and socket I/O, `exec`, `fork` and namespace-making `clone`, `clone3` as unknown, `ptrace`, other processes' memory, descriptors, signals and limits, mounts, `bpf`, `io_uring`, `userfaultfd`, `perf_event_open`, keys, modules) |
//! | Windows | An AppContainer token with no capabilities (the app starts the worker in one, see [`crate::client`]) inside a Job Object that limits memory and forbids child processes |
//!
//! Every descriptor above stderr is closed first: a sandbox checks a file
//! when it is opened, so one the app left inheritable would stay readable.
//!
//! After applying its mechanism the worker checks that it took hold (it can
//! no longer open a file outside its folders or create a socket), and a
//! mechanism that did not take is the same as one that could not be applied.

use std::path::PathBuf;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
pub(crate) mod windows;

#[cfg(target_os = "macos")]
use macos as platform;

#[cfg(target_os = "linux")]
use linux as platform;

#[cfg(windows)]
use windows as platform;

/// What the worker may still read once contained.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Allow {
    /// Folders that may be read, and code in them loaded: the worker's own
    /// folder and the one that holds the PDFium library.
    pub read_dirs: Vec<PathBuf>,
}

/// Why the worker could not be contained.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct ContainError(pub String);

/// The name of the mechanism this build uses, for logs and the PR.
pub const MECHANISM: &str = if cfg!(target_os = "macos") {
    "Seatbelt profile"
} else if cfg!(target_os = "linux") {
    "Landlock and seccomp"
} else if cfg!(windows) {
    "AppContainer and Job Object"
} else {
    "none"
};

/// The most memory the worker may use before it is stopped: 1 GiB.
pub const MEMORY_BUDGET: u64 = 1024 * 1024 * 1024;

/// The variable a test sets to make containment fail, so the refusal to
/// start can be tested where containment works. It can only make the worker
/// refuse; the app starts the worker with an empty environment.
pub const FAIL_FOR_TEST: &str = "KANSO_PDF_WORKER_FAIL_CONTAINMENT";

/// Contains this process. On `Ok` the worker is contained and has checked
/// that it is; on `Err` it must not go on.
#[cfg(any(target_os = "macos", target_os = "linux", windows))]
pub fn contain(allow: &Allow) -> Result<(), ContainError> {
    if std::env::var_os(FAIL_FOR_TEST).is_some() {
        return Err(ContainError("containment was made to fail".into()));
    }
    // A sandbox checks a file when it is opened, not a descriptor that is
    // already open: anything the app left inheritable (a database, a socket)
    // would still be readable inside it. Only the protocol's pipes stay.
    close_inherited()?;
    limit_memory()?;
    platform::apply(allow)?;
    platform::verify()
}

/// This system has no containment for the worker, so it does not run.
#[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]
pub fn contain(_allow: &Allow) -> Result<(), ContainError> {
    Err(ContainError(
        "PDFs can't be read safely on this system".into(),
    ))
}

/// Caps the address space the worker may grow to. Linux enforces it; macOS
/// treats it as advice for some allocations, so the worker also checks its
/// resident size between requests (see `worker`); Windows has the Job
/// Object's limit instead.
#[cfg(unix)]
fn limit_memory() -> Result<(), ContainError> {
    // Address space is more than memory: PDFium and the system libraries
    // map a good deal that is never touched. Four times the budget leaves
    // room for them and still stops a runaway.
    let cap = (MEMORY_BUDGET * 4) as libc::rlim_t;
    let limit = libc::rlimit {
        rlim_cur: cap,
        rlim_max: cap,
    };
    // SAFETY: `limit` is a valid rlimit for the call's duration.
    let result = unsafe { libc::setrlimit(libc::RLIMIT_AS, &limit) };
    if result != 0 && cfg!(target_os = "linux") {
        return Err(ContainError(format!(
            "couldn't limit memory: {}",
            std::io::Error::last_os_error()
        )));
    }
    Ok(())
}

#[cfg(not(unix))]
fn limit_memory() -> Result<(), ContainError> {
    Ok(())
}

/// Closes every descriptor but stdin, stdout and stderr. The worker has
/// opened nothing yet, so whatever is above 2 came from whoever started it.
#[cfg(unix)]
fn close_inherited() -> Result<(), ContainError> {
    let listed = if cfg!(target_os = "linux") {
        "/proc/self/fd"
    } else {
        "/dev/fd"
    };
    let open: Option<Vec<libc::c_int>> = std::fs::read_dir(listed).ok().map(|dir| {
        dir.filter_map(|entry| entry.ok()?.file_name().to_str()?.parse().ok())
            .collect()
    });
    match open {
        // The listing's own descriptor is among them, closed already; closing
        // it again is a harmless EBADF.
        Some(fds) => {
            for fd in fds.into_iter().filter(|fd| *fd > 2) {
                // SAFETY: closing a descriptor this process does not use.
                unsafe { libc::close(fd) };
            }
        }
        // No listing (no /proc): every number up to the descriptor limit.
        None => {
            // SAFETY: plain sysconf.
            let max = unsafe { libc::sysconf(libc::_SC_OPEN_MAX) };
            let max = if max <= 0 { 65_536 } else { max.min(1 << 20) };
            for fd in 3..max as libc::c_int {
                // SAFETY: as above.
                unsafe { libc::close(fd) };
            }
        }
    }
    // Check: nothing above 2 may be left.
    // SAFETY: fcntl on a number that may not be open just fails.
    let left = (3..1024).any(|fd| unsafe { libc::fcntl(fd, libc::F_GETFD) } != -1);
    if left {
        return Err(ContainError(
            "an inherited file is still open in the PDF reader".into(),
        ));
    }
    Ok(())
}

#[cfg(not(unix))]
fn close_inherited() -> Result<(), ContainError> {
    // The app hands the worker only its pipes (a handle list, see
    // `client::launch_windows`).
    Ok(())
}

/// The resident memory of this process in bytes, where the system says.
#[cfg(unix)]
pub fn resident_bytes() -> Option<u64> {
    // SAFETY: `usage` is zeroed plain data the call fills in.
    let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
    if unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut usage) } != 0 {
        return None;
    }
    // Bytes on macOS, kilobytes elsewhere.
    let max = u64::try_from(usage.ru_maxrss).ok()?;
    Some(if cfg!(target_os = "macos") {
        max
    } else {
        max * 1024
    })
}

#[cfg(not(unix))]
pub fn resident_bytes() -> Option<u64> {
    None
}

/// Runs `load` only once `contain` has succeeded: the worker never touches
/// PDFium, or a document, uncontained.
pub fn contained_then<T>(
    contain: impl FnOnce() -> Result<(), ContainError>,
    load: impl FnOnce() -> Result<T, ContainError>,
) -> Result<T, ContainError> {
    contain()?;
    load()
}

/// Checks, with plain std calls, that the process cannot do what containment
/// forbids on Unix: read a file outside its folders or open a socket.
#[cfg(unix)]
pub(crate) fn verify_unix() -> Result<(), ContainError> {
    if std::fs::File::open("/etc/hosts").is_ok() {
        return Err(ContainError(
            "the sandbox did not take hold: a file outside the worker's folders can be read".into(),
        ));
    }
    // Reaching the network stack at all is the failure: a socket that can
    // be made and connected (even to be refused) means no sandbox. Seatbelt
    // lets a socket be made but not used; seccomp refuses the socket.
    // SAFETY: plain syscalls with constant arguments, on a descriptor that is
    // closed again here.
    unsafe {
        let fd = libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0);
        if fd < 0 {
            return Ok(());
        }
        let mut address: libc::sockaddr_in = std::mem::zeroed();
        address.sin_family = libc::AF_INET as libc::sa_family_t;
        address.sin_port = 9u16.to_be();
        address.sin_addr.s_addr = u32::from_ne_bytes([127, 0, 0, 1]);
        let connected = libc::connect(
            fd,
            (&address as *const libc::sockaddr_in).cast(),
            std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t,
        );
        let denied = connected != 0
            && matches!(
                std::io::Error::last_os_error().raw_os_error(),
                Some(libc::EPERM | libc::EACCES)
            );
        libc::close(fd);
        if !denied {
            return Err(ContainError(
                "the sandbox did not take hold: the network can be reached".into(),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn the_worker_does_not_load_without_containment() {
        let loaded = Cell::new(false);
        let result: Result<(), _> = contained_then(
            || Err(ContainError("no".into())),
            || {
                loaded.set(true);
                Ok(())
            },
        );
        assert_eq!(result, Err(ContainError("no".into())));
        assert!(!loaded.get(), "PDFium was loaded uncontained");
    }

    #[test]
    fn the_worker_loads_after_containment() {
        let order = std::cell::RefCell::new(Vec::new());
        let result = contained_then(
            || {
                order.borrow_mut().push("contain");
                Ok(())
            },
            || {
                order.borrow_mut().push("load");
                Ok(7)
            },
        );
        assert_eq!(result, Ok(7));
        assert_eq!(*order.borrow(), ["contain", "load"]);
    }
}
