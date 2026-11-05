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
    ABI, Access, AccessFs, AccessNet, PathBeneath, PathFd, Ruleset, RulesetAttr,
    RulesetCreatedAttr, RulesetStatus, Scope,
};
use seccompiler::{
    BpfProgram, SeccompAction, SeccompCmpArgLen, SeccompCmpOp, SeccompCondition, SeccompFilter,
    SeccompRule, TargetArch,
};

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

/// Calls the worker never makes: the network (and any socket it might have
/// been handed), new programs and processes, other processes' memory,
/// descriptors and limits, kernel attack surface it has no use for, and
/// anything that changes the system.
fn denied_calls() -> Vec<i64> {
    let mut calls = vec![
        // Sockets, made or inherited.
        libc::SYS_socket,
        libc::SYS_socketpair,
        libc::SYS_connect,
        libc::SYS_bind,
        libc::SYS_listen,
        libc::SYS_accept,
        libc::SYS_accept4,
        libc::SYS_sendto,
        libc::SYS_sendmsg,
        libc::SYS_sendmmsg,
        libc::SYS_recvfrom,
        libc::SYS_recvmsg,
        libc::SYS_recvmmsg,
        // New programs.
        libc::SYS_execve,
        libc::SYS_execveat,
        // Other processes.
        libc::SYS_ptrace,
        libc::SYS_process_vm_readv,
        libc::SYS_process_vm_writev,
        libc::SYS_process_madvise,
        libc::SYS_kcmp,
        libc::SYS_pidfd_open,
        libc::SYS_pidfd_getfd,
        libc::SYS_pidfd_send_signal,
        libc::SYS_tkill,
        libc::SYS_move_pages,
        libc::SYS_migrate_pages,
        // Mounts and namespaces.
        libc::SYS_mount,
        libc::SYS_umount2,
        libc::SYS_pivot_root,
        libc::SYS_chroot,
        libc::SYS_unshare,
        libc::SYS_setns,
        libc::SYS_fsopen,
        libc::SYS_fsconfig,
        libc::SYS_fsmount,
        libc::SYS_fspick,
        libc::SYS_open_tree,
        libc::SYS_move_mount,
        libc::SYS_mount_setattr,
        libc::SYS_open_by_handle_at,
        libc::SYS_name_to_handle_at,
        // Kernel surface.
        libc::SYS_bpf,
        libc::SYS_userfaultfd,
        libc::SYS_perf_event_open,
        libc::SYS_io_uring_setup,
        libc::SYS_io_uring_enter,
        libc::SYS_io_uring_register,
        libc::SYS_keyctl,
        libc::SYS_add_key,
        libc::SYS_request_key,
        libc::SYS_init_module,
        libc::SYS_finit_module,
        libc::SYS_delete_module,
        libc::SYS_kexec_load,
    ];
    #[cfg(target_arch = "x86_64")]
    calls.extend([
        libc::SYS_fork,
        libc::SYS_vfork,
        libc::SYS_iopl,
        libc::SYS_ioperm,
    ]);
    calls.sort_unstable();
    calls.dedup();
    calls
}

/// The namespace flags of `clone`: a new user namespace is the way into most
/// of the kernel a contained process otherwise can't reach.
const CLONE_NAMESPACES: &[u64] = &[
    libc::CLONE_NEWNS as u64,
    libc::CLONE_NEWUTS as u64,
    libc::CLONE_NEWIPC as u64,
    libc::CLONE_NEWUSER as u64,
    libc::CLONE_NEWPID as u64,
    libc::CLONE_NEWNET as u64,
    libc::CLONE_NEWCGROUP as u64,
];

fn condition(
    arg: u8,
    len: SeccompCmpArgLen,
    op: SeccompCmpOp,
    value: u64,
) -> Result<SeccompCondition, ContainError> {
    SeccompCondition::new(arg, len, op, value)
        .map_err(|e| ContainError(format!("couldn't build the seccomp filter: {e}")))
}

fn rule(conditions: Vec<SeccompCondition>) -> Result<SeccompRule, ContainError> {
    SeccompRule::new(conditions)
        .map_err(|e| ContainError(format!("couldn't build the seccomp filter: {e}")))
}

/// Calls refused only for some arguments: `clone` that makes a process
/// rather than a thread, or a namespace; signals and limits aimed at any
/// process but this one (the app is the obvious target).
fn conditional_rules(self_pid: u64) -> Result<Vec<(i64, Vec<SeccompRule>)>, ContainError> {
    use SeccompCmpArgLen::{Dword, Qword};
    let not_self = |arg| condition(arg, Dword, SeccompCmpOp::Ne, self_pid);
    let mut clone = vec![rule(vec![condition(
        0,
        Qword,
        SeccompCmpOp::MaskedEq(libc::CLONE_THREAD as u64),
        0,
    )?])?];
    for flag in CLONE_NAMESPACES {
        clone.push(rule(vec![condition(
            0,
            Qword,
            SeccompCmpOp::MaskedEq(*flag),
            *flag,
        )?])?);
    }
    Ok(vec![
        (libc::SYS_clone, clone),
        (libc::SYS_kill, vec![rule(vec![not_self(0)?])?]),
        (libc::SYS_tgkill, vec![rule(vec![not_self(0)?])?]),
        (libc::SYS_rt_sigqueueinfo, vec![rule(vec![not_self(0)?])?]),
        (libc::SYS_rt_tgsigqueueinfo, vec![rule(vec![not_self(0)?])?]),
        // 0 is this process.
        (
            libc::SYS_prlimit64,
            vec![rule(vec![
                not_self(0)?,
                condition(0, Dword, SeccompCmpOp::Ne, 0)?,
            ])?],
        ),
    ])
}

fn arch() -> Result<TargetArch, ContainError> {
    std::env::consts::ARCH
        .try_into()
        .map_err(|_| ContainError("this processor has no seccomp filter".into()))
}

fn compile(rules: BTreeMap<i64, Vec<SeccompRule>>, errno: i32) -> Result<BpfProgram, ContainError> {
    let filter = SeccompFilter::new(
        rules,
        SeccompAction::Allow,
        SeccompAction::Errno(errno as u32),
        arch()?,
    )
    .map_err(|e| ContainError(format!("couldn't build the seccomp filter: {e}")))?;
    filter.try_into().map_err(|e: seccompiler::BackendError| {
        ContainError(format!("couldn't compile the seccomp filter: {e}"))
    })
}

/// The filters, in the order they are applied: the denials (`EPERM`), and
/// `clone3` as unknown (`ENOSYS`). `clone3` takes its flags in memory a
/// filter can't read, so it is refused in the one way the C library takes
/// as "use `clone`", whose flags the first filter checks.
fn seccomp_programs(self_pid: u64) -> Result<[BpfProgram; 2], ContainError> {
    let mut rules: BTreeMap<i64, Vec<SeccompRule>> = denied_calls()
        .into_iter()
        .map(|call| (call, Vec::new()))
        .collect();
    for (call, conditional) in conditional_rules(self_pid)? {
        rules.insert(call, conditional);
    }
    let denials = compile(rules, libc::EPERM)?;
    let clone3 = compile(
        BTreeMap::from([(libc::SYS_clone3, Vec::new())]),
        libc::ENOSYS,
    )?;
    Ok([denials, clone3])
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

    // Files from V1, TCP from V4, signals and abstract sockets from V6; an
    // older kernel enforces what it has (the seccomp filter covers sockets
    // and signals there too).
    let abi = ABI::V6;
    let landlock_error = |e: &dyn std::fmt::Display| ContainError(format!("Landlock: {e}"));
    let mut ruleset = Ruleset::default()
        .handle_access(AccessFs::from_all(abi))
        .map_err(|e| landlock_error(&e))?
        .handle_access(AccessNet::from_all(abi))
        .map_err(|e| landlock_error(&e))?
        .scope(Scope::from_all(abi))
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

    let self_pid = u64::from(std::process::id());
    for program in seccomp_programs(self_pid)? {
        seccompiler::apply_filter(&program)
            .map_err(|e| ContainError(format!("couldn't apply the seccomp filter: {e}")))?;
    }
    Ok(())
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
        for call in [
            libc::SYS_sendmsg,
            libc::SYS_recvmsg,
            libc::SYS_pidfd_getfd,
            libc::SYS_io_uring_enter,
            libc::SYS_process_vm_readv,
            libc::SYS_userfaultfd,
            libc::SYS_bpf,
        ] {
            assert!(calls.contains(&call), "{call}");
        }
        // Threads (clone, checked by its flags) and memory (mmap) stay:
        // PDFium needs them.
        assert!(!calls.contains(&libc::SYS_clone));
        assert!(!calls.contains(&libc::SYS_clone3));
        assert!(!calls.contains(&libc::SYS_mmap));
        let conditional = conditional_rules(1234).unwrap();
        assert!(conditional.iter().any(|(call, _)| *call == libc::SYS_clone));
        assert!(conditional.iter().any(|(call, _)| *call == libc::SYS_kill));
        assert!(seccomp_programs(1234).is_ok());
    }
}
