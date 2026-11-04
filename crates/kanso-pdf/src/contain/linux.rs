//! Linux: Landlock for files, seccomp for sockets and new programs.
//!
//! Landlock (kernel 5.13 and later) lets a process give up its own access to
//! the file system; here it keeps only reads under the library and font
//! folders. The seccomp filter then refuses, with `EPERM`, the calls a
//! renderer has no use for: sockets, `exec`, `ptrace`, mounts and the like.
//! `no_new_privs` is set first, as both need. A kernel without Landlock
//! cannot contain the worker, so there the worker refuses to start.

use std::collections::BTreeMap;
use std::path::Path;

use landlock::{
    ABI, Access, AccessFs, PathBeneath, PathFd, Ruleset, RulesetAttr, RulesetCreatedAttr,
    RulesetStatus,
};
use seccompiler::{BpfProgram, SeccompAction, SeccompFilter, SeccompRule, TargetArch};

use super::{Allow, ContainError};

/// What the system's libraries and fonts need to be read: the loader opens
/// PDFium's dependencies after the worker is contained.
const SYSTEM_READS: &[&str] = &[
    "/lib",
    "/lib64",
    "/usr/lib",
    "/usr/lib64",
    "/usr/share/fonts",
    "/usr/share/X11/fonts",
    "/usr/local/share/fonts",
    "/usr/share/zoneinfo",
    "/etc/fonts",
];

/// Calls the worker never makes: the network, new programs, other
/// processes' memory, and anything that changes the system.
fn denied_calls() -> Vec<i64> {
    let mut calls = vec![
        libc::SYS_socket,
        libc::SYS_socketpair,
        libc::SYS_connect,
        libc::SYS_bind,
        libc::SYS_listen,
        libc::SYS_accept,
        libc::SYS_accept4,
        libc::SYS_execve,
        libc::SYS_execveat,
        libc::SYS_ptrace,
        libc::SYS_process_vm_readv,
        libc::SYS_process_vm_writev,
        libc::SYS_mount,
        libc::SYS_umount2,
        libc::SYS_pivot_root,
        libc::SYS_chroot,
        libc::SYS_bpf,
        libc::SYS_userfaultfd,
        libc::SYS_keyctl,
        libc::SYS_add_key,
        libc::SYS_request_key,
        libc::SYS_init_module,
        libc::SYS_finit_module,
        libc::SYS_delete_module,
        libc::SYS_kexec_load,
        libc::SYS_perf_event_open,
        libc::SYS_open_by_handle_at,
        libc::SYS_name_to_handle_at,
        libc::SYS_io_uring_setup,
        libc::SYS_unshare,
        libc::SYS_setns,
    ];
    calls.sort_unstable();
    calls.dedup();
    calls
}

fn arch() -> Result<TargetArch, ContainError> {
    std::env::consts::ARCH
        .try_into()
        .map_err(|_| ContainError("this processor has no seccomp filter".into()))
}

fn seccomp_program() -> Result<BpfProgram, ContainError> {
    let rules: BTreeMap<i64, Vec<SeccompRule>> = denied_calls()
        .into_iter()
        .map(|call| (call, Vec::new()))
        .collect();
    let filter = SeccompFilter::new(
        rules,
        SeccompAction::Allow,
        SeccompAction::Errno(libc::EPERM as u32),
        arch()?,
    )
    .map_err(|e| ContainError(format!("couldn't build the seccomp filter: {e}")))?;
    filter.try_into().map_err(|e: seccompiler::BackendError| {
        ContainError(format!("couldn't compile the seccomp filter: {e}"))
    })
}

pub(super) fn apply(allow: &Allow) -> Result<(), ContainError> {
    // Both Landlock and seccomp need this, and it can't be undone.
    // SAFETY: a plain prctl with constant arguments.
    if unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) } != 0 {
        return Err(ContainError(format!(
            "couldn't set no_new_privs: {}",
            std::io::Error::last_os_error()
        )));
    }

    let abi = ABI::V3;
    let landlock_error = |e: &dyn std::fmt::Display| ContainError(format!("Landlock: {e}"));
    let mut ruleset = Ruleset::default()
        .handle_access(AccessFs::from_all(abi))
        .map_err(|e| landlock_error(&e))?
        .create()
        .map_err(|e| landlock_error(&e))?;
    let reads = AccessFs::from_read(abi);
    for dir in allow
        .read_dirs
        .iter()
        .map(Path::new)
        .chain(SYSTEM_READS.iter().map(Path::new))
    {
        // A folder this system doesn't have is not a rule.
        let Ok(fd) = PathFd::new(dir) else { continue };
        ruleset = ruleset
            .add_rule(PathBeneath::new(fd, reads))
            .map_err(|e| landlock_error(&e))?;
    }
    let status = ruleset.restrict_self().map_err(|e| landlock_error(&e))?;
    if status.ruleset == RulesetStatus::NotEnforced {
        return Err(ContainError(
            "this kernel does not support Landlock, so the PDF reader can't be contained".into(),
        ));
    }

    let program = seccomp_program()?;
    seccompiler::apply_filter(&program)
        .map_err(|e| ContainError(format!("couldn't apply the seccomp filter: {e}")))
}

pub(super) fn verify() -> Result<(), ContainError> {
    super::verify_unix()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_filter_compiles_and_denies_sockets_and_exec() {
        let calls = denied_calls();
        assert!(calls.contains(&libc::SYS_socket));
        assert!(calls.contains(&libc::SYS_connect));
        assert!(calls.contains(&libc::SYS_execve));
        // Threads (clone) and memory (mmap) stay: PDFium needs them.
        assert!(!calls.contains(&libc::SYS_clone));
        assert!(!calls.contains(&libc::SYS_mmap));
        assert!(seccomp_program().is_ok());
    }
}
